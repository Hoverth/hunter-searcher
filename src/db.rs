use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;

struct DB {
    pool: Pool<Postgres>,
}

impl DB {
    pub async fn new(un: &str, pw: &str, hs: &str, ep: &str) -> DB {
        // alright, postgresql with a relational db of tokens <-> documents i guess
        //
        // * Token table
        //   - token - must be uppercase, not null
        //
        // * Document table
        //   -
    
        DB {
            pool: PgPoolOptions::new()
                        .max_connections(5)
                        .connect(
                            format!("postgres://{un}:{pw}@{hs}/{ep}").as_str()
                            ).await
                        .expect("Failed to connect to postgres server!")
        }
    }
}

