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

pub fn position_to_byte_index(
    files: &Files,
    id: FileId,
    position: Position,
) -> anyhow::Result<usize> {
    let line_span = files.get(id).line_span(position.line as usize)?;
    let byte_offset = position.character as usize;

    Ok(line_span.start() as usize + byte_offset)
}

pub fn range_to_byte_span(
    files: &Files,
    id: FileId,
    range: &Range,
) -> anyhow::Result<std::ops::Range<usize>> {
    Ok(position_to_byte_index(files, id, range.start)?
        ..position_to_byte_index(files, id, range.end)?)
}

pub fn byte_index_to_position(
    files: &Files,
    id: FileId,
    byte_index: usize,
) -> Result<Position, LocationError> {
    let file = files.get(id);
    let location = file.location(byte_index)?;

    Ok(location)
}

pub fn byte_span_to_range(files: &Files, id: FileId, span: Span) -> Result<Range, LocationError> {
    Ok(Range {
        start: byte_index_to_position(files, id, span.start())?,
        end: byte_index_to_position(files, id, span.end())?,
    })
}

pub fn get_line(files: &Files, id: FileId, line_index: usize) -> anyhow::Result<Span> {
    let file = files.get(id);

    file.line_span(line_index)
}

#[allow(dead_code)]
pub fn get_line_source(files: &Files, id: FileId, span: Span) -> anyhow::Result<&str> {
    let file = files.get(id);

    file.source_slice(span)
}

pub fn get_word_at_position(files: &Files, id: FileId, position: Position) -> anyhow::Result<&str> {
    let file = files.get(id);
    let line = file.source_slice(file.line_span(position.line as usize)?)?;
    let range = find_word_at_pos(line, position.character as usize);

    Ok(line.get(range.0..range.1).unwrap())
}

fn find_word_at_pos(line: &str, col: usize) -> (usize, usize) {
    let line_ = format!("{} ", line);
    let is_ident_char = |c: char| c.is_alphanumeric() || c == '_' || c == '@' || c == '.';

    let start = line_
        .chars()
        .enumerate()
        .take(col)
        .filter(|&(_, c)| !is_ident_char(c))
        .last()
        .map(|(i, _)| i + 1)
        .unwrap_or(0);

    let mut end = line_
        .chars()
        .enumerate()
        .skip(col)
        .filter(|&(_, c)| !is_ident_char(c));

    let end = end.next();
    (start, end.map(|(i, _)| i).unwrap_or(col))
}
