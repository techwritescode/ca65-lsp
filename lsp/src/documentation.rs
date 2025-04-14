use std::{
    collections::HashMap,
    sync::OnceLock,
};
use serde::Deserialize;
use tower_lsp_server::lsp_types;

#[derive(Deserialize)]
pub struct KeywordInfo {
    documentation: String,
    snippet_type: String,
}

#[derive(Deserialize)]
pub struct IndexedDocumentation {
    keys_to_doc: HashMap<String, KeywordInfo>,
    keys_with_shared_doc: HashMap<String, String>,
}

impl IndexedDocumentation {
    pub fn get_doc_for_word(&self, word: &str) -> Option<String> {
        if let Some(keyword_info) = self.keys_to_doc.get(word) {
            Some(keyword_info.documentation.clone())
        } else if let Some(alias) = self.keys_with_shared_doc.get(word) {
            Some(self.keys_to_doc.get(alias).expect("indexed doc alias does not match a keyword").documentation.clone())
        } else {
            None
        }
    }
}

pub static CA65_DOCUMENTATION: OnceLock<IndexedDocumentation> = OnceLock::new();
pub static OPCODE_DOCUMENTATION: OnceLock<IndexedDocumentation> = OnceLock::new();
pub static SNIPPETS: OnceLock<HashMap<String, String>> = OnceLock::new();

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
    if let Ok(snippets) = serde_json::from_str::<HashMap<String, String>>(include_str!("../../data/snippets.json")) {
        if SNIPPETS.set(snippets).is_err() {
            eprintln!("SNIPPETS not able to be initialized");
        }
    }
}

pub static CA65_KEYWORD_COMPLETION_ITEMS: OnceLock<Vec<lsp_types::CompletionItem>> = OnceLock::new();
pub fn init_completion_item_vecs() {
    let ca65_documentation = CA65_DOCUMENTATION.get().expect("Could not get CA65_DOCUMENTATION in init_completion_item_vecs()");

    let ca65_keyword_completion_items = get_completion_item_vec_from_indexed_documentation(ca65_documentation);

    CA65_KEYWORD_COMPLETION_ITEMS.set(ca65_keyword_completion_items).unwrap();
}
fn get_completion_item_vec_from_indexed_documentation(doc: &IndexedDocumentation) -> Vec<lsp_types::CompletionItem> {
    let snippets = SNIPPETS.get().expect("Could not get SNIPPETS in get_completion_item_vec_from_indexed_documentation()");
    doc
        .keys_to_doc
        .iter()
        .map(|(keyword, keyword_info)| lsp_types::CompletionItem {
            label: format!(".{keyword}"),
            kind: Some(lsp_types::CompletionItemKind::KEYWORD),
            documentation: Some(lsp_types::Documentation::MarkupContent(lsp_types::MarkupContent {
                kind: lsp_types::MarkupKind::Markdown,
                value: keyword_info.documentation.clone(),
            })),
            insert_text: Some(snippets
                .get(&keyword_info.snippet_type)
                .expect("Could not get snippet type for keyword")
                .replace("%", keyword)
            ),
            insert_text_format: Some(lsp_types::InsertTextFormat::SNIPPET),
            ..Default::default()
        })
        .collect()
}