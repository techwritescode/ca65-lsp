use codespan::Span;
use parser::{Statement, StatementKind, Token};
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub enum Symbol {
    Scope { name: Token },
    Label { name: Token },
    Macro { name: Token, parameters: Vec<Token> },
    Constant { name: Token },
    Parameter { name: Token }, // Disabled for now, need to track macro scopes
}

impl Symbol {
    pub fn get_span(&self) -> Span {
        let name = match self {
            Symbol::Scope { name, .. } => name,
            Symbol::Label { name, .. } => name,
            Symbol::Macro { name, .. } => name,
            Symbol::Constant { name, .. } => name,
            Symbol::Parameter { name, .. } => name,
        };

        name.span
    }

    pub fn get_name(&self) -> String {
        match self {
            Symbol::Scope { name } => name.lexeme.clone(),
            Symbol::Label { name, .. } => name.lexeme.clone(),
            Symbol::Macro { name, .. } => name.lexeme.clone(),
            Symbol::Constant { name, .. } => name.lexeme.clone(),
            Symbol::Parameter { name, .. } => name.lexeme.clone(),
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            Symbol::Scope { name } => name.lexeme.clone(),
            Symbol::Label { name, .. } => format!("{}:", name.lexeme),
            Symbol::Macro {
                name, parameters, ..
            } => Self::format_parameters(name, parameters),
            Symbol::Constant { name, .. } => name.lexeme.clone(),
            Symbol::Parameter { name, .. } => name.lexeme.clone(),
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
}

#[derive(Clone, Debug)]
pub struct Scope {
    pub name: String,
    pub span: Span,
    pub children: Vec<Scope>,
}

impl Scope {
    fn find_inner_scope(&self, index: usize) -> Option<Vec<Scope>> {
        if index < self.span.start || index >= self.span.end {
            return None;
        }

        for child in &self.children {
            if let Some(inner_scope) = child.find_inner_scope(index) {
                return Some([&[self.clone()], &inner_scope[..]].concat());
            }
        }

        Some(vec![self.clone()])
    }
}

pub struct ScopeAnalyzer {
    pub statements: Vec<Statement>,
    pub stack: Vec<String>,
    pub symtab: HashMap<String, Symbol>,
}

impl ScopeAnalyzer {
    pub fn remove_denominator(scope: &[String], fqn: String) -> String {
        let target: Vec<String> = fqn.split("::").map(|s| s.to_string()).collect();

        for (i, (a, b)) in target.iter().zip(scope).enumerate() {
            if *a != *b {
                return target[i..].join("::");
            }
        }

        target.last().expect("Empty Symbol").clone()
    }
}

impl ScopeAnalyzer {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self {
            statements,
            stack: Vec::new(),
            symtab: HashMap::new(),
        }
    }

    pub fn analyze(&mut self) -> (Vec<Scope>, HashMap<String, Symbol>) {
        let statements = self.statements.clone();
        let scopes: Vec<Scope> = statements
            .iter()
            .cloned()
            .flat_map(|statement| self.parse_statement(statement))
            .collect();

        (scopes, self.symtab.clone())
    }

    fn parse_statement(&mut self, statement: Statement) -> Option<Scope> {
        match statement.kind {
            StatementKind::Scope(Some(name), statements) => {
                let lexeme = name.lexeme.clone();
                self.symtab
                    .insert(self.format_name(&name), Symbol::Scope { name });
                self.stack.push(lexeme.clone());
                let scopes = statements
                    .iter()
                    .cloned()
                    .flat_map(|stmt| self.parse_statement(stmt))
                    .collect();
                self.stack.pop();

                Some(Scope {
                    name: lexeme,
                    children: scopes,
                    span: statement.span,
                })
            }
            StatementKind::ConstantAssign(ca) => {
                self.symtab.insert(
                    self.format_name(&ca.name),
                    Symbol::Constant { name: ca.name },
                );
                None
            }
            StatementKind::Procedure(name, statements) => {
                let lexeme = name.lexeme.clone();
                self.symtab
                    .insert(self.format_name(&name), Symbol::Label { name });

                self.stack.push(lexeme.clone());
                let scopes = statements
                    .iter()
                    .cloned()
                    .flat_map(|stmt| self.parse_statement(stmt))
                    .collect();
                self.stack.pop();

                Some(Scope {
                    name: lexeme,
                    children: scopes,
                    span: statement.span,
                })
            }
            StatementKind::MacroDefinition(name, parameters, _) => {
                self.symtab
                    .insert(self.format_name(&name), Symbol::Macro { name, parameters });

                None
            }
            _ => None,
        }
    }

    pub fn search(scopes: &[Scope], index: usize) -> Vec<String> {
        let scope = scopes
            .iter()
            .find_map(|scope| scope.find_inner_scope(index))
            .unwrap_or(vec![]);
        let scope_names: Vec<String> = scope.iter().map(|scope| scope.name.clone()).collect();

        [&["".to_owned()], &scope_names[..]].concat()
    }

    #[inline]
    fn format_name(&self, name: &Token) -> String {
        [&["".to_owned()], &self.stack[..], &[name.lexeme.clone()]]
            .concat()
            .join("::")
            .to_string()
    }
}
