use serde::Serialize;
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::JsonValue;
use log::warn;

#[derive(Debug,Serialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub blurb: Option<String>,
    number_js: i32,
    rank: Option<f32>,
}

pub struct DB {
    pool: Pool<Postgres>,
}

impl DB {
    pub async fn new(un: &str, pw: &str, hs: &str, ep: &str) -> DB {
        let pool = PgPoolOptions::new()
                        .max_connections(5)
                        .connect(
                            format!("postgres://{un}:{pw}@{hs}/{ep}").as_str()
                            ).await
                        .expect("Failed to connect to postgres server!");

        // Run setup tasks
        let query = r#"CREATE TABLE IF NOT EXISTS webpages (
            id serial PRIMARY KEY,
            title TEXT NOT NULL,
            blurb TEXT,
            content TEXT NOT NULL,
            number_js INTEGER NOT NULL,
            tf JSON,
            url TEXT NOT NULL,
            search_vector tsvector
        );
        CREATE INDEX ix_search_vector ON webpages USING GIN (search_vector);
        CREATE OR REPLACE FUNCTION update_webpage_content() RETURNS trigger AS $$
BEGIN
    new.search_vector := setweight(to_tsvector(coalesce(new.title, '')), 'A') ||
        setweight(to_tsvector(coalesce(new.blurb, '')), 'B') ||
        setweight(to_tsvector(coalesce(new.content, '')), 'C') ||
        setweight(to_tsvector(coalesce(new.url, '')), 'D');
    return new;
END
$$ LANGUAGE plpgsql;
CREATE TRIGGER webpage_search_vector_update
BEFORE INSERT OR UPDATE
ON webpages
FOR EACH ROW EXECUTE PROCEDURE update_webpage_content();"#;

        match sqlx::query(query).execute(&pool).await {
            Ok(_) => {},
            Err(_) => warn!("Couldn't set up database!")
        }
        
        DB {
            pool
        }
    }

    pub async fn search(&self, input: &str) -> Option<Vec<SearchResult>>{
        match sqlx::query_as!(SearchResult, r#" 
SELECT title, url, blurb, number_js, rank
FROM (select title, url, blurb, number_js, ts_rank(search_vector, websearch_to_tsquery($1)) as rank from webpages)
Where rank > 0.1
ORDER BY rank DESC 
"#, input).fetch_all(&self.pool).await {
    Ok(results) => Some(results),
    Err(_) => None
}
    }

    pub async fn add_webpage(&self, title: String, url: String, blurb: String, content: String, number_js: i32, tf: String) {
        println!("Reached adding webpage {title}");
        match sqlx::query!(r#"
INSERT INTO webpages (title, url, blurb, content, number_js, tf) VALUES ($1, $2, $3, $4, $5, $6)
            "#, title, url, blurb, content, number_js, JsonValue::String(tf)).execute(&self.pool).await {
            Ok(_) => {},
            Err(_) => warn!("Couldn't set up database!")
        }
    }
}

