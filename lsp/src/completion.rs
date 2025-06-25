use crate::documentation::{DocumentationKind, COMPLETION_ITEMS_COLLECTION};
use crate::include_resolver::IncludeResolver;
use crate::{data::symbol::SymbolType, state::State};
use analysis::ScopeAnalyzer;
use codespan::FileId;
use codespan::Position;
use parser::TokenType;
use tower_lsp_server::lsp_types::{CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionTextEdit, Range, InsertReplaceEdit};

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
        let show_instructions = state.files.show_instructions(id, position); // Makes a naive guess at whether the current line contains an instruction. Doesn't work on lines with labels
        let byte_position = file.file.position_to_byte_index(position).unwrap_or(0);
        let scope = ScopeAnalyzer::search(&file.scopes, byte_position);

        let word_at_position = file.file.get_word_at_position(position).unwrap_or("");
        let has_namespace = word_at_position.contains(":");
        let mut resolved = IncludeResolver::new();
        resolved.resolve_include_tree(&state.files, &state.sources, id);

        resolved
            .symbols
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

                    // TODO: Add back once scopes and procs are sorted out
                    // let postfix = if matches!(symbol.sym_type, SymbolType::Scope) {
                    //     "::"
                    // } else {
                    //     ""
                    // };
                    let postfix = "";

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
        state: &State,
        id: FileId,
        position: Position,
    ) -> Vec<CompletionItem> {
        let curr_word = state
            .files
            .get(id)
            .file
            .get_word_at_position(position)
            .expect("Could not get word at position in completion provider");

        let insert_range = Range {
            start: tower_lsp_server::lsp_types::Position {
                line: position.line as u32,
                character: (position.character - curr_word.len()) as u32
            },
            end: tower_lsp_server::lsp_types::Position {
                line: position.line as u32,
                character: position.character as u32
            },
        };
        
        COMPLETION_ITEMS_COLLECTION
            .get()
            .expect("Could not get completion items collection for ca65 dot operators")
            .get(&DocumentationKind::Ca65DotOperator)
            .expect("Could not get ca65 dot operator completion items")
            .iter()
            .map(|item| {
                let mut new_item = item.clone();
                new_item.text_edit = Some(CompletionTextEdit::InsertAndReplace(InsertReplaceEdit {
                    new_text: item.insert_text.as_ref().expect("ca65 dot operator completion item did not have insert_text").clone(),
                    insert: insert_range,
                    replace: insert_range,
                }));
                new_item
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
        let curr_word = state
            .files
            .get(id)
            .file
            .get_word_at_position(position)
            .expect("Could not get word at position in completion provider");

        let insert_range = Range {
            start: tower_lsp_server::lsp_types::Position {
                line: position.line as u32,
                character: (position.character - curr_word.len()) as u32
            },
            end: tower_lsp_server::lsp_types::Position {
                line: position.line as u32,
                character: position.character as u32
            },
        };

        COMPLETION_ITEMS_COLLECTION
            .get()
            .expect("Could not get completion items collection for ca65 keywords")
            .get(&DocumentationKind::Ca65Keyword)
            .expect("Could not get ca65 keyword completion items")
            .iter()
            .map(|item| {
                let mut new_item = item.clone();
                new_item.text_edit = Some(CompletionTextEdit::InsertAndReplace(InsertReplaceEdit {
                    new_text: item.insert_text.as_ref().expect("ca65 keyword completion item did not have insert_text").clone(),
                    insert: insert_range,
                    replace: insert_range,
                }));
                new_item
            })
            .collect()
        
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
