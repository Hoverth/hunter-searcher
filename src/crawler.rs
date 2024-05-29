use reqwest::{self, Client};
use url::Url;
use texting_robots::{Robot, get_robots_url};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use tokio::time::sleep;
use log::{debug, info, error};

use crate::db::DB;
 
/// Index Entry - A index entry format struct
#[derive(Clone,Debug)]
pub struct IndexEntry {
    /// The URL of the document
    pub url: String, 
    /// The number of Javascript scripts in the document
    pub number_js: usize, 
    /// The title of the page
    pub title: String,
    /// The links in the document
    pub links: Vec<String>, 
    /// The text content of the document (whitespace removed)
    pub content: String, 
    /// A small blurb for the document (the 10th-30th words)
    pub blurb: String, 
}

/// Crawler - A simple web crawler class implementation
#[derive(Clone,Debug)]
pub struct Crawler {
    /// The crawler's user-agent
    user_agent: String, 
    /// The reqwest client, for more efficient requesting
    client: Client,
    /// The store of `robots.txt` records, cached for efficient retrieval
    robot_records: HashMap<String, String>,
    /// A vector containing the domains of the websites the crawler has scraped
    websites: Vec<String>,
    /// The max amount of documents to get, decided in a first in, first out queue
    max_depth: i32,
    /// Whether the whitelist is enabled or not
    whitelist_en: bool,
    /// The whitelist contains domains or other patterns that the crawler will check that a
    /// document **must** have in it's URL
    whitelist: Vec<String>,
    /// Whether the blacklist is enabled or not
    blacklist_en: bool,
    /// The blacklist contains domains or other strings that the crawler will check that a
    /// document **must not** have in it's URL 
    ///
    /// **This overrides the blacklist** 
    blacklist: Vec<String>,
    /// The time between requests
    delay_time: Duration,
    /// The in-memory index of all the documents gotten this crawl
    index: Vec<IndexEntry>,
}

impl Crawler {
    /// Crawl from a seed URL
    pub async fn crawl(&mut self, db: &DB, seed_url: &str) -> Vec<IndexEntry> {
        let mut to_get_links: VecDeque<String> = VecDeque::new();
        let mut latest_index;
        to_get_links.push_back(seed_url.to_string());

        debug!("Entering crawling loop...");
        loop {
            if to_get_links.is_empty() { debug!("No links to get!"); break }
            if self.max_depth != -1 && self.index.len() >= self.max_depth.try_into().unwrap() { info!("Exiting loop as depth limit reached..."); break }
            
            let url = to_get_links.pop_front().expect("no urls!");
            let url_host = Url::parse(&url).expect("can't unwrap url")
                                    .host_str().unwrap().to_string();

            info!("Starting on {url}...");

            if self.robot_records.contains_key(&url_host) {
                let r = Robot::new(&self.user_agent, self.robot_records.get(&url_host).expect("failed to get key!").as_bytes()).unwrap();

                if !r.allowed(url.as_str()) { info!("Robots.txt dissallowed this path!"); continue }
            } else if let Some(resp) = self.request_body(&get_robots_url(url.as_str()).expect("failed to get robots.txt url!").to_string()).await {
                let r = Robot::new(&self.user_agent, resp.as_bytes()).unwrap();
                self.robot_records.insert(url_host.clone(), resp);

                if !r.allowed(url.as_str()) { info!("Robots.txt dissallowed this path!"); continue }
            } else {
                //TODO: mayhaps a default rebots.txt?
                self.robot_records.insert(url_host.clone(), String::new());
            }

            let mut skip: bool = false;
            if self.blacklist_en {
                for item in &self.blacklist {
                    if url_host.contains(item) { info!("Website in blacklist!"); skip = true }
                }
            }
            if self.whitelist_en {
                for item in &self.whitelist {
                    if !url_host.contains(item) { info!("Website not in whitelist!"); skip = true }
                }
            }
            if skip { info!("Skipping..."); continue }

            Self::push_dedup(&mut self.websites, url_host);
            
            latest_index = self.index_url(url.as_str()).await;
        
            for link in &latest_index.as_ref().expect("no latest index!").links {
                if !to_get_links.contains(link) {
                    to_get_links.push_back(link.to_string())
                }
            }

            println!("Getting \"{:<60}\" ({:_>6} left, {:_>6} total sites)", latest_index.as_ref().expect("no latest index!").url, to_get_links.len(), self.websites.len());

            let i = latest_index.clone().unwrap();
            db.add_webpage(i.title, i.url, i.blurb, i.content, i.number_js.try_into().expect("Failed to convert!")).await;

            self.index.push(latest_index.unwrap())
        }

        self.index.clone()
    }
    
    /// Index a single URL
    pub async fn index_url(&self, url: &str) -> Option<IndexEntry> {
        info!("Indexing {url}...");
        let resp = match self.request_body(url).await {
            Some(resp) => resp,
            None => { return None; }
        };

        let mut page_urls: Vec<String> = Vec::new();
    
        let dom = tl::parse(resp.as_str(), tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        // TODO add decoding escaped html characters
        let mut title: String = String::new();

        debug!("Constructing text...");
        let mut text: String = String::new();
        for tag in ["body", "title", "img[alt]"] {
            for child in dom.query_selector(tag).unwrap() {
                if !tag.contains('[') {
                    let child = child.get(parser).unwrap();
                    text += &child.inner_text(parser);
                    text += " ";

                    for sub in child.as_tag().unwrap().query_selector(parser, "script").unwrap() {
                        text = text.replace(&sub.get(parser).unwrap().inner_text(parser).into_owned(), "");
                    }

                    if tag.contains("title") {
                        title = child.inner_text(parser).to_string();
                    }
                } else {
                    let img = child.get(parser).unwrap();
                    let text_portion = match img.as_tag() {
                        Some(imgtag) => { 
                            match imgtag.attributes().get("alt") {
                                Some(text) => text.unwrap()
                                                  .try_as_utf8_str().unwrap(),
                                None => { continue }
                            } 
                        },
                        None => { continue }
                    };
                    text += &text_portion;
                    text += " ";
                }
            }
        }

        let mut number_js = 0;
        for _ in dom.query_selector("script").unwrap() {
            number_js += 1;
        }
        let text: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
        debug!("{text}");
        
        let words = text.split_whitespace().collect::<Vec<_>>();
        
        let content: String = text.to_string(); // we should trim whitespace and extract blurb here
    
        // not ideal TODO: make this better
        let blurb: String;
        if words.len() <= 5 { blurb = words.join(" ") } 
        else if words.len() < 30 && words.len() > 5 { blurb = words[5..].join(" "); }
        else { blurb = words[10..30].join(" "); }

        debug!("Collecting links...");
        for a in dom.query_selector("a[href]").unwrap(){
            let mut href = a.get(parser)
                     .expect("no node")
                     .as_tag()
                     .expect("no tag")
                     .attributes()
                     .get("href")
                     .expect("no href")
                     .expect("Attribute not found or malformed")
                     .try_as_utf8_str()
                     .expect("not utf8!")
                     .to_string()
                     .replace("&#x2F;", "/");
            
            if href == "/" ||
               href == "#" ||
               href == url 
               { continue };
    
            if let Some(hash) = href.find('#') {
                href.truncate(hash);
            }
            if let Some(query) = href.find('?') {
                href.truncate(query);
            }

            if href.starts_with("http://") || href.starts_with("https://") {
                Self::push_dedup(&mut page_urls, href)
            } else {
                Self::push_dedup(&mut page_urls, Self::resolve_relative_url(url, href))
            }
        }
    
        Some(IndexEntry{ 
            url: url.to_string(), 
            links: page_urls, 
            title,
            number_js, 
            content,
            blurb,
        })
    }
    
    /// Returns the body of a request as a string
    pub async fn request_body(&self, url: &str) -> Option<String> {
        let mut body: Option<String> = None;
        let resp = match self.client.execute(
                self.client.get(url).build().expect("failed to build request!")
            )
            .await {
                Ok(r) => r,
                Err(e) =>{ error!("Request failed to {url}: {e}"); return None; }
            };

        if resp.status() == 200 {
            body = Some(resp.text().await.expect("error getting text"));
        }
        sleep(self.delay_time).await;
        body
    }

    /// Simple utility function that pushes a value if it's not already in the vector
    fn push_dedup(vec: &mut Vec<String>, thing: String) {
        if !vec.contains(&thing) {
            vec.push(thing);
        }
    }
    
    /// Helper function to resolve a relative HREF from a document's URL
    fn resolve_relative_url(url: &str, href: String) -> String {
        Url::parse(url).expect("can't parse url")
                       .join(href.as_str()).expect("can't join relative path!")
                       .as_str().to_string()
    }
}

/// CrawlerBuilder - a builder for the Crawler class
pub struct CrawlerBuilder {
    /// The in-construction Crawler instance
    crawler: Crawler
}

#[allow(dead_code)]
impl CrawlerBuilder {
    /// Create a CrawlerBuilder with the specified user agent
    pub fn new (user_agent: &str) -> CrawlerBuilder {
        CrawlerBuilder {
            crawler: Crawler {
                user_agent: user_agent.to_string(),
                //TODO: only accept html
                client: Client::builder().user_agent(user_agent)
                               .build().unwrap(), 
                robot_records: HashMap::new(),
                websites: Vec::new(),
                max_depth: -1,
                whitelist_en: false,
                whitelist: Vec::new(),
                blacklist_en: false,
                blacklist: Vec::new(),
                delay_time: Duration::from_millis(1000),
                index: Vec::new(),
            }
        }
    }

    /// Add a whitelist to the Crawler
    pub fn add_whitelist(mut self, whitelist: Vec<String>) -> CrawlerBuilder {
        self.crawler.whitelist_en = true;
        self.crawler.whitelist = whitelist.to_owned();
        self
    }
    
    /// Add a blacklist to the Crawler
    pub fn add_blacklist(mut self, blacklist: Vec<String>) -> CrawlerBuilder {
        self.crawler.blacklist_en = true;
        self.crawler.blacklist = blacklist.to_owned();
        self
    }
    
    /// Adjust the default (1s) minimum delay between requests
    pub fn delay_time(mut self, duration: Duration) -> CrawlerBuilder {
        self.crawler.delay_time = duration;
        self
    }

    /// Adjust the maximum max depth of the crawl
    ///
    /// By default this is -1, which means no limit
    pub fn max_depth(mut self, depth: i32) -> CrawlerBuilder {
        self.crawler.max_depth = depth;
        self
    }

    /// Build the Crawler from the CrawlerBuilder
    pub fn build(&self) -> Crawler {
        self.crawler.clone()
    }
}
