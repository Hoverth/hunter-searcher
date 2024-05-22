use std::collections::HashMap;

/// Index Entry - A index entry format struct
#[derive(Clone,Debug)]
pub struct IndexEntry {
    /// The URL of the document
    pub url: String, 
    /// The number of Javascript scripts in the document
    pub number_js: usize, 
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
    for i in index {
        println!("{i:?}");
    }
}

