use crate::state::State;
use crate::codespan::FileId;
use crate::documentation::{
    CA65_DOT_OPERATOR_COMPLETION_ITEMS, CA65_KEYWORD_COMPLETION_ITEMS, FEATURE_COMPLETION_ITEMS, INSTRUCTION_COMPLETION_ITEMS, MACPACK_COMPLETION_ITEMS
};
use crate::symbol_cache::{symbol_cache_get, SymbolType};
use analysis::ScopeAnalyzer;
use codespan::Position;
use parser::TokenType;
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
            INSTRUCTION_COMPLETION_ITEMS
                .get()
                .expect("Could not get INSTRUCTION_COMPLETION_ITEMS")
                .clone()
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
        let scopes = state
            .scopes
            .get(&state.files.get_uri(id))
            .unwrap_or(&vec![])
            .clone();
        let byte_position = state
            .files
            .get(id)
            .position_to_byte_index(position)
            .unwrap_or(0);
        let scope = ScopeAnalyzer::search(&scopes, byte_position);

        let word_at_position = state
            .files
            .get(id)
            .get_word_at_position(position)
            .unwrap_or("");
        let has_namespace = word_at_position.contains(":");

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
                    let name = if has_namespace {
                        symbol.fqn.clone()
                    } else {
                        ScopeAnalyzer::remove_denominator(&scope, symbol.fqn.clone())
                    };

                    let postfix = if matches!(symbol.sym_type, SymbolType::Scope) {
                        "::"
                    } else {
                        ""
                    };

                    Some(CompletionItem {
                        label: format!("{name}{postfix}"),
                        filter_text: if has_namespace {
                            Some(symbol.fqn.clone())
                        } else {
                            Some(symbol.label.clone())
                        },
                        detail: Some(symbol.comment.to_owned()),
                        label_details: Some(CompletionItemLabelDetails {
                            detail: None,
                            description: state.files.get_uri_relative(symbol.file_id, id),
                        }),
                        kind: Some(match symbol.sym_type {
                            SymbolType::Label => CompletionItemKind::FUNCTION,
                            SymbolType::Constant => CompletionItemKind::CONSTANT,
                            SymbolType::Macro => CompletionItemKind::SNIPPET,
                            SymbolType::Scope => CompletionItemKind::MODULE,
                        }),
                        ..Default::default()
                    })
                }
            })
            .collect()
    }
}

pub struct Ca65DotOperatorCompletionProvider;
impl CompletionProvider for Ca65DotOperatorCompletionProvider {
    fn completions_for(
        &self,
        _state: &State,
        _id: FileId,
        _position: Position
    ) -> Vec<CompletionItem> {
        CA65_DOT_OPERATOR_COMPLETION_ITEMS
            .get()
            .expect("Could not get ca65 dot operator completion items in completion provider")
            .clone()
    }
}

pub struct Ca65KeywordCompletionProvider;

impl CompletionProvider for Ca65KeywordCompletionProvider {
    fn completions_for(
        &self,
        _state: &State,
        _id: FileId,
        _position: Position,
    ) -> Vec<CompletionItem> {
        CA65_KEYWORD_COMPLETION_ITEMS
            .get()
            .expect("Could not get ca65 completion items in completion provider")
            .clone()
    }
}

pub struct MacpackCompletionProvider;

impl CompletionProvider for MacpackCompletionProvider {
    fn completions_for(
        &self,
        state: &State,
        id: FileId,
        position: Position,
    ) -> Vec<CompletionItem> {
        if state
            .files
            .line_tokens(id, position)
            .iter()
            .filter(|tok| tok.token_type != TokenType::EOL)
            .nth_back(1)
            .is_some_and(|tok| tok.lexeme == ".macpack")
        {
            MACPACK_COMPLETION_ITEMS
                .get()
                .expect("Could not get MACPACK_COMPLETION_ITEMS in completion provider")
                .clone()
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
        position: Position,
    ) -> Vec<CompletionItem> {
        if state
            .files
            .line_tokens(id, position)
            .iter()
            .filter(|tok| tok.token_type != TokenType::EOL)
            .nth_back(1)
            .is_some_and(|tok| tok.lexeme == ".feature")
        {
            FEATURE_COMPLETION_ITEMS
                .get()
                .expect("Could not get FEATURE_COMPLETION_ITEMS in completion provider")
                .clone()
        } else {
            Vec::new()
        }
    }
}

