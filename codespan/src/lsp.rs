impl From<crate::Range> for lsp_types::Range {
    fn from(range: crate::Range) -> lsp_types::Range {
        lsp_types::Range {
            start: range.start.into(),
            end: range.end.into(),
        }
    }
}

impl From<lsp_types::Range> for crate::Range {
    fn from(range: lsp_types::Range) -> crate::Range {
        crate::Range {
            start: range.start.into(),
            end: range.end.into(),
        }
    }
}

impl From<crate::Position> for lsp_types::Position {
    fn from(pos: crate::Position) -> lsp_types::Position {
        lsp_types::Position {
            line: pos.line as u32,
            character: pos.character as u32,
        }
    }
}

impl From<lsp_types::Position> for crate::Position {
    fn from(pos: lsp_types::Position) -> crate::Position {
        crate::Position {
            line: pos.line as usize,
            character: pos.character as usize,
        }
    }
}
