use crate::analysis::visitor::ASTVisitor;
use crate::cache_file::Include;
use codespan::Span;
use parser::{
    Ast, ConstantAssign, EnumMember, Expression, ImportExport, Statement, StructMember, Token,
};
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
    pub name_span: Span,
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

impl PartialEq for Scope {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct ScopeAnalyzer {
    pub ast: Vec<Statement>,
    pub stack: Vec<Scope>,
    pub symtab: HashMap<String, Symbol>,
    pub includes: Vec<Include>,
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
    pub fn new(ast: Ast) -> Self {
        Self {
            ast,
            stack: vec![Scope {
                name: "Root".to_string(),
                name_span: Span::NONE,
                span: Span::NONE,
                children: vec![],
            }],
            includes: vec![],
            symtab: HashMap::new(),
        }
    }

    pub fn analyze(&mut self) -> (Vec<Scope>, HashMap<String, Symbol>, Vec<Include>) {
        for statement in self.ast.clone().iter() {
            self.visit_statement(statement);
        }

        // Get children of root node
        (
            self.stack[0].children.clone(),
            self.symtab.clone(),
            self.includes.clone(),
        )
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
        let stack: Vec<String> = self.stack[1..].iter().map(|s| s.name.clone()).collect();
        [&["".to_owned()], &stack[..], &[name.lexeme.clone()]]
            .concat()
            .join("::")
            .to_string()
    }

    fn push_scope(&mut self, name: &Token, span: Span) {
        self.stack.push(Scope {
            name: name.to_string(),
            name_span: name.span,
            children: vec![],
            span,
        });
    }
    fn pop_scope(&mut self) {
        if let Some(node) = self.stack.pop() {
            if let Some(parent) = self.stack.last_mut() {
                parent.children.push(node);
            }
        }
    }

    fn insert_symbol(&mut self, name: &Token, symbol: Symbol) {
        self.symtab.insert(self.format_name(name), symbol);
    }
}

impl ASTVisitor for ScopeAnalyzer {
    fn visit_scope(&mut self, name: &Option<Token>, statements: &[Statement], span: Span) {
        if let Some(name) = name {
            self.insert_symbol(name, Symbol::Scope { name: name.clone() });

            self.push_scope(name, span);

            for statement in statements {
                self.visit_statement(statement);
            }

            self.pop_scope();
        }
    }
    fn visit_constant_assign(&mut self, statement: &ConstantAssign, _span: Span) {
        self.insert_symbol(
            &statement.name,
            Symbol::Constant {
                name: statement.name.clone(),
            },
        );
        self.visit_expression(&statement.value);
    }
    fn visit_procedure(&mut self, name: &Token, _far: &bool, statements: &[Statement], span: Span) {
        self.insert_symbol(name, Symbol::Scope { name: name.clone() });

        self.push_scope(name, span);

        for statement in statements {
            self.visit_statement(statement);
        }

        self.pop_scope()
    }
    fn visit_macro_definition(
        &mut self,
        name: &Token,
        parameters: &[Token],
        statements: &[Statement],
        span: Span,
    ) {
        self.insert_symbol(name, Symbol::Scope { name: name.clone() });

        self.push_scope(name, span);

        for parameter in parameters.iter() {
            self.insert_symbol(
                parameter,
                Symbol::Scope {
                    name: parameter.clone(),
                },
            );
        }

        for statement in statements {
            self.visit_statement(statement);
        }

        self.pop_scope()
    }
    fn visit_define(
        &mut self,
        ident: &Token,
        params: &Option<Vec<Token>>,
        expr: &Expression,
        span: Span,
    ) {
        if let Some(params) = params {
            self.push_scope(ident, span);
            for param in params.iter() {
                self.insert_symbol(
                    param,
                    Symbol::Constant {
                        name: param.clone(),
                    },
                );
            }
            self.pop_scope();
        }
        self.visit_expression(expr);
    }
    fn visit_label(&mut self, name: &Token, _span: Span) {
        self.insert_symbol(name, Symbol::Label { name: name.clone() });
    }
    fn visit_struct(&mut self, name: &Token, members: &[StructMember], span: Span) {
        self.insert_symbol(name, Symbol::Scope { name: name.clone() });

        self.push_scope(name, span);

        for member in members.iter() {
            match member {
                StructMember::Field(field) => {
                    self.insert_symbol(
                        field,
                        Symbol::Constant {
                            name: field.clone(),
                        },
                    );
                }
                StructMember::Struct(strct) => {
                    self.visit_statement(strct); // TODO: this should cause a syntax error if anything except struct.
                }
            }
        }

        self.pop_scope()
    }
    fn visit_enum(&mut self, name: &Option<Token>, members: &[EnumMember], span: Span) {
        if let Some(name) = name {
            self.insert_symbol(name, Symbol::Scope { name: name.clone() });

            self.push_scope(name, span);

            for member in members.iter() {
                self.insert_symbol(
                    &member.name,
                    Symbol::Constant {
                        name: member.name.clone(),
                    },
                );
            }

            self.pop_scope()
        }
    }

    fn visit_repeat(
        &mut self,
        _max: &Expression,
        incr: &Option<Token>,
        statements: &[Statement],
        span: Span,
    ) {
        // TODO: figure out how to have "invisible scopes"
        // self.push_scope("__repeat".to_owned(), span);
        if let Some(incr) = incr {
            self.insert_symbol(incr, Symbol::Constant { name: incr.clone() });
        }
        for statement in statements {
            self.visit_statement(statement);
        }
        // self.pop_scope()
    }

    fn visit_import(&mut self, imports: &[ImportExport], _zero_page: &bool, _span: Span) {
        for import in imports {
            self.insert_symbol(
                &import.name,
                Symbol::Constant {
                    name: import.name.clone(),
                },
            );
        }
    }

    fn visit_export(&mut self, imports: &[ImportExport], _zero_page: &bool, _span: Span) {
        for import in imports {
            if import.value.is_some() {
                self.insert_symbol(
                    &import.name,
                    Symbol::Constant {
                        name: import.name.clone(),
                    },
                );
            }
        }
    }

    fn visit_global(&mut self, identifiers: &[Token], _zero_page: &bool, _span: Span) {
        for identifier in identifiers {
            self.insert_symbol(
                identifier,
                Symbol::Constant {
                    name: identifier.clone(),
                },
            );
        }
    }

    fn visit_include(&mut self, path: &Token, _span: Span) {
        self.includes.push(Include {
            path: path.clone(),
            scope: self.stack.clone(),
        })
    }
}
