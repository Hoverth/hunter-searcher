use serde::Serialize;
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use log::{warn, info, debug};

/*
CREATE TABLE IF NOT EXISTS webpages (
    id serial PRIMARY KEY,
    title TEXT NOT NULL,
    blurb TEXT,
    content TEXT NOT NULL,
    number_js INTEGER NOT NULL,
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
FOR EACH ROW EXECUTE PROCEDURE update_webpage_content();
*/

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
        match sqlx::query!(r#"
        CREATE TABLE IF NOT EXISTS webpages (
            id serial PRIMARY KEY,
            title TEXT NOT NULL,
            blurb TEXT,
            content TEXT NOT NULL,
            number_js INTEGER NOT NULL,
            url TEXT NOT NULL,
            search_vector tsvector
        );"#).execute(&pool).await {
            Err(err) => println!("Failed on create table: {err:?}"),
            Ok(_) => {},
        }
        match sqlx::query!(
                "CREATE INDEX IF NOT EXISTS ix_search_vector ON webpages USING GIN (search_vector);"
                ).execute(&pool).await { 
            Err(err) => println!("failed on create index: {err:?}"),
            Ok(_) => {},
        }
        
        match sqlx::query!(r#"CREATE OR REPLACE FUNCTION update_webpage_content() RETURNS trigger AS $$
        BEGIN
            new.search_vector := setweight(to_tsvector(coalesce(new.title, '')), 'A') ||
                setweight(to_tsvector(coalesce(new.blurb, '')), 'B') ||
                setweight(to_tsvector(coalesce(new.content, '')), 'C') ||
                setweight(to_tsvector(coalesce(new.url, '')), 'D');
            return new;
        END
        $$ LANGUAGE plpgsql;"#).execute(&pool).await {
            Err(err) => println!("failed on create function: {err:?}"),
            Ok(_) => {},
        }

        match sqlx::query!(
                "DROP TRIGGER IF EXISTS webpage_search_vector_update ON webpages;"
                ).execute(&pool).await {
            Err(err) => println!("Failed to drop existing trigger: {err:?}"),
            Ok(_) => {},
        }

        match sqlx::query!(r#"
                CREATE TRIGGER webpage_search_vector_update
                BEFORE INSERT OR UPDATE
                ON webpages
                FOR EACH ROW EXECUTE PROCEDURE update_webpage_content();"#
                ).execute(&pool).await {
            Ok(_) => {},
            Err(err) => println!("failed on create trigger: {err:?}"),
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
                ORDER BY rank DESC"#, input
                ).fetch_all(&self.pool).await {
            Ok(results) => Some(results),
            Err(_) => None
        }
    }

    pub async fn add_webpage(&self, title: String, url: String, blurb: String, content: String, number_js: i32) {
        debug!("Adding {url} to database...");

        match sqlx::query_as!(TCU, "SELECT title, url, content FROM webpages WHERE url = $1", url).fetch_one(&self.pool).await {
            Ok(res) => {
                if res.title == title && 
                   res.content == content && 
                   res.url == url
                   { info!("Already in database, skipping..."); return; }
                else if res.url == url {
                    debug!("Database entry stale, deleting...");
                    self.drop_index(res.url).await;
                }
            }
            Err(_) => {}
        };

        match sqlx::query!(r#"INSERT INTO webpages (title, url, blurb, content, number_js) VALUES ($1, $2, $3, $4, $5)"#, title, url, blurb, content, number_js).execute(&self.pool).await {
            Ok(_) => { info!("Added {title}, {url} to database successfully!"); },
            Err(_) => warn!("Couldn't add to database!")
        }
    }

    async fn drop_index(&self, url: String) {
        match sqlx::query!("DELETE FROM webpages WHERE url = $1", url).execute(&self.pool).await {
            Ok(_) => debug!("Deleted index {url}..."),
            Err(_) => warn!("Couldn't delete index with {url}!")
        }
    }
}
struct TCU {
    title: String,
    content: String,
    url: String
}
