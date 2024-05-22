use log::{debug, info};

mod crawler;
mod lexer; 
mod index;
mod db;

use crate::crawler::CrawlerBuilder;
use crate::index::process_index;

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Started crawler!");

    let mut crawler = CrawlerBuilder::new("hunter-searcher crawler/v0.1.0")
                                    .max_depth(1)
                                    .build();
    
    debug!("Created Crawler from builder");

    let addition_to_index = crawler.crawl("https://example.com").await;

    process_index(addition_to_index);
}
