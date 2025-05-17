use crate::path::diff_paths;
use codespan::{File, Position};
use lazy_static::lazy_static;
use parser::{Ast, Instructions, ParseError, Token, TokenizerError};
use std::path::Path;
use std::str::FromStr;
use tower_lsp_server::lsp_types::Uri;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(u32);

impl FileId {
    const OFFSET: u32 = 1;

    fn new(index: usize) -> FileId {
        FileId(index as u32 + Self::OFFSET)
    }

    fn get(self) -> usize {
        (self.0 - Self::OFFSET) as usize
    }
}

pub struct CacheFile {
    pub file: File,
    pub tokens: Vec<Token>,
    pub ast: Ast,
    complete: bool,
}

impl CacheFile {
    pub fn new(file: File) -> CacheFile {
        CacheFile {
            file,
            tokens: Vec::new(),
            ast: Ast::new(),
            complete: false,
        }
    }
}

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

type Result<T> = std::result::Result<T, IndexError>;

lazy_static! {
    static ref INSTRUCTIONS: Instructions = Instructions::load();
}

impl Files {
    pub fn new() -> Self {
        Self { files: vec![] }
    }

    pub fn add(&mut self, uri: Uri, contents: String) -> FileId {
        let file_id = FileId::new(self.files.len());
        self.files
            .push(CacheFile::new(File::new(uri.as_str(), contents)));
        file_id
    }

    pub fn get(&self, id: FileId) -> &File {
        &self.files[id.get()].file
    }

    pub fn get_mut(&mut self, id: FileId) -> &mut File {
        &mut self.files[id.get()].file
    }

    pub fn get_uri(&self, id: FileId) -> Uri {
        Uri::from_str(self.get(id).name.as_str()).unwrap()
    }

    pub fn source(&self, id: FileId) -> &String {
        &self.get(id).source
    }

    pub fn ast(&self, id: FileId) -> &Ast {
        &self.files[id.get()].ast
    }

    pub fn line_tokens(&self, id: FileId, position: Position) -> Vec<Token> {
        let line_span = self.get(id).get_line(position.line).unwrap();
        let tokens = &self.files[id.get()].tokens;

        tokens
            .iter()
            .filter(|token| token.span.start >= line_span.start && token.span.end <= line_span.end)
            .cloned()
            .collect::<Vec<Token>>()
    }

    pub fn update(&mut self, id: FileId, source: String) {
        // tracing::info!("{}", source);
        self.get_mut(id).update(source)
    }

    pub fn show_instructions(&self, id: FileId, position: Position) -> bool {
        let tokens = self.line_tokens(id, position);
        let offset = self.get(id).position_to_byte_index(position).unwrap();
        tokens.is_empty() || tokens[0].span.end >= offset // Makes a naive guess at whether the current line contains an instruction. Doesn't work on lines with labels
    }

    pub fn index(&mut self, id: FileId) -> Result<()> {
        match parser::Tokenizer::new(self.source(id), &INSTRUCTIONS).parse() {
            Ok(tokens) => {
                self.files[id.get()].tokens = tokens;

                match parser::Parser::new(&self.files[id.get()].tokens).parse() {
                    Ok(ast) => {
                        self.files[id.get()].ast = ast;
                    }
                    Err(err) => {
                        return Err(IndexError::ParseError(err));
                    }
                }

                Ok(())
            }
            Err(err) => Err(IndexError::TokenizerError(err)),
        }
    }
}
