use serde::Deserialize;
use std::{collections::HashMap, sync::OnceLock};
use tower_lsp_server::lsp_types::{
    self, CompletionItem, CompletionItemKind, Documentation, InsertTextFormat, MarkupContent,
    MarkupKind,
};

#[derive(Deserialize)]
pub struct KeywordInfo {
    documentation: String,
    snippet_type: String,
}

type Keyword = String;
#[derive(Deserialize)]
pub struct IndexedDocumentation {
    keys_to_doc: HashMap<Keyword, KeywordInfo>,
    keys_with_shared_doc: HashMap<Keyword, Keyword>,
}

impl IndexedDocumentation {
    pub fn get_doc_for_word(&self, word: &str) -> Option<String> {
        if let Some(keyword_info) = self.keys_to_doc.get(word) {
            Some(keyword_info.documentation.clone())
        } else if let Some(alias) = self.keys_with_shared_doc.get(word) {
            Some(
                self.keys_to_doc
                    .get(alias)
                    .expect("indexed doc alias does not match a keyword")
                    .documentation
                    .clone(),
            )
        } else {
            None
        }
    }
}

pub static CA65_DOCUMENTATION: OnceLock<IndexedDocumentation> = OnceLock::new();
pub static INSTRUCTION_DOCUMENTATION: OnceLock<IndexedDocumentation> = OnceLock::new();
pub static MACPACK_DOCUMENTATION: OnceLock<HashMap<String, String>> = OnceLock::new();
pub static FEATURE_DOCUMENTATION: OnceLock<HashMap<String, String>> = OnceLock::new();

pub fn init() {
    parse_json_to_hashmaps();
    parse_json_to_completion_items();
}

#[inline]
fn parse_json_to_hashmaps() {
    if let Ok(doc) = serde_json::from_str::<IndexedDocumentation>(include_str!(
        "../../data/ca65-keyword-doc.json"
    )) {
        if CA65_DOCUMENTATION.set(doc).is_err() {
            eprintln!("CA65_KEYWORDS_MAP not able to be initialized");
        }
    }
    if let Ok(doc) = serde_json::from_str::<IndexedDocumentation>(include_str!(
        "../../data/65xx-instruction-doc.json"
    )) {
        if INSTRUCTION_DOCUMENTATION.set(doc).is_err() {
            eprintln!("INSTRUCTION_DOC not able to be initialized");
        }
    }
    if let Ok(doc) = serde_json::from_str::<HashMap<String, String>>(include_str!(
        "../../data/macpack-packages-doc.json"
    )) {
        if MACPACK_DOCUMENTATION.set(doc).is_err() {
            eprintln!("MACPACK_DOC not able to be initialized");
        }
    }
    if let Ok(doc) = serde_json::from_str::<HashMap<String, String>>(include_str!(
        "../../data/features-doc.json"
    )) {
        if FEATURE_DOCUMENTATION.set(doc).is_err() {
            eprintln!("FEATURES_DOCUMENTATION not able to be initialized");
        }
    }
}

pub static CA65_KEYWORD_COMPLETION_ITEMS: OnceLock<Vec<CompletionItem>> = OnceLock::new();
pub static INSTRUCTION_COMPLETION_ITEMS: OnceLock<Vec<CompletionItem>> = OnceLock::new();
pub static MACPACK_COMPLETION_ITEMS: OnceLock<Vec<CompletionItem>> = OnceLock::new();
pub static FEATURE_COMPLETION_ITEMS: OnceLock<Vec<CompletionItem>> = OnceLock::new();
#[inline]
fn parse_json_to_completion_items() {
    let snippets =
        serde_json::from_str::<HashMap<String, String>>(include_str!("../../data/snippets.json"))
            .expect("Could not parse snippets JSON");

    let ca65_documentation = CA65_DOCUMENTATION
        .get()
        .expect("Could not get CA65_DOCUMENTATION in init_completion_item_vecs()");
    let ca65_keyword_completion_items =
        get_completion_item_vec_from_indexed_documentation(ca65_documentation, &snippets, ".");
    CA65_KEYWORD_COMPLETION_ITEMS
        .set(ca65_keyword_completion_items)
        .expect("Could not set CA65_KEYWORD_COMPLETION_ITEMS");

    let instruction_documentation = INSTRUCTION_DOCUMENTATION
        .get()
        .expect("Could not get CA65_DOCUMENTATION in init_completion_item_vecs()");
    let instruction_completion_items = get_completion_item_vec_from_indexed_documentation(
        instruction_documentation,
        &snippets,
        "",
    );
    INSTRUCTION_COMPLETION_ITEMS
        .set(instruction_completion_items)
        .expect("Could not set CA65_KEYWORD_COMPLETION_ITEMS");

    let macpack_documentation = MACPACK_DOCUMENTATION
        .get()
        .expect("Could not get MACPACK_DOCUMENTATION in init_completion_item_vecs()");
    let macpack_completion_items =
        get_completion_item_vec_from_string_string_hashmap(macpack_documentation);
    MACPACK_COMPLETION_ITEMS
        .set(macpack_completion_items)
        .expect("Could not set MACACK_COMPLETION_ITEMS");

    let features_documentation = FEATURE_DOCUMENTATION
        .get()
        .expect("Could not get FEATURES_DOCUMENTATION in init_completion_item_vecs()");
    let features_completion_items =
        get_completion_item_vec_from_string_string_hashmap(features_documentation);
    FEATURE_COMPLETION_ITEMS
        .set(features_completion_items)
        .expect("Could not set FEATURE_COMPLETION_ITEMS");
}
fn get_completion_item_vec_from_indexed_documentation(
    doc: &IndexedDocumentation,
    snippets: &HashMap<String, String>,
    keyword_prepend_text: &str,
) -> Vec<CompletionItem> {
    vec![
        doc.keys_to_doc
            .iter()
            .map(|(keyword, keyword_info)| CompletionItem {
                filter_text: Some(format!("{keyword_prepend_text}{keyword}")),
                label: format!("{keyword_prepend_text}{keyword}"),
                kind: Some(CompletionItemKind::KEYWORD),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: keyword_info.documentation.clone(),
                })),
                insert_text: Some(
                    snippets
                        .get(&keyword_info.snippet_type)
                        .expect("Could not get snippet type for keyword")
                        .replace("%", keyword),
                ),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            })
            .collect::<Vec<CompletionItem>>(),
        doc.keys_with_shared_doc
            .iter()
            .map(|(alias, key)| {
                (
                    alias,
                    doc.keys_to_doc
                        .get(key)
                        .expect("Alias in IndexedDocumentation did not point to a key"),
                )
            })
            .map(|(alias, keyword_info)| CompletionItem {
                filter_text: Some(format!("{keyword_prepend_text}{alias}")),
                label: format!("{keyword_prepend_text}{alias}"),
                kind: Some(CompletionItemKind::KEYWORD),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: keyword_info.documentation.clone(),
                })),
                insert_text: Some(
                    snippets
                        .get(keyword_info.snippet_type.as_str())
                        .expect("Could not get snippet type for keyword")
                        .replace("%", alias),
                ),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            })
            .collect(),
    ]
    .concat()
}

fn get_completion_item_vec_from_string_string_hashmap(
    doc: &HashMap<String, String>,
) -> Vec<CompletionItem> {
    doc.iter()
        .map(|(keyword, documentation_text)| CompletionItem {
            filter_text: Some(keyword.clone()),
            label: keyword.clone(),
            kind: Some(CompletionItemKind::MODULE),
            documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: documentation_text.clone(),
            })),
            ..Default::default()
        })
        .collect()
}
