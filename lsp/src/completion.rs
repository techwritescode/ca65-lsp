use crate::asm_server::State;
use crate::codespan::FileId;
use crate::instructions;
use crate::symbol_cache::{symbol_cache_get, SymbolType};
use crate::documentation::{CA65_KEYWORD_COMPLETION_ITEMS, FEATURE_COMPLETION_ITEMS, MACPACK_COMPLETION_ITEMS, OPCODE_COMPLETION_ITEMS};
use codespan::Position;
use tower_lsp_server::lsp_types::{CompletionItem, CompletionItemKind, CompletionItemLabelDetails};

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
            OPCODE_COMPLETION_ITEMS.get().expect("Could not get OPCODE_COMPLETION_ITEMS").clone()
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

pub struct Ca65KeywordCompletionProvider;

impl CompletionProvider for Ca65KeywordCompletionProvider {
    fn completions_for(
        &self,
        state: &State,
        id: FileId,
        position: Position,
    ) -> Vec<CompletionItem> {
        CA65_KEYWORD_COMPLETION_ITEMS.get().expect("Could not get ca65 completion items in completion provider").clone()
    }
}

pub struct MacpackCompletionProvider;

impl CompletionProvider for MacpackCompletionProvider {
    fn completions_for(
        &self,
        state: &State,
        id: FileId,
        position: Position
    ) -> Vec<CompletionItem> {
        if state.files.line_tokens(id, position).last().is_some_and(|tok| tok.lexeme == ".macpack") {
            MACPACK_COMPLETION_ITEMS.get().expect("Could not get MACPACK_COMPLETION_ITEMS in completion provider").clone()
        } else {
            Vec::new()
        }
    }
}

pub struct FeatureCompletionProvider;
impl CompletionProvider for FeatureCompletionProvider {
    fn completions_for(
        &self,
        state: &State,
        id: FileId,
        position: Position
    ) -> Vec<CompletionItem> {
        if state.files.line_tokens(id, position).last().is_some_and(|tok| tok.lexeme == ".feature") {
            FEATURE_COMPLETION_ITEMS.get().expect("Could not get FEATURE_COMPLETION_ITEMS in completion provider").clone()
        } else {
            Vec::new()
        }
    }
}