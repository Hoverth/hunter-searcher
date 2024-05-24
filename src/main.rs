use log::{debug, info};
use std::env;

mod crawler;
mod db;
mod app;

use crate::crawler::CrawlerBuilder;
use crate::app::serve;
use crate::db::DB;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let db = DB::new(
        &env::var("POSTGRES_USER").expect("No POSTGRES_USER env var set!"), 
        &env::var("POSTGRES_PASSWORD").expect("No POSTGRES_PASSWORD env var set!"), 
        &env::var("POSTGRES_HOST").expect("No POSTGRES_HOST env var set!"), 
        &env::var("POSTGRES_DB").expect("No POSTGRES_DB env var set!")
        ).await;

    info!("Started crawler!");

    let mut crawler = CrawlerBuilder::new("hunter-searcher crawler/v0.1.0")
                                    .max_depth(5)
                                    .build();
    
    debug!("Created Crawler from builder");

    let _ = crawler.crawl(&db, "https://example.com").await;

    serve(db).await;
}
