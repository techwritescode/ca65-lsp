use crate::documentation::{DocumentationKind, CA65_CONTEXT_TYPES, COMPLETION_ITEMS_COLLECTION};
use crate::{data::symbol::SymbolType, state::State};
use analysis::ScopeAnalyzer;
use codespan::FileId;
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
        if state.files.show_lhs_completions(id, position) {
            COMPLETION_ITEMS_COLLECTION
                .get()
                .expect("Could not get completion items collection for instructions")
                .get(&DocumentationKind::Instruction)
                .expect("Could not get instruction completion items")
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
        let file = &state.files.get(id);
        let show_lhs_completions = state.files.show_lhs_completions(id, position); // Makes a naive guess at whether the current line contains an instruction. Doesn't work on lines with labels
        let byte_position = file.file.position_to_byte_index(position).unwrap_or(0);
        let scope = ScopeAnalyzer::search(&file.scopes, byte_position);

        let word_at_position = file.file.get_word_at_position(position).unwrap_or("");
        let has_namespace = word_at_position.contains(":");

        file.symbols
            .iter()
            .filter_map(|symbol| {
                if show_lhs_completions
                    && matches!(symbol.sym_type, SymbolType::Label | SymbolType::Constant)
                {
                    None
                } else if !show_lhs_completions && matches!(symbol.sym_type, SymbolType::Macro) {
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
        _position: Position,
    ) -> Vec<CompletionItem> {
        COMPLETION_ITEMS_COLLECTION
            .get()
            .expect("Could not get completion items collection for ca65 dot operators")
            .get(&DocumentationKind::Ca65DotOperator)
            .expect("Could not get ca65 dot operator completion items")
            .clone()
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
        let context_types = CA65_CONTEXT_TYPES
            .get()
            .expect("Could not get context types for ca65 completion provider");
        let all_ca65_completion_items = COMPLETION_ITEMS_COLLECTION
            .get()
            .expect("Could not get completion items collection for ca65 keywords")
            .get(&DocumentationKind::Ca65Keyword)
            .expect("Could not get ca65 keyword completion items")
            .clone();

        if state.files.show_lhs_completions(id, position) {
            let lhs_context_items = context_types
                .get("lhs")
                .expect("Couldn't get lhs context items");
            return all_ca65_completion_items
                .into_iter()
                .filter(|item| lhs_context_items.contains(&item.label))
                .collect();
        }
        vec![]
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
            COMPLETION_ITEMS_COLLECTION
                .get()
                .expect("Could not get completion items collection for macpack packages")
                .get(&DocumentationKind::Macpack)
                .expect("Could not get macpack package completion items")
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
            COMPLETION_ITEMS_COLLECTION
                .get()
                .expect("Could not get completion items collection for feature names")
                .get(&DocumentationKind::Macpack)
                .expect("Could not get feature name completion items")
                .clone()
        } else {
            Vec::new()
        }
    }
}
