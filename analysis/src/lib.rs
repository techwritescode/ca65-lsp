mod arena;

use codespan::Span;
use parser::{Line, LineKind};
use std::collections::HashMap;

pub struct ScopeAnalyzer {
    lines: Vec<Line>,
    symtab: HashMap<String, Span>,
}

impl ScopeAnalyzer {
    pub fn new(lines: Vec<Line>) -> ScopeAnalyzer {
        ScopeAnalyzer {
            lines,
            symtab: HashMap::new(),
        }
    }

    pub fn parse(mut self) -> HashMap<String, Span> {
        for line in self.lines.iter() {
            self.symtab.extend(Self::parse_line(line));
        }

        self.symtab
    }

    fn parse_line(line: &Line) -> Vec<(String, Span)> {
        match &line.kind {
            LineKind::Label(label) => {
                vec![(label.lexeme.clone(), label.span)]
            }
            LineKind::Procedure(name, instructions) => {
                let mut symbols = vec![(name.lexeme.clone(), name.span)];
                
                for line in instructions.iter() {
                    symbols.extend(Self::parse_line(line));
                }
                
                symbols
            }
            _ => vec![],
        }
    }
}
