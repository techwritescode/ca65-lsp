use std::{
    collections::HashMap,
    sync::OnceLock
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Ca65Doc {
    keywords_to_markdown: HashMap<String, String>,
    aliases: HashMap<String, String>,
}

impl Ca65Doc {
    pub fn get_doc_for_word(&self, word: &str) -> Option<String> {
        if let Some(doc) = self.keywords_to_markdown.get(word) {
            Some(doc.clone())
        } else if let Some(alias) = self.aliases.get(word) {
            Some(self.keywords_to_markdown.get(alias).expect("ca65 doc alias does not match a keyword").clone())
        } else {
            None
        }
    }
}

pub static CA65_DOC: OnceLock<Ca65Doc> = OnceLock::new();

pub fn parse_json_to_hashmap() {
    if let Ok(doc) = serde_json::from_str::<Ca65Doc>(include_str!("../../data/ca65-keyword-doc.json")) {
        if !CA65_DOC.set(doc).is_err() {
            eprintln!("CA65_KEYWORDS_MAP not able to be initialized");
        }
    }
}