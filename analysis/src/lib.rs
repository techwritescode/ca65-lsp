use std::fmt::Write;
mod arena;

use codespan::Span;
use parser::{Statement, StatementKind, Token};
use std::collections::HashMap;

pub struct ScopeAnalyzer {
    lines: Vec<Statement>,
    symtab: HashMap<String, Scope>,
}

// TODO: these definitions are murky, this is more of a symbol analyzer
pub enum ScopeKind {
    Label,
    Macro,
    Constant,
    Parameter
}

pub struct Scope {
    pub title: String,
    pub description: String,
    pub span: Span,
    pub kind: ScopeKind,
}

impl ScopeAnalyzer {
    pub fn new(lines: Vec<Statement>) -> ScopeAnalyzer {
        ScopeAnalyzer {
            lines,
            symtab: HashMap::new(),
        }
    }

    pub fn parse(mut self) -> HashMap<String, Scope> {
        for line in self.lines.iter() {
            self.symtab.extend(Self::parse_line(line));
        }

        self.symtab
    }

    fn parse_line(line: &Statement) -> Vec<(String, Scope)> {
        match &line.kind {
            StatementKind::ConstantAssign(assign) => {
                vec![(
                    assign.name.lexeme.clone(),
                    Scope {
                        title: assign.name.lexeme.clone(),
                        description: assign.name.lexeme.clone(), // TODO: Add expression flattening/preview? Might need method on ast nodes to print formatted output
                        span: assign.name.span,
                        kind: ScopeKind::Constant,
                    },
                )]
            }
            StatementKind::Label(label) => {
                vec![(
                    label.lexeme.clone(),
                    Scope {
                        title: label.lexeme.clone(),
                        description: format!("{}:", label.lexeme.clone()),
                        span: label.span,
                        kind: ScopeKind::Label,
                    },
                )]
            }
            StatementKind::Procedure(name, instructions) => {
                let mut symbols = vec![(
                    name.lexeme.clone(),
                    Scope {
                        title: name.lexeme.clone(),
                        description: format!("{}:", name.lexeme.clone()),
                        span: name.span,
                        kind: ScopeKind::Label,
                    },
                )];

                for line in instructions.iter() {
                    symbols.extend(Self::parse_line(line));
                }

                symbols
            }
            StatementKind::MacroDefinition(name, parameters, _) => {
                let mut symbols = vec![(
                    name.lexeme.clone(),
                    Scope {
                        title: name.lexeme.clone(),
                        description: format_parameters(name, parameters),
                        span: name.span,
                        kind: ScopeKind::Macro,
                    },
                )];

                for line in parameters.iter() {
                    symbols.push((
                        line.lexeme.clone(),
                        Scope {
                            title: line.lexeme.clone(),
                            description: line.lexeme.clone(),
                            span: line.span,
                            kind: ScopeKind::Parameter,
                        },
                    ));
                }

                symbols
            }
            _ => vec![],
        }
    }
}

fn format_parameters(name: &Token, parameters: &[Token]) -> String {
    let mut output = String::new();

    write!(&mut output, ".macro {} ", name.lexeme).unwrap();

    for (i, token) in parameters.iter().enumerate() {
        match i {
            0 => write!(output, "{}", token.lexeme),
            _ => write!(output, ", {}", token.lexeme),
        }
        .unwrap()
    }

    output
}
