use log::{debug, info};
use std::env;
use clap::Parser;

mod crawler;
mod db;
mod app;

use crate::crawler::CrawlerBuilder;
use crate::app::serve;
use crate::db::DB;

#[derive(Parser, Debug)]
#[command(version, about, long_about = "A simple search engine in rust")]
struct Args {
    #[arg(index = 1, help = "either 'serve' or 'crawl'")]
    command: String,

    #[arg(long,short,default_value_t=String::new(),help="the url to crawl")]
    url: String,

    #[arg(long,short,default_value_t=1,help="the max amount of pages to crawl (set to -1 to infinitely crawl)")]
    depth: i32,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();
    
    let db = DB::new(
        &env::var("POSTGRES_USER").expect("No POSTGRES_USER env var set!"), 
        &env::var("POSTGRES_PASSWORD").expect("No POSTGRES_PASSWORD env var set!"), 
        &env::var("POSTGRES_HOST").expect("No POSTGRES_HOST env var set!"), 
        &env::var("POSTGRES_DB").expect("No POSTGRES_DB env var set!")
        ).await;

    if args.command == "crawl" {
        if args.url == "" { panic!("Cannot crawl a blank URL!"); }

        let mut url = args.url;
        if !url.starts_with("http") { url = "http://".to_owned() + &url }
        info!("Started crawler!");
        
        let mut crawler = CrawlerBuilder::new("hunter-searcher crawler/v0.1.0")
                                        .max_depth(args.depth)
                                        .build();
        
        debug!("Created Crawler from builder");

        let _ = crawler.crawl(&db, &url).await;
    }

    if args.command == "serve" {
        serve(db).await;
    }
}
