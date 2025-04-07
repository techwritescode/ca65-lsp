use core::panic;
use std::fmt::{Display, Formatter};
use tower_lsp_server::lsp_types::{Position, Range, Uri};
use codespan::File;

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

pub struct Files {
    files: Vec<File>,
}

impl Files {
    pub fn new() -> Self {
        Self { files: vec![] }
    }

    pub fn add(&mut self, uri: Uri, contents: String) -> FileId {
        let file_id = FileId::new(self.files.len());
        self.files.push(File::new(uri.as_str(), contents));
        file_id
    }

    pub fn get(&self, id: FileId) -> &File {
        &self.files[id.get()]
    }

    fn get_mut(&mut self, id: FileId) -> &mut File {
        &mut self.files[id.get()]
    }

    pub fn source(&self, id: FileId) -> &String {
        &self.get(id).source
    }

    pub fn update(&mut self, id: FileId, source: String) {
        // tracing::info!("{}", source);
        self.get_mut(id).update(source)
    }
}