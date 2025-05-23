use crate::visitor::ASTVisitor;
use codespan::Span;
use parser::{Ast, Statement, Token};

#[derive(Debug)]
pub struct IdentifierAccess {
    pub name: String,
    pub span: Span,
    pub scope: Vec<String>
}

pub struct SymbolResolver {
    identifiers: Vec<IdentifierAccess>,
    scope_stack: Vec<String>,
}

impl SymbolResolver {
    pub fn find_identifiers(ast: Ast) -> Vec<IdentifierAccess> {
        let mut slf = SymbolResolver {
            identifiers: Vec::new(),
            scope_stack: Vec::new(),
        };
        for statement in ast.iter() {
            slf.visit_statement(statement);
        }
        slf.identifiers
    }
}

impl ASTVisitor for SymbolResolver {
    fn visit_scope(&mut self, name: &Option<Token>, statements: &[Statement], _span: Span) {
        if let Some(name) = name {
            self.scope_stack.push(name.to_string());
        }
        
        for statement in statements {
            self.visit_statement(statement);
        }
        
        if name.is_some() {
            self.scope_stack.pop();
        }
    }
    fn visit_procedure(&mut self, name: &Token, statements: &[Statement], _span: Span) {
        self.scope_stack.push(name.to_string());

        for statement in statements {
            self.visit_statement(statement);
        }

        self.scope_stack.pop();
    }
    fn visit_identifier(&mut self, ident: &str, span: Span) {
        let scope = self.scope_stack[..].to_vec();
        self.identifiers.push(IdentifierAccess{
            name: ident.to_owned(),
            span,
            scope,
        });
    }
}
