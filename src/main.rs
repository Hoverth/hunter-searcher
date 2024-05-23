use log::{debug, info};

mod crawler;
mod lexer; 
mod db;
mod app;

use crate::crawler::CrawlerBuilder;
use crate::app::serve;
use crate::db::DB;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let db = DB::new("archive", "archive", "localhost:5433", "archive_db").await;

    info!("Started crawler!");

    let mut crawler = CrawlerBuilder::new("hunter-searcher crawler/v0.1.0")
                                    .max_depth(5)
                                    .build();
    
    debug!("Created Crawler from builder");

    let _ = crawler.crawl(&db, "https://example.com").await;

    serve(db).await;
}
