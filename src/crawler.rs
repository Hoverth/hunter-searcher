use reqwest::{self, Client};
use url::Url;
use texting_robots::{Robot, get_robots_url};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use tokio::time::sleep;
use log::{debug, info};

use crate::lexer::Lexer;
use crate::index::IndexEntry;

#[derive(Clone,Debug)]
pub struct Crawler {
    user_agent: String,
    client: Client,
    robot_records: HashMap<String, String>,
    websites: Vec<String>,
    max_depth: i32,
    whitelist_en: bool,
    whitelist: Vec<String>,
    blacklist_en: bool,
    blacklist: Vec<String>,
    delay_time: Duration,
    index: Vec<IndexEntry>,
}

impl Crawler {
    pub async fn crawl(&mut self, seed_url: &str) -> Vec<IndexEntry> {
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
                    if !url_host.contains(item) { info!("Website in whitelist!"); skip = true }
                }
            }
            if skip { info!("Skipping..."); continue }

            Self::push_dedup(&mut self.websites, url_host);
            
            latest_index = Some(self.index_url(url.as_str()).await);
        
            for link in &latest_index.as_ref().expect("no latest index!").links {
                if !to_get_links.contains(link) {
                    to_get_links.push_back(link.to_string())
                }
            }

            println!("Getting \"{:<60}\" ({:_>6} left, {:_>6} total sites)", latest_index.as_ref().expect("no latest index!").url, to_get_links.len(), self.websites.len());

            self.index.push(latest_index.unwrap())
        }

        self.index.clone()
    }
    
    pub async fn index_url(&self, url: &str) -> IndexEntry {
        info!("Indexing {url}...");
        let resp = self.request_body(url).await.expect("error making request");

        let mut page_urls: Vec<String> = Vec::new();
    
        let dom = tl::parse(resp.as_str(), tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        // TODO add decoding escaped html characters

        debug!("Constructing text...");
        let mut text: String = String::new();
        for tag in vec!["body", "title", "img[alt]"] {
            for child in dom.query_selector(tag).unwrap() {
                if !tag.contains("[") {
                    let text_portion = child.get(parser)
                                            .unwrap().inner_text(parser);
                    text += &text_portion;
                    text += " ";
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
        debug!("{text}");
        
        debug!("Generating TF table...");
        let mut tf = HashMap::<String, usize>::new();

        for token in Lexer::new(&text.chars().collect::<Vec<_>>()){
            let term = token.iter().map(|x| x.to_ascii_uppercase()).collect::<String>();
            if let Some(freq) = tf.get_mut(&term){
                *freq += 1;
            } else {
                tf.insert(term, 1);
            }
        }

        //TODO: do we bother with stopwords and lemmatisation?
        
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
    
        IndexEntry{ url: url.to_string(), links: page_urls, number_js: 0, tf }
    }
    
    async fn request_body(&self, url: &str) -> Option<String> {
        let mut body: Option<String> = None;
        let resp = self.client.execute(
                self.client.get(url).build().expect("failed to build request!")
            )
            .await
            .expect("request failed!");

        if resp.status() == 200 {
            body = Some(resp.text().await.expect("error getting text"));
        }
        sleep(self.delay_time).await;
        body
    }

    fn push_dedup(vec: &mut Vec<String>, thing: String) {
        if !vec.contains(&thing) {
            vec.push(thing);
        }
    }
    
    fn resolve_relative_url(url: &str, href: String) -> String {
        Url::parse(url).expect("can't parse url")
                       .join(href.as_str()).expect("can't join relative path!")
                       .as_str().to_string()
    }
}

pub struct CrawlerBuilder {
    crawler: Crawler
}

impl CrawlerBuilder {
    pub fn new (user_agent: &str) -> CrawlerBuilder {
        CrawlerBuilder {
            crawler: Crawler {
                user_agent: user_agent.to_string(),
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

    pub fn add_whitelist(mut self, whitelist: Vec<String>) -> CrawlerBuilder {
        self.crawler.whitelist_en = true;
        self.crawler.whitelist = whitelist.to_owned();
        self
    }
    
    pub fn add_blacklist(mut self, blacklist: Vec<String>) -> CrawlerBuilder {
        self.crawler.blacklist_en = true;
        self.crawler.blacklist = blacklist.to_owned();
        self
    }
    
    pub fn delay_time(mut self, duration: Duration) -> CrawlerBuilder {
        self.crawler.delay_time = duration;
        self
    }

    pub fn max_depth(mut self, depth: i32) -> CrawlerBuilder {
        self.crawler.max_depth = depth;
        self
    }

    pub fn build(&self) -> Crawler {
        self.crawler.clone()
    }
}
