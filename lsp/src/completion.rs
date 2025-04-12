use tokio::io::join;
use crate::asm_server::State;
use crate::codespan::FileId;
use crate::instructions;
use crate::symbol_cache::{symbol_cache_get, SymbolType};
use codespan::Position;
use tower_lsp_server::lsp_types::{CompletionItem, CompletionItemKind, CompletionItemLabelDetails, Documentation, InsertTextFormat, MarkupContent, MarkupKind};
use crate::ca65_doc::CA65_DOC;

static BLOCK_CONTROL_COMMANDS: &[&str] = &[
    "scope", "proc", "macro", "enum", "union", "if", "repeat", "struct",
];

pub trait CompletionProvider {
    fn completions_for(&self, state: &State, id: FileId, position: Position)
        -> Vec<CompletionItem>;
}

pub struct InstructionCompletionProvider;

impl CompletionProvider for InstructionCompletionProvider {
    fn completions_for(
        &self,
        state: &State,
        id: FileId,
        position: Position,
    ) -> Vec<CompletionItem> {
        if state.files.show_instructions(id, position) {
            instructions::INSTRUCTION_MAP
                .get()
                .expect("Instructions not loaded")
                .iter()
                .map(|(opcode, description)| CompletionItem {
                    label: opcode.to_lowercase().to_owned(),
                    detail: Some(description.to_owned()),
                    kind: Some(CompletionItemKind::KEYWORD),
                    ..Default::default()
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

pub struct SymbolCompletionProvider;

impl CompletionProvider for SymbolCompletionProvider {
    fn completions_for(
        &self,
        state: &State,
        id: FileId,
        position: Position,
    ) -> Vec<CompletionItem> {
        let show_instructions = state.files.show_instructions(id, position); // Makes a naive guess at whether the current line contains an instruction. Doesn't work on lines with labels

        symbol_cache_get()
            .iter()
            .filter_map(|symbol| {
                if show_instructions
                    && matches!(symbol.sym_type, SymbolType::Label | SymbolType::Constant)
                {
                    None
                } else if !show_instructions && matches!(symbol.sym_type, SymbolType::Macro) {
                    None
                } else {
                    Some(CompletionItem {
                        label: symbol.label.to_owned(),
                        detail: Some(symbol.comment.to_owned()),
                        label_details: Some(CompletionItemLabelDetails{
                            detail: None,
                            description: state.files.get_uri_relative(symbol.file_id, id),
                        }),
                        kind: Some(match symbol.sym_type {
                            SymbolType::Label => CompletionItemKind::FUNCTION,
                            SymbolType::Constant => CompletionItemKind::CONSTANT,
                            SymbolType::Macro => CompletionItemKind::SNIPPET,
                        }),
                        ..Default::default()
                    })
                }
            })
            .collect()
    }
}

pub struct BlockControlCompletionProvider;

impl CompletionProvider for BlockControlCompletionProvider {
    fn completions_for(
        &self,
        state: &State,
        id: FileId,
        position: Position,
    ) -> Vec<CompletionItem> {
        if state.files.show_instructions(id, position) {
            BLOCK_CONTROL_COMMANDS
                .iter()
                .map(|command| CompletionItem {
                    label: (*command).to_string(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    insert_text: Some(format!(".{} $1\n\t$0\n.end{} ; End $1", *command, *command)),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

pub struct Ca65KeywordCompletionProvider;

impl CompletionProvider for Ca65KeywordCompletionProvider {
    fn completions_for(
        &self,
        state: &State,
        id: FileId,
        position: Position,
    ) -> Vec<CompletionItem> {
        CA65_DOC
            .get()
            .unwrap()
            .get_vec_of_all_entries()
            .iter()
            .map(|(k, v)| CompletionItem {
                label: k.clone(),
                kind: Some(CompletionItemKind::KEYWORD),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: v.clone(),
                })),
                ..Default::default()
            })
            .collect()
    }
}
