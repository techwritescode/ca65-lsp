use crate::codespan::FileId;
use crate::state::State;
use crate::symbol_cache::{symbol_cache_fetch, Symbol};
use analysis::ScopeAnalyzer;
use codespan::{FileError, Position, Span};
use std::cmp::Ordering;

#[derive(Debug, Copy, Clone)]
pub struct Definition;

pub fn find_word_at_pos(line: &str, col: usize) -> Span {
    let line_ = format!("{line} ");
    let is_ident_char = |c: char| c.is_alphanumeric() || c == '_';

    let start = line_
        .chars()
        .enumerate()
        .take(col)
        .filter(|&(_, c)| !is_ident_char(c))
        .last()
        .map(|(i, _)| i + 1)
        .unwrap_or(0);

    let end = line_
        .chars()
        .enumerate()
        .skip(col)
        .find(|&(_, c)| !is_ident_char(c))
        .map(|(i, _)| i)
        .unwrap_or(col);

    Span::new(start, end)
}

fn get_sub_identifier(identifier: &str, index: usize, span: Span) -> Span {
    let index = index - span.start;
    find_word_at_pos(identifier, index)
}

impl Definition {
    pub fn get_definition_position(
        &self,
        state: &State,
        id: FileId,
        position: Position,
    ) -> Result<Option<(Vec<Symbol>, Span)>, FileError> {
        let (word, span) = state.files.get(id).get_word_span_at_position(position)?;
        let index = state.files.get(id).position_to_byte_index(position)?;
        let scopes = state.scopes.get(&state.files.get_uri(id)).unwrap();
        let current_scopes = ScopeAnalyzer::search(scopes, index);

        let new_span = get_sub_identifier(word, index, span);
        let slice = &word[0..new_span.end];

        let mut definitions = vec![];

        if slice.starts_with("::") {
            definitions.extend(symbol_cache_fetch(slice.to_string()));
        } else {
            for (idx, _scope) in current_scopes.iter().rev().enumerate() {
                let fqn = [&current_scopes[0..=idx], &[slice.to_string()]]
                    .concat()
                    .join("::");
                let defs = symbol_cache_fetch(fqn.clone());
                if !defs.is_empty() {
                    definitions.extend(defs);
                    break;
                }
            }
        }

        definitions.sort_by(|sym, _| {
            if sym.file_id == id {
                return Ordering::Less;
            }
            Ordering::Equal
        });

        Ok(Some((
            definitions,
            Span::new(span.start + new_span.start, span.start + new_span.end),
        )))
    }
}
