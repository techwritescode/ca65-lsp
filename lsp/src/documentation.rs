use std::{
    collections::HashMap,
    sync::OnceLock
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct IndexedDocumentation {
    keys_to_doc: HashMap<String, String>,
    keys_with_shared_doc: HashMap<String, String>,
}

pub static CA65_DOC: OnceLock<IndexedDocumentation> = OnceLock::new();
pub static INSTRUCTION_DOC: OnceLock<IndexedDocumentation> = OnceLock::new();

impl IndexedDocumentation {
    pub fn get_doc_for_word(&self, word: &str) -> Option<String> {
        if let Some(doc) = self.keys_to_doc.get(word) {
            Some(doc.clone())
        } else if let Some(alias) = self.keys_with_shared_doc.get(word) {
            Some(self.keys_to_doc.get(alias).expect("indexed doc alias does not match a keyword").clone())
        } else {
            None
        }
    }
}


pub fn init_documentation_map() {
    if let Ok(doc) = serde_json::from_str::<IndexedDocumentation>(include_str!("../../data/ca65-keyword-doc.json")) {
        if CA65_DOC.set(doc).is_err() {
            eprintln!("CA65_KEYWORDS_MAP not able to be initialized");
        }
    }
    if let Ok(doc) = serde_json::from_str::<IndexedDocumentation>(include_str!("../../data/65xx-instruction-doc.json")) {
        if INSTRUCTION_DOC.set(doc).is_err() {
            eprintln!("INSTRUCTION_DOC not able to be initialized");
        }
    }
}