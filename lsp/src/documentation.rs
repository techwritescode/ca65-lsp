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

#[derive(Deserialize)]
struct Ca65KeywordSnippet {
    snippet_text: String,
    members: Vec<String>,
}
static CA65_KEYWORD_SNIPPETS: OnceLock<HashMap<String, Ca65KeywordSnippet>> = OnceLock::new();
static KEYWORDS_TO_SNIPPETS: OnceLock<HashMap<String, String>> = OnceLock::new();
pub fn init_ca65_keyword_snippets() {
    if let Ok(snippets) = serde_json::from_str::<HashMap<String, Ca65KeywordSnippet>>(include_str!("../../data/ca65-keyword-snippets.json")) {
        if CA65_KEYWORD_SNIPPETS.set(snippets).is_err() {
            eprintln!("CA65_KEYWORD_SNIPPETS not able to be initialized");
        } else {
            let mut keywords_to_snippets: HashMap<String, String> = HashMap::new();
            CA65_KEYWORD_SNIPPETS.get().unwrap().iter()
                .for_each(|(snippet_name, snippet)| {
                    snippet.members.iter()
                        .for_each(|keyword| {
                            keywords_to_snippets.insert(keyword.clone(), snippet_name.clone());
                        })
                });
            KEYWORDS_TO_SNIPPETS.set(keywords_to_snippets).unwrap();
        }
    }

}
pub fn get_ca65_keyword_snippet_text(keyword: &str) -> String {
    let ca65_keyword_snippets = CA65_KEYWORD_SNIPPETS.get().unwrap();
    let keywords_to_snippets = KEYWORDS_TO_SNIPPETS.get().unwrap();
    let snippet_name = keywords_to_snippets.get(keyword).unwrap();
    let snippet = ca65_keyword_snippets.get(snippet_name).unwrap();
    snippet.snippet_text.replace("cmd", keyword)
}