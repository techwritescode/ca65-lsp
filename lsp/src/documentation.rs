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
pub struct MultiKeySingleDoc {
    keys_to_doc: HashMap<Keyword, KeywordInfo>,
    keys_with_shared_doc: HashMap<Keyword, Keyword>,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum DocumentationKind {
    Ca65Keyword,
    Ca65DotOperator,
    Instruction,
    Feature,
    Macpack,
}
pub static DOCUMENTATION_COLLECTION: OnceLock<HashMap<DocumentationKind, MultiKeySingleDoc>> =
    OnceLock::new();

impl MultiKeySingleDoc {
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

pub fn init() {
    init_docs();
    init_completion_items();
    init_context_types();
}

#[inline]
fn init_docs() {
    let docs: HashMap<DocumentationKind, MultiKeySingleDoc> =
        Vec::<(DocumentationKind, &str)>::from([
            (
                DocumentationKind::Ca65Keyword,
                include_str!("../../data/ca65-keyword-doc.json"),
            ),
            (
                DocumentationKind::Ca65DotOperator,
                include_str!("../../data/ca65-dot-operators-doc.json"),
            ),
            (
                DocumentationKind::Instruction,
                include_str!("../../data/65xx-instruction-doc.json"),
            ),
            (
                DocumentationKind::Macpack,
                include_str!("../../data/macpack-packages-doc.json"),
            ),
            (
                DocumentationKind::Feature,
                include_str!("../../data/features-doc.json"),
            ),
        ])
        .into_iter()
        .filter_map(|(kind, file_contents)| {
            if let Ok(doc) = serde_json::from_str::<MultiKeySingleDoc>(file_contents) {
                return Some((kind, doc));
            }
            None
        })
        .collect();

    if DOCUMENTATION_COLLECTION.set(docs).is_err() {
        eprintln!("Could not set documentation collection");
    }
}

pub static COMPLETION_ITEMS_COLLECTION: OnceLock<HashMap<DocumentationKind, Vec<CompletionItem>>> =
    OnceLock::new();

#[inline]
fn init_completion_items() {
    let snippets =
        serde_json::from_str::<HashMap<String, String>>(include_str!("../../data/snippets.json"))
            .expect("Could not parse snippets JSON");

    let items: HashMap<DocumentationKind, Vec<CompletionItem>> = DOCUMENTATION_COLLECTION
        .get()
        .expect("Could not get documentation collection")
        .into_iter()
        .map(|(kind, doc)| {
            (
                kind.clone(),
                get_completion_item_vec_from_multi_key_single_doc(doc, &snippets),
            )
        })
        .collect();

    if COMPLETION_ITEMS_COLLECTION.set(items).is_err() {
        eprintln!("Could not set completion items collection");
    }
}

fn get_completion_item_vec_from_multi_key_single_doc(
    doc: &MultiKeySingleDoc,
    snippets: &HashMap<String, String>,
) -> Vec<CompletionItem> {
    vec![
        doc.keys_to_doc
            .iter()
            .map(|(keyword, keyword_info)| CompletionItem {
                filter_text: Some(keyword.clone()),
                label: keyword.clone(),
                kind: Some(CompletionItemKind::KEYWORD),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: keyword_info.documentation.clone(),
                })),
                insert_text: Some(
                    snippets
                        .get(&keyword_info.snippet_type)
                        .expect("Could not get snippet type for keyword")
                        .replace("%", keyword)
                        .replace("@", &keyword[1..]),
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
                        .expect("Alias in documentation did not point to a key"),
                )
            })
            .map(|(alias, keyword_info)| CompletionItem {
                filter_text: Some(alias.clone()),
                label: alias.clone(),
                kind: Some(CompletionItemKind::KEYWORD),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: keyword_info.documentation.clone(),
                })),
                insert_text: Some(
                    snippets
                        .get(keyword_info.snippet_type.as_str())
                        .expect("Could not get snippet type for keyword")
                        .replace("%", alias)
                        .replace("@", &alias[1..]),
                ),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            })
            .collect(),
    ]
    .concat()
}

pub static CA65_CONTEXT_TYPES: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();

#[inline]
pub fn init_context_types() {
    CA65_CONTEXT_TYPES.set(
        serde_json::from_str::<HashMap<String, Vec<String>>>(
            include_str!("../../data/ca65-context-types.json")
        ).expect("Could not parse ca65 context types json")
    ).expect("Could not set ca65 context types OnceLock");
}
