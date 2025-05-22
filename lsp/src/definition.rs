use crate::codespan::{FileId, Files};
use crate::symbol_cache::{symbol_cache_fetch, Symbol};
use codespan::{FileError, Position};
use std::cmp::Ordering;

#[derive(Debug, Copy, Clone)]
pub struct Definition;

impl Definition {
    pub fn get_definition_position(
        &self,
        files: &Files,
        id: FileId,
        position: Position,
    ) -> Result<Option<Vec<Symbol>>, FileError> {
        let word = files.get(id).get_word_at_position(position)?;

        let mut definitions = symbol_cache_fetch(word.to_string());

        definitions.sort_by(|sym, _| {
            if sym.file_id == id {
                return Ordering::Less;
            }
            Ordering::Equal
        });

        Ok(Some(definitions))
    }
}
