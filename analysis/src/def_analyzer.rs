use codespan::Span;
use parser::{Statement, StatementKind, Token};
use std::collections::HashMap;
use std::fmt::Write;

pub struct DefAnalyzer {
    statements: Vec<Statement>,
    symtab: HashMap<String, SymDef>,
}

// TODO: these definitions are murky, this is more of a symbol analyzer
pub enum ScopeKind {
    Label,
    Macro,
    Constant,
    Parameter,
}

pub struct SymDef {
    pub title: String,
    pub description: String,
    pub span: Span,
    pub kind: ScopeKind,
}

impl DefAnalyzer {
    pub fn new(statements: Vec<Statement>) -> DefAnalyzer {
        DefAnalyzer {
            statements,
            symtab: HashMap::new(),
        }
    }

    pub fn parse(mut self) -> HashMap<String, SymDef> {
        for line in self.statements.iter() {
            self.symtab.extend(Self::parse_statement(line));
        }

        self.symtab
    }

    fn parse_statement(line: &Statement) -> Vec<(String, SymDef)> {
        match &line.kind {
            StatementKind::ConstantAssign(assign) => {
                vec![(
                    assign.name.lexeme.clone(),
                    SymDef {
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
                    SymDef {
                        title: label.lexeme.clone(),
                        description: format!("{}:", label.lexeme.clone()),
                        span: label.span,
                        kind: ScopeKind::Label,
                    },
                )]
            }
            StatementKind::Procedure(name, _far, instructions) => {
                let mut symbols = vec![(
                    name.lexeme.clone(),
                    SymDef {
                        title: name.lexeme.clone(),
                        description: format!("{}:", name.lexeme.clone()),
                        span: name.span,
                        kind: ScopeKind::Label,
                    },
                )];

                for line in instructions.iter() {
                    symbols.extend(Self::parse_statement(line));
                }

                symbols
            }
            StatementKind::MacroDefinition(name, parameters, _) => {
                let mut symbols = vec![(
                    name.lexeme.clone(),
                    SymDef {
                        title: name.lexeme.clone(),
                        description: format_parameters(name, parameters),
                        span: name.span,
                        kind: ScopeKind::Macro,
                    },
                )];

                for line in parameters.iter() {
                    symbols.push((
                        line.lexeme.clone(),
                        SymDef {
                            title: line.lexeme.clone(),
                            description: line.lexeme.clone(),
                            span: line.span,
                            kind: ScopeKind::Parameter,
                        },
                    ));
                }

                symbols
            }
            StatementKind::Instruction(_instruction) => {
                vec![]
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
