use core::panic;

use tower_lsp::lsp_types::{Position, Range, Url};

#[derive(Debug, PartialEq)]
pub enum LocationError {
    OutOfBounds { given: u32, span: Span },
    InvalidCharBoundary { given: u32 },
}

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

pub struct File {
    pub name: String,
    pub source: String,
    line_starts: Vec<u32>,
}

fn get_line_starts(contents: &str) -> Vec<u32> {
    std::iter::once(0)
        .chain(contents.match_indices('\n').map(|(i, _)| i as u32 + 1))
        .collect()
}

impl File {
    fn new(name: impl Into<String>, source: String) -> Self {
        let line_starts = get_line_starts(source.as_ref());
        Self {
            name: name.into(),
            source,
            line_starts,
        }
    }

    fn update(&mut self, source: String) {
        let line_starts = get_line_starts(source.as_ref());
        self.source = source;
        self.line_starts = line_starts;
    }

    fn line_start(&self, line_index: usize) -> anyhow::Result<u32> {
        use std::cmp::Ordering;

        match line_index.cmp(&self.last_line_index()) {
            Ordering::Less => Ok(self.line_starts[line_index]),
            Ordering::Equal => Ok(self.source.len() as u32),
            Ordering::Greater => {
                tracing::error!("Line index out of bounds");
                panic!("Line index out of bounds")
            }
        }
    }

    fn last_line_index(&self) -> usize {
        self.line_starts.len()
    }

    fn line_span(&self, line_index: usize) -> anyhow::Result<Span> {
        let line_start = self.line_start(line_index)?;
        let next_line_start = self.line_start(line_index + 1)?;

        Ok(Span::new(line_start, next_line_start))
    }

    fn location(&self, byte_index: u32) -> Result<Position, LocationError> {
        match self.line_starts.binary_search(&byte_index) {
            Ok(line) => Ok(Position {
                line: line as u32,
                character: 0,
            }),
            Err(next_line) => {
                let line_index = next_line - 1;
                let line_start_index =
                    self.line_start(line_index)
                        .map_err(|_| LocationError::OutOfBounds {
                            given: byte_index,
                            span: self.source_span(),
                        })?;

                let line_src = self
                    .source
                    .get((line_start_index as usize)..(byte_index as usize))
                    .ok_or_else(|| {
                        let given = byte_index;
                        if given >= self.source_span().end() {
                            let span = self.source_span();
                            LocationError::OutOfBounds { given, span }
                        } else {
                            LocationError::InvalidCharBoundary { given }
                        }
                    })?;

                Ok(Position::new(line_index as u32, line_src.len() as u32))
            }
        }
    }

    fn source_slice(&self, span: Span) -> anyhow::Result<&str> {
        let start = span.start as usize;
        let end = span.end as usize;

        self.source.get(start..end).ok_or_else(|| {
            tracing::error!("Failed to create source span");
            panic!()
        })
    }

    fn source_span(&self) -> Span {
        Span::new(0, self.source.len() as u32)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    start: u32,
    end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    fn start(self) -> u32 {
        self.start
    }

    fn end(self) -> u32 {
        self.end
    }
}

pub struct Files {
    files: Vec<File>,
}

impl Files {
    pub fn new() -> Self {
        Self { files: vec![] }
    }

    pub fn add(&mut self, uri: Url, contents: String) -> FileId {
        let file_id = FileId::new(self.files.len());
        self.files.push(File::new(uri, contents));
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
        tracing::info!("{}", source);
        self.get_mut(id).update(source)
    }
}

fn position_to_byte_index(files: &Files, id: FileId, position: Position) -> anyhow::Result<usize> {
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
    byte_index: u32,
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

    Ok(file.line_span(line_index)?)
}

pub fn get_line_source(files: &Files, id: FileId, span: Span) -> anyhow::Result<&str> {
    let file = files.get(id);

    Ok(file.source_slice(span)?)
}

pub fn get_word_at_position(files: &Files, id: FileId, position: Position) -> anyhow::Result<&str> {
    let file = files.get(id);
    let line = file.source_slice(file.line_span(position.line as usize)?)?;
    let range = find_word_at_pos(line, position.character as usize);

    Ok(line.get(range.0..range.1).unwrap())
}

fn find_word_at_pos(line: &str, col: usize) -> (usize, usize) {
    let line_ = format!("{} ", line);
    let is_ident_char = |c: char| c.is_alphanumeric() || c == '_' || c == '.';

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
