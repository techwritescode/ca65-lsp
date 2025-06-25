use crate::cache_file::CacheFile;
use crate::data::path::diff_paths;
use codespan::{File, FileId, Position};
use lazy_static::lazy_static;
use parser::{Instructions, ParseError, Token, TokenizerError};
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

    pub fn show_instructions(&self, id: FileId, position: Position) -> bool {
        let tokens = self.line_tokens(id, position);
        let offset = self.get(id).file.position_to_byte_index(position).unwrap();
        tokens.is_empty() || tokens[0].span.end >= offset // Makes a naive guess at whether the current line contains an instruction. Doesn't work on lines with labels
    }
}
