use std::{
    collections::HashMap,
    sync::OnceLock,
};
use serde::Deserialize;

type Key = String;
type Documentation = String;

#[derive(Deserialize)]
pub struct IndexedDocumentation {
    keys_to_doc: HashMap<Key, Documentation>,
    keys_with_shared_doc: HashMap<Key, Key>,
}

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
    pub fn get_vec_of_all_entries(&self) -> Vec<(String, String)> {
        self.keys_to_doc
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

pub static CA65_DOCUMENTATION: OnceLock<IndexedDocumentation> = OnceLock::new();
pub static OPCODE_DOCUMENTATION: OnceLock<IndexedDocumentation> = OnceLock::new();

pub fn init_documentation_maps() {
    if let Ok(doc) = serde_json::from_str::<IndexedDocumentation>(include_str!("../../data/ca65-keyword-doc.json")) {
        if CA65_DOCUMENTATION.set(doc).is_err() {
            eprintln!("CA65_KEYWORDS_MAP not able to be initialized");
        }
    }
    if let Ok(doc) = serde_json::from_str::<IndexedDocumentation>(include_str!("../../data/65xx-instruction-doc.json")) {
        if OPCODE_DOCUMENTATION.set(doc).is_err() {
            eprintln!("OPCODE_DOC not able to be initialized");
        }
    }
}