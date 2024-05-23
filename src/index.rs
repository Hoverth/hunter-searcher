use std::collections::HashMap;

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
    /// The term frequency table for the document
    pub tf: HashMap<String, usize>, 
    /// The text content of the document
    pub content: String, 
    /// A small blurb for the document
    pub blurb: String, 
}

pub async fn process_index(index: Vec<IndexEntry>) {
    let db = DB::new("archive", "archive", "localhost:5433", "archive_db").await;
    for i in index {
        db.add_webpage(i.title, i.url, String::from("blurb"), i.content, i.number_js.try_into().expect("Failed to convert!"), String::from("")).await;
    }
}

