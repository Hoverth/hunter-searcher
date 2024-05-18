use std::collections::HashMap;

#[derive(Clone,Debug)]
pub struct IndexEntry {
    pub url: String,
    pub number_js: usize,
    pub links: Vec<String>,
    pub tf: HashMap<String, usize>,
}

pub fn process_index(index: Vec<IndexEntry>) {
    for i in index {
        println!("{i:?}");
    }

    // alright, postgresql with a relational db of tokens <-> documents i guess
}
