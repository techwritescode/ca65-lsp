use codespan::Span;
use parser::{
    ConstantAssign, Expression, ExpressionKind, IfKind, Instruction, MacroInvocation,
    MacroParameter, Statement, StatementKind, StructMember, Token, TokenType,
};

pub trait ASTVisitor {
    fn visit_statement(&mut self, statement: &Statement) {
        match &statement.kind {
            StatementKind::ConstantAssign(stmt) => self.visit_constant_assign(stmt, statement.span),
            StatementKind::Include(path) => self.visit_include(path, statement.span),
            StatementKind::Label(name) => self.visit_label(name, statement.span),
            StatementKind::Instruction(instruction) => {
                self.visit_instruction(instruction, statement.span)
            }
            StatementKind::Procedure(name, statements) => {
                self.visit_procedure(name, statements, statement.span)
            }
            StatementKind::Enum(name, variants) => self.visit_enum(name, variants, statement.span),
            StatementKind::Struct(name, members) => {
                self.visit_struct(name, members, statement.span)
            }
            StatementKind::Macro => self.visit_macro(statement.span),
            StatementKind::SetCPU(cpu) => self.visit_set_cpu(cpu, statement.span),
            StatementKind::Segment(segment) => self.visit_segment(segment, statement.span),
            StatementKind::Reserve(expression) => self.visit_reserve(expression, statement.span),
            StatementKind::MacroInvocation(macro_invocation) => {
                self.visit_macro_invocation(macro_invocation, statement.span)
            }
            StatementKind::MacroPack(pack) => self.visit_macro_pack(pack, statement.span),
            StatementKind::Feature(name) => self.visit_feature(name, statement.span),
            StatementKind::Scope(name, statements) => {
                self.visit_scope(name, statements, statement.span)
            }
            StatementKind::IncludeBinary(path) => self.visit_include_binary(path, statement.span),
            StatementKind::MacroDefinition(name, parameters, statements) => {
                self.visit_macro_definition(name, parameters, statements, statement.span)
            }
            StatementKind::Data(expressions) => self.visit_data(expressions, statement.span),
            StatementKind::Org(address) => self.visit_org(address, statement.span),
            StatementKind::Repeat(max, incr, statements) => {
                self.visit_repeat(max, incr, statements, statement.span)
            }
            StatementKind::Global {
                identifiers,
                zero_page,
            } => self.visit_global(identifiers, zero_page, statement.span),
            StatementKind::Export {
                identifiers,
                zero_page,
            } => self.visit_export(identifiers, zero_page, statement.span),
            StatementKind::Import {
                identifiers,
                zero_page,
            } => self.visit_import(identifiers, zero_page, statement.span),
            StatementKind::Ascii(string) => self.visit_ascii(string, statement.span),
            StatementKind::If(if_statement, statements) => {
                self.visit_if(if_statement, statements, statement.span)
            }
            StatementKind::UnnamedLabel => self.visit_unnamed_label(statement.span),
            StatementKind::Define(ident, expr) => self.visit_define(ident, expr, statement.span),
        }
    }

    fn visit_constant_assign(&mut self, statement: &ConstantAssign, _span: Span) {
        self.visit_expression(&statement.value);
    }
    fn visit_include(&mut self, _path: &Token, _span: Span) {}
    fn visit_label(&mut self, _name: &Token, _span: Span) {}
    fn visit_instruction(&mut self, instruction: &Instruction, _span: Span) {
        for expression in instruction.parameters.iter() {
            self.visit_expression(expression);
        }
    }
    fn visit_procedure(&mut self, _name: &Token, statements: &[Statement], _span: Span) {
        for statement in statements {
            self.visit_statement(statement);
        }
    }
    fn visit_enum(&mut self, _name: &Option<Token>, _variants: &[Expression], _span: Span) {}
    fn visit_struct(&mut self, _name: &Token, _members: &[StructMember], _span: Span) {}
    fn visit_macro(&mut self, _span: Span) {}
    fn visit_set_cpu(&mut self, _cpu: &str, _span: Span) {}
    fn visit_segment(&mut self, _segment: &str, _span: Span) {}
    fn visit_reserve(&mut self, expression: &Expression, _span: Span) {
        self.visit_expression(expression);
    }
    fn visit_macro_invocation(&mut self, macro_invocation: &MacroInvocation, _span: Span) {
        for param in macro_invocation.parameters.iter() {
            if let MacroParameter::Expression(param) = param {
                self.visit_expression(param);
            }
        }
    }
    fn visit_macro_pack(&mut self, _pack: &str, _span: Span) {}
    fn visit_feature(&mut self, _name: &str, _span: Span) {}
    fn visit_scope(&mut self, _name: &Option<Token>, statements: &[Statement], _span: Span) {
        for statement in statements {
            self.visit_statement(statement);
        }
    }
    fn visit_include_binary(&mut self, _path: &Token, _span: Span) {}
    fn visit_macro_definition(
        &mut self,
        _name: &Token,
        _parameters: &[Token],
        statements: &[Statement],
        _span: Span,
    ) {
        for statement in statements {
            self.visit_statement(statement);
        }
    }
    fn visit_data(&mut self, expressions: &[Expression], _span: Span) {
        for expression in expressions {
            self.visit_expression(expression);
        }
    }
    fn visit_org(&mut self, _address: &str, _span: Span) {}
    fn visit_repeat(
        &mut self,
        max: &Expression,
        _incr: &Option<Token>,
        statements: &[Statement],
        _span: Span,
    ) {
        self.visit_expression(max);
        for statement in statements {
            self.visit_statement(statement);
        }
    }
    fn visit_global(&mut self, _identifiers: &[Token], _zero_page: &bool, _span: Span) {}
    fn visit_export(&mut self, _identifiers: &[Token], _zero_page: &bool, _span: Span) {}
    fn visit_import(&mut self, _identifiers: &[Token], _zero_page: &bool, _span: Span) {}
    fn visit_ascii(&mut self, _string: &Token, _span: Span) {}
    fn visit_if(&mut self, if_statement: &IfKind, statements: &[Statement], _span: Span) {
        match if_statement {
            IfKind::WithExpression(expression) => self.visit_expression(expression),
            IfKind::NoParams => {}
            IfKind::WithTokens(_tokens) => {}
        }

        for statement in statements {
            self.visit_statement(statement);
        }
    }
    fn visit_unnamed_label(&mut self, _span: Span) {}
    fn visit_define(&mut self, _ident: &Token, expr: &Expression, _span: Span) {
        self.visit_expression(expr);
    }

    fn visit_expression(&mut self, expression: &Expression) {
        match &expression.kind {
            ExpressionKind::Immediate(expr) => self.visit_immediate(expr, expression.span),
            ExpressionKind::Unary(token, expr) => self.visit_unary(token, expr, expression.span),
            ExpressionKind::Literal(string) => self.visit_literal(string, expression.span),
            ExpressionKind::Group(group) => self.visit_group(group, expression.span),
            ExpressionKind::UnaryPositive(expr) => self.visit_unary_positive(expr, expression.span),
            ExpressionKind::Math(tok, expr1, expr2) => {
                self.visit_math(tok, expr1, expr2, expression.span)
            }
            ExpressionKind::Not(expr) => self.visit_not(expr, expression.span),
            ExpressionKind::Or(expr1, expr2) => self.visit_or(expr1, expr2, expression.span),
            ExpressionKind::And(expr1, expr2) => self.visit_and(expr1, expr2, expression.span),
            ExpressionKind::Xor(expr1, expr2) => self.visit_xor(expr1, expr2, expression.span),
            ExpressionKind::Comparison(tok, expr1, expr2) => {
                self.visit_comparison(tok, expr1, expr2, expression.span)
            }
            ExpressionKind::SimpleExpression(tok, expr1, expr2) => {
                self.visit_simple_expression(tok, expr1, expr2, expression.span)
            }
            ExpressionKind::Term(tok, expr1, expr2) => {
                self.visit_term(tok, expr1, expr2, expression.span)
            }
            ExpressionKind::Bank(expr) => self.visit_bank(expr, expression.span),
            ExpressionKind::SizeOf(expr) => self.visit_sizeof(expr, expression.span),
            ExpressionKind::Match(expr1, expr2) => self.visit_match(expr1, expr2, expression.span),
            ExpressionKind::Def(tok) => self.visit_def(tok, expression.span),
            ExpressionKind::Identifier(ident) => self.visit_identifier(ident, expression.span),
            ExpressionKind::UnnamedLabelReference(reference) => {
                self.visit_unnamed_label_reference(reference, expression.span)
            }
            ExpressionKind::String(str) => self.visit_string(str, expression.span),
            ExpressionKind::Extract(tok, expr1, expr2) => {
                self.visit_extract(tok, expr1, expr2, expression.span)
            }
            ExpressionKind::TokenList(toks) => self.visit_token_list(toks, expression.span),
        }
    }

    fn visit_immediate(&mut self, expression: &Expression, _span: Span) {
        self.visit_expression(expression);
    }
    fn visit_unary(&mut self, _token: &TokenType, expression: &Expression, _span: Span) {
        self.visit_expression(expression);
    }
    fn visit_literal(&mut self, _string: &str, _span: Span) {}
    fn visit_group(&mut self, group: &Expression, _span: Span) {
        self.visit_expression(group);
    }
    fn visit_unary_positive(&mut self, expr: &Expression, _span: Span) {
        self.visit_expression(expr);
    }
    fn visit_math(
        &mut self,
        _tok: &TokenType,
        expr1: &Expression,
        expr2: &Expression,
        _span: Span,
    ) {
        self.visit_expression(expr1);
        self.visit_expression(expr2);
    }
    fn visit_not(&mut self, expr: &Expression, _span: Span) {
        self.visit_expression(expr);
    }
    fn visit_or(&mut self, expr1: &Expression, expr2: &Expression, _span: Span) {
        self.visit_expression(expr1);
        self.visit_expression(expr2);
    }
    fn visit_and(&mut self, expr1: &Expression, expr2: &Expression, _span: Span) {
        self.visit_expression(expr1);
        self.visit_expression(expr2);
    }
    fn visit_xor(&mut self, expr1: &Expression, expr2: &Expression, _span: Span) {
        self.visit_expression(expr1);
        self.visit_expression(expr2);
    }
    fn visit_comparison(
        &mut self,
        _tok: &TokenType,
        expr1: &Expression,
        expr2: &Expression,
        _span: Span,
    ) {
        self.visit_expression(expr1);
        self.visit_expression(expr2);
    }
    fn visit_simple_expression(
        &mut self,
        _tok: &Token,
        expr1: &Expression,
        expr2: &Expression,
        _span: Span,
    ) {
        self.visit_expression(expr1);
        self.visit_expression(expr2);
    }
    fn visit_term(
        &mut self,
        _tok: &TokenType,
        expr1: &Expression,
        expr2: &Expression,
        _span: Span,
    ) {
        self.visit_expression(expr1);
        self.visit_expression(expr2);
    }
    fn visit_bank(&mut self, tok: &Expression, _span: Span) {
        self.visit_expression(tok);
    }
    fn visit_sizeof(&mut self, expr: &Expression, _span: Span) {
        self.visit_expression(expr);
    }
    fn visit_match(&mut self, _expr1: &Expression, _expr2: &Expression, _span: Span) {}
    fn visit_def(&mut self, _tok: &Token, _span: Span) {}
    fn visit_identifier(&mut self, _ident: &str, _span: Span) {}
    fn visit_unnamed_label_reference(&mut self, _reference: &i8, _span: Span) {}
    fn visit_string(&mut self, _str: &str, _span: Span) {}
    fn visit_extract(
        &mut self,
        _tok: &Token,
        _expr1: &Expression,
        _expr2: &Expression,
        _span: Span,
    ) {
    }
    fn visit_token_list(&mut self, _toks: &[Token], _span: Span) {}
}
