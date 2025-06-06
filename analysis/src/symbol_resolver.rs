use crate::visitor::ASTVisitor;
use codespan::Span;
use parser::{Ast, EnumMember, Expression, ImportExport, Statement, StructMember, Token};

#[derive(Debug)]
pub struct IdentifierAccess {
    pub name: String,
    pub span: Span,
    pub scope: Vec<String>,
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
    fn visit_macro_definition(
        &mut self,
        name: &Token,
        _parameters: &[Token],
        statements: &[Statement],
        _span: Span,
    ) {
        self.scope_stack.push(name.to_string());

        // Skip type checking in macros for now. Might be good to add local label completion at some point, but ultimately we don't know the context the macro is invoked in yet

        for statement in statements {
            self.visit_statement(statement);
        }

        self.scope_stack.pop();
    }
    fn visit_enum(&mut self, name: &Option<Token>, _variants: &[EnumMember], _span: Span) {
        if let Some(name) = name {
            self.scope_stack.push(name.to_string());

            self.scope_stack.pop();
        }
    }
    fn visit_define(
        &mut self,
        ident: &Token,
        params: &Option<Vec<Token>>,
        expr: &Expression,
        _span: Span,
    ) {
        if params.is_some() {
            self.scope_stack.push(ident.to_string());
            self.visit_expression(expr);
            self.scope_stack.pop();
        }
    }
    fn visit_struct(&mut self, name: &Token, _members: &[StructMember], _span: Span) {
        self.scope_stack.push(name.lexeme.clone());

        self.scope_stack.pop();
    }
    fn visit_repeat(
        &mut self,
        _max: &Expression,
        _incr: &Option<Token>,
        statements: &[Statement],
        _span: Span,
    ) {
        self.scope_stack.push("__repeat".to_owned());

        for statement in statements {
            self.visit_statement(statement);
        }

        self.scope_stack.pop();
    }
    fn visit_procedure(
        &mut self,
        name: &Token,
        _far: &bool,
        statements: &[Statement],
        _span: Span,
    ) {
        self.scope_stack.push(name.to_string());

        for statement in statements {
            self.visit_statement(statement);
        }

        self.scope_stack.pop();
    }
    fn visit_identifier(&mut self, ident: &str, span: Span) {
        let scope = self.scope_stack[..].to_vec();
        self.identifiers.push(IdentifierAccess {
            name: ident.to_owned(),
            span,
            scope,
        });
    }

    fn visit_export(&mut self, exports: &[ImportExport], _zero_page: &bool, _span: Span) {
        let scope = self.scope_stack[..].to_vec();
        for export in exports {
            if export.value.is_none() {
                self.identifiers.push(IdentifierAccess {
                    name: export.name.to_string(),
                    span: export.name.span,
                    scope: scope.clone(),
                })
            }
        }
    }
}
