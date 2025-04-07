use crate::position::Position;
use crate::{Range, Span};
use core::panic;
use std::fmt::{Display, Formatter};

#[allow(dead_code)]
pub struct File {
    pub name: String,
    pub source: String,
    line_starts: Vec<usize>,
}

fn get_line_starts(contents: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(contents.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}

#[derive(Debug)]
pub enum FileError {
    OutOfBounds { given: usize, span: Span },
    InvalidCharBoundary { given: usize },
}

impl Display for FileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "File Error")
    }
}

type Result<T> = std::result::Result<T, FileError>;

impl File {
    pub fn new(name: impl Into<String>, source: String) -> Self {
        let line_starts = get_line_starts(source.as_ref());
        Self {
            name: name.into(),
            source,
            line_starts,
        }
    }

    pub fn update(&mut self, source: String) {
        let line_starts = get_line_starts(source.as_ref());
        self.source = source;
        self.line_starts = line_starts;
    }

    fn line_start(&self, line_index: usize) -> Result<usize> {
        use std::cmp::Ordering;

        match line_index.cmp(&self.last_line_index()) {
            Ordering::Less => Ok(self.line_starts[line_index]),
            Ordering::Equal => Ok(self.source.len()),
            Ordering::Greater => {
                // tracing::error!("Line index out of bounds");
                panic!("Line index out of bounds")
            }
        }
    }

    fn last_line_index(&self) -> usize {
        self.line_starts.len()
    }

    fn line_span(&self, line_index: usize) -> Result<Span> {
        let line_start = self.line_start(line_index)?;
        let next_line_start = self.line_start(line_index + 1)?;

        Ok(Span::new(line_start, next_line_start))
    }

    fn location(&self, byte_index: usize) -> Result<Position> {
        match self.line_starts.binary_search(&byte_index) {
            Ok(line) => Ok(Position { line, character: 0 }),
            Err(next_line) => {
                let line_index = next_line - 1;
                let line_start_index =
                    self.line_start(line_index)
                        .map_err(|_| FileError::OutOfBounds {
                            given: byte_index,
                            span: self.source_span(),
                        })?;

                let line_src = self
                    .source
                    .get(line_start_index..byte_index)
                    .ok_or_else(|| {
                        let given = byte_index;
                        if given >= self.source_span().end() {
                            let span = self.source_span();
                            FileError::OutOfBounds { given, span }
                        } else {
                            FileError::InvalidCharBoundary { given }
                        }
                    })?;

                Ok(Position::new(line_index, line_src.len()))
            }
        }
    }

    fn source_slice(&self, span: Span) -> Result<&str> {
        let start = span.start;
        let end = span.end;

        self.source.get(start..end).ok_or_else(|| {
            // tracing::error!("Failed to create source span");
            panic!()
        })
    }

    fn source_span(&self) -> Span {
        Span::new(0, self.source.len())
    }

    pub fn position_to_byte_index(&self, position: Position) -> Result<usize> {
        let line_span = &self.line_span(position.line)?;
        let byte_offset = position.character;

        Ok(line_span.start() + byte_offset)
    }

    pub fn range_to_byte_span(&self, range: &Range) -> Result<std::ops::Range<usize>> {
        Ok(self.position_to_byte_index(range.start)?..self.position_to_byte_index(range.end)?)
    }

    pub fn byte_index_to_position(&self, byte_index: usize) -> Result<Position> {
        let location = self.location(byte_index)?;

        Ok(location)
    }

    pub fn byte_span_to_range(&self, span: Span) -> Result<Range> {
        Ok(Range {
            start: self.byte_index_to_position(span.start())?,
            end: self.byte_index_to_position(span.end())?,
        })
    }

    pub fn get_line(&self, line_index: usize) -> Result<Span> {
        self.line_span(line_index)
    }

    #[allow(dead_code)]
    pub fn get_line_source(&self, span: Span) -> Result<&str> {
        self.source_slice(span)
    }

    pub fn get_word_at_position(&self, position: Position) -> Result<&str> {
        let line = self.source_slice(self.line_span(position.line)?)?;
        let range = find_word_at_pos(line, position.character);

        Ok(line.get(range.0..range.1).unwrap())
    }
}

pub fn find_word_at_pos(line: &str, col: usize) -> (usize, usize) {
    let line_ = format!("{} ", line);
    let is_ident_char = |c: char| c.is_alphanumeric() || c == '_' || c == '@';

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

    (start, end)
}
