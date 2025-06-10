use crate::cache_file::CacheFile;
use crate::data::path::diff_paths;
use codespan::{File, FileId, Position};
use lazy_static::lazy_static;
use parser::{Ast, Instructions, ParseError, Token, TokenType, TokenizerError};
use std::path::Path;
use std::str::FromStr;
use tower_lsp_server::lsp_types::Uri;

pub struct Files {
    files: Vec<CacheFile>,
}

impl Files {
    pub fn get_uri_relative(&self, id: FileId, root: FileId) -> Option<String> {
        let target_uri = self.get_uri(id);
        let relative_uri = self.get_uri(root);

        let path = diff_paths(
            Path::new(&target_uri.path().to_string()),
            Path::new(&relative_uri.path().to_string()).parent()?,
        )?;

        Some(path.to_string_lossy().to_string())
    }
}

pub enum IndexError {
    TokenizerError(TokenizerError),
    ParseError(ParseError),
}

lazy_static! {
    pub static ref INSTRUCTIONS: Instructions = Instructions::load();
}

impl Files {
    pub fn new() -> Self {
        Self { files: vec![] }
    }

    pub fn add(&mut self, uri: Uri, contents: String) -> FileId {
        let file_id = FileId::new(self.files.len());
        self.files
            .push(CacheFile::new(File::new(uri.as_str(), contents), file_id));
        file_id
    }

    pub fn get(&self, id: FileId) -> &CacheFile {
        &self.files[id.get()]
    }

    pub fn get_mut(&mut self, id: FileId) -> &mut CacheFile {
        &mut self.files[id.get()]
    }

    pub fn get_uri(&self, id: FileId) -> Uri {
        Uri::from_str(self.get(id).file.name.as_str()).unwrap()
    }

    pub fn source(&self, id: FileId) -> &String {
        &self.get(id).file.source
    }

    pub fn line_tokens(&self, id: FileId, position: Position) -> Vec<Token> {
        let line_span = self.get(id).file.get_line(position.line).unwrap();
        let tokens = &self.files[id.get()].tokens;

        tokens
            .iter()
            .filter(|token| token.span.start >= line_span.start && token.span.end <= line_span.end)
            .cloned()
            .collect::<Vec<Token>>()
    }

    pub fn update(&mut self, id: FileId, source: String) {
        // tracing::info!("{}", source);
        self.get_mut(id).file.update(source)
    }

    fn get_tokens_before_cursor(&self, id: FileId, position: Position) -> Vec<&Token> {
        let tokens = self.line_tokens(id, position);
        let offset = self.get(id).file.position_to_byte_index(position).unwrap();

        tokens.iter().filter(|tok| tok.span.end < offset).collect()
    }
    pub fn show_lhs_completions(&self, id: FileId, position: Position) -> bool {
        let tokens_before_cursor = self.get_tokens_before_cursor(id, position);

        tokens_before_cursor.is_empty()
            || (tokens_before_cursor.len() == 2
                && tokens_before_cursor[0].token_type == TokenType::Identifier
                && tokens_before_cursor[1].token_type == TokenType::Colon)
    }
    pub fn show_rhs_completions(&self, id: FileId, position: Position) -> bool {
        let tokens_before_cursor = self.get_tokens_before_cursor(id, position);

        !tokens_before_cursor.is_empty()
            && (tokens_before_cursor.last().is_some_and(|tok| {
                tok.token_type == TokenType::Instruction || tok.token_type == TokenType::Macro
            }))
    }
}
