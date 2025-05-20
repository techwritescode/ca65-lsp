use codespan::Span;
use parser::{Statement, StatementKind};
use std::collections::HashMap;

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
    pub symtab: HashMap<String, Span>,
}

impl ScopeAnalyzer {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self {
            statements,
            stack: Vec::new(),
            symtab: HashMap::new(),
        }
    }

    pub fn analyze(&mut self) -> Vec<Scope> {
        let statements = self.statements.clone();
        let scopes: Vec<Scope> = statements.iter().cloned().flat_map(|statement| self.parse_statement(statement)).collect();

        scopes
    }

    fn parse_statement(&mut self, statement: Statement) -> Option<Scope> {
        match statement.kind {
            StatementKind::Scope(Some(name), statements) => {
                self.symtab.insert([&self.stack[..], &[name.lexeme.clone()]].concat().join("::").to_string(), name.span);
                self.stack.push(name.lexeme.clone());
                let scopes = statements.iter().cloned().flat_map(|stmt| self.parse_statement(stmt)).collect();
                self.stack.pop();

                Some(Scope {
                    name: name.lexeme,
                    children: scopes,
                    span: statement.span,
                })
            }
            StatementKind::ConstantAssign(ca) => {
                self.symtab.insert([&self.stack[..], &[ca.name.lexeme]].concat().join("::").to_string(), ca.span);
                None
            }
            StatementKind::Procedure(name, statements) => {
                self.symtab.insert([&self.stack[..], &[name.lexeme.clone()]].concat().join("::").to_string(), name.span);

                self.stack.push(name.lexeme.clone());
                let scopes = statements.iter().cloned().flat_map(|stmt| self.parse_statement(stmt)).collect();
                self.stack.pop();

                Some(Scope {
                    name: name.lexeme,
                    children: scopes,
                    span: statement.span,
                })
            }
            _ => None
        }
    }

    pub fn search(scopes: Vec<Scope>, index: usize) -> Vec<String> {
        let scope = scopes.iter().find_map(|scope| scope.find_inner_scope(index)).unwrap_or(vec![]);
        let scope_names: Vec<String> = scope.iter().map(|scope| scope.name.clone()).collect();

        [&["".to_owned()], &scope_names[..]].concat()
    }
}