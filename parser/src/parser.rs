use crate::{Token, TokenType};
use codespan::Span;
use std::fmt::{Display, Formatter};

macro_rules! match_token {
    ($stream:expr, $token:pat) => {
        if let Some(Token {
            token_type: $token, ..
        }) = $stream.peek()
        {
            $stream.advance();
            true
        } else {
            false
        }
    };
}

macro_rules! check_token {
    ($stream:expr, $token:pat) => {
        if let Some(Token {
            token_type: $token, ..
        }) = $stream.peek()
        {
            true
        } else {
            false
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportExport {
    pub name: Token,
    pub far: bool,
    pub value: Option<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken(Token),
    Expected {
        expected: TokenType,
        received: Token,
    },
    EOF,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError")
        // todo!()
    }
}

type Result<T> = std::result::Result<T, ParseError>;

pub struct TokenStream<'a> {
    tokens: &'a Vec<Token>,
    position: usize,
}

impl<'a> TokenStream<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    pub fn advance(&mut self) -> Option<Token> {
        if !self.at_end() {
            self.position += 1;
        }
        self.previous().ok()
    }

    pub fn peek(&self) -> Option<Token> {
        if !self.at_end() {
            // println!("Peeking {:#?}", self.tokens[self.position]);
            return Some(self.tokens[self.position].clone());
        }
        None
    }

    pub fn previous(&self) -> Result<Token> {
        if self.position > 0 {
            Ok(self.tokens[self.position - 1].clone())
        } else {
            Err(ParseError::EOF)
        }
    }

    fn at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstantAssign {
    pub name: Token,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    Immediate(Box<Expression>),
    Unary(TokenType, Box<Expression>),
    Literal(String),
    UnnamedLabelReference(i8),
    Group(Box<Expression>),
    MemoryAccess(Box<Expression>),
    UnaryPositive(Box<Expression>),
    Math(TokenType, Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Xor(Box<Expression>, Box<Expression>),
    Comparison(TokenType, Box<Expression>, Box<Expression>),
    SimpleExpression(Token, Box<Expression>, Box<Expression>),
    Term(TokenType, Box<Expression>, Box<Expression>),
    Bank(Box<Expression>),
    SizeOf(Box<Expression>),
    Identifier(String),
    Match(Box<Expression>, Box<Expression>),
    Def(Token),
    String(String),
    Extract(Token, Box<Expression>, Box<Expression>),
    TokenList(Vec<Token>),
    Call(String, Vec<Expression>),
    WordOp(Token, Box<Expression>),
    Ident(Box<Expression>),
    Sprintf(Box<Expression>, Vec<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroParameter {
    Opcode(Token),
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub mnemonic: String,
    pub parameters: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IfKind {
    WithExpression(Expression),
    WithTokens(Vec<Token>),
    NoParams,
}

pub struct IfStatement {
    pub kind: IfKind,
    pub if_body: Vec<Statement>,
    pub else_body: Option<Vec<Statement>>,
    pub else_ifs: Option<Vec<(Expression, Vec<Statement>)>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    ConstantAssign(ConstantAssign),
    Include(Token),
    Label(Token),
    UnnamedLabel,
    Instruction(Instruction),
    Procedure(Token, bool, Vec<Statement>),
    Enum(Option<Token>, Vec<EnumMember>),
    Macro,
    SetCPU(String),
    Segment(Segment),
    Tag(Expression),
    Reserve(Expression, Option<Expression>),

    MacroInvocation(MacroInvocation),
    MacroPack(String),
    Feature(String),
    Scope(Option<Token>, Vec<Statement>),
    IncludeBinary(Token, Option<Token>, Option<Token>),
    MacroDefinition(Token, Vec<Token>, Vec<Statement>),
    Data(Vec<Expression>),
    Org(String),
    Repeat(Expression, Option<Token>, Vec<Statement>),
    Global {
        identifiers: Vec<Token>,
        zero_page: bool,
    },
    Export {
        exports: Vec<ImportExport>,
        zero_page: bool,
    },
    Ascii(Token),
    If(IfKind, Vec<Statement>),
    Struct(Token, Vec<StructMember>),
    Import {
        imports: Vec<ImportExport>,
        zero_page: bool,
    },
    Define(Token, Option<Vec<Token>>, Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
    Literal(String),
    Identifier(Token),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StructMember {
    Struct(Statement),
    Field(Token), // TODO: Add data type
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumMember {
    pub name: Token,
    pub value: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

pub type Ast = Vec<Statement>;

#[derive(Debug, Clone, PartialEq)]
pub enum ControlCommandType {}

#[derive(Debug, Clone, PartialEq)]
pub struct ControlCommand {
    pub control_type: ControlCommandType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroInvocation {
    pub name: Token,
    pub parameters: Vec<MacroParameter>,
}

pub struct Parser<'a> {
    tokens: TokenStream<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self {
            tokens: TokenStream::new(tokens),
        }
    }

    pub fn parse(&mut self) -> (Ast, Vec<ParseError>) {
        let mut lines = vec![];
        let mut errors = vec![];

        while !self.tokens.at_end() {
            let result = self.parse_line();
            match result {
                Ok(Some(operation)) => lines.push(operation),
                Ok(None) => (),
                Err(e) => {
                    errors.push(e);
                    self.error_recovery();
                }
            }
        }

        (lines, errors)
    }

    fn parse_line(&mut self) -> Result<Option<Statement>> {
        if let Some(token) = self.tokens.peek() {
            let operation = match token.token_type {
                TokenType::Macro => self.parse_macro(),
                TokenType::Identifier => Ok(Some(self.parse_assignment()?)),
                TokenType::Instruction => Ok(Some(self.parse_instruction()?)),
                TokenType::Colon => self.parse_unnamed_label(),
                TokenType::EOL => {
                    self.tokens.advance();
                    Ok(None)
                }
                _ => Err(ParseError::UnexpectedToken(token)),
            };

            return operation;
        }

        Err(ParseError::UnexpectedToken(self.tokens.peek().unwrap()))
    }

    fn parse_macro(&mut self) -> Result<Option<Statement>> {
        if match_token!(self.tokens, TokenType::Macro) {
            let mac = self.tokens.previous()?;
            let start = self.mark_start();
            let ident = &mac.lexeme;
            let macro_matcher = ident.to_lowercase();
            return match macro_matcher.as_str() {
                ".global" | ".globalzp" => {
                    let zero_page = macro_matcher == ".globalzp";
                    let mut identifiers = vec![];
                    identifiers.push(self.consume_token(TokenType::Identifier)?);
                    while match_token!(self.tokens, TokenType::Comma) {
                        identifiers.push(self.consume_token(TokenType::Identifier)?);
                    }
                    let end = self.mark_end();
                    self.consume_newline()?;
                    Ok(Some(Statement {
                        kind: StatementKind::Global {
                            identifiers,
                            zero_page,
                        },
                        span: Span::new(start, end),
                    }))
                }
                ".export" | ".exportzp" => {
                    let zp = macro_matcher == ".exportzp";
                    let exports = self.parse_import_export_list()?;
                    let end = self.mark_end();
                    self.consume_newline()?;
                    Ok(Some(Statement {
                        kind: StatementKind::Export {
                            exports,
                            zero_page: zp,
                        },
                        span: Span::new(start, end),
                    }))
                }
                ".import" | ".importzp" => {
                    let is_zp = macro_matcher == ".importzp";
                    let imports = self.parse_import_export_list()?;
                    let end = self.mark_end();
                    self.consume_newline()?;
                    Ok(Some(Statement {
                        kind: StatementKind::Import {
                            imports,
                            zero_page: is_zp,
                        },
                        span: Span::new(start, end),
                    }))
                }
                ".macpack" => {
                    self.consume_token(TokenType::Identifier)?;
                    let pack = self.last().lexeme;
                    let end = self.mark_end();
                    self.consume_newline()?;
                    Ok(Some(Statement {
                        kind: StatementKind::MacroPack(pack),
                        span: Span::new(start, end),
                    }))
                }
                ".feature" => {
                    self.consume_token(TokenType::Identifier)?;
                    let feature = self.last().lexeme;
                    let end = self.mark_end();
                    self.consume_newline()?;
                    Ok(Some(Statement {
                        kind: StatementKind::Feature(feature),
                        span: Span::new(start, end),
                    }))
                }
                ".include" => {
                    self.consume_token(TokenType::String)?;
                    let path = self.last();
                    let end = self.mark_end();
                    self.consume_newline()?;
                    Ok(Some(Statement {
                        kind: StatementKind::Include(path),
                        span: Span::new(start, end),
                    }))
                }
                ".incbin" => {
                    self.consume_token(TokenType::String)?;
                    let path = self.last();
                    let mut bin_offset = None;
                    let mut bin_end = None;
                    if match_token!(self.tokens, TokenType::Comma) {
                        bin_offset = Some(self.consume_token(TokenType::Number)?);
                    }
                    if match_token!(self.tokens, TokenType::Comma) {
                        bin_end = Some(self.consume_token(TokenType::Number)?);
                    }
                    let end = self.mark_end();
                    self.consume_newline()?;
                    Ok(Some(Statement {
                        kind: StatementKind::IncludeBinary(path, bin_offset, bin_end),
                        span: Span::new(start, end),
                    }))
                }
                ".setcpu" => {
                    self.consume_token(TokenType::String)?;
                    let cpu = self.last().lexeme;
                    let end = self.mark_end();
                    self.consume_newline()?;

                    Ok(Some(Statement {
                        kind: StatementKind::SetCPU(cpu),
                        span: Span::new(start, end),
                    }))
                }
                ".org" => {
                    let address = self.consume_token(TokenType::Number)?;
                    let end = self.mark_end();
                    self.consume_newline()?;

                    Ok(Some(Statement {
                        kind: StatementKind::Org(address.lexeme.clone()),
                        span: Span::new(start, end),
                    }))
                }
                ".segment" => {
                    if match_token!(self.tokens, TokenType::String) {
                        // string
                    } else if match_token!(self.tokens, TokenType::Identifier) {
                        // identifier
                    } else {
                        return Err(ParseError::UnexpectedToken(self.peek()?));
                    }
                    // self.consume_token(TokenType::String)?;
                    let segment = self.last();
                    let end = self.mark_end();
                    self.consume_newline()?;

                    Ok(Some(Statement {
                        kind: StatementKind::Segment(Segment::Identifier(segment)),
                        span: Span::new(start, end),
                    }))
                }
                ".macro" | ".mac" => Ok(Some(self.parse_macro_def()?)),
                ".enum" => Ok(Some(self.parse_enum()?)),
                ".proc" => {
                    self.consume_token(TokenType::Identifier)?;
                    let ident = self.last();
                    let far = if match_token!(self.tokens, TokenType::Colon) {
                        self.consume_token(TokenType::Identifier)?;
                        self.last().lexeme == "far"
                    } else {
                        false
                    };

                    self.consume_newline()?;
                    let commands: Vec<Statement> = self.parse_statement_block(&[".endproc"])?;
                    let end = self.mark_end();
                    return Ok(Some(Statement {
                        kind: StatementKind::Procedure(ident, far, commands),
                        span: Span::new(start, end),
                    }));
                }
                ".scope" => {
                    let ident = if match_token!(self.tokens, TokenType::Identifier) {
                        Some(self.last())
                    } else {
                        None
                    };
                    self.consume_newline()?;
                    let commands = self.parse_statement_block(&[".endscope"])?;
                    let end = self.mark_end();
                    return Ok(Some(Statement {
                        kind: StatementKind::Scope(ident, commands),
                        span: Span::new(start, end),
                    }));
                }
                ".repeat" => {
                    let max = self.parse_expression()?;
                    let iter = if match_token!(self.tokens, TokenType::Comma) {
                        let ident = self.consume_token(TokenType::Identifier)?;
                        Some(ident)
                    } else {
                        None
                    };
                    self.consume_newline()?;
                    let commands = self.parse_statement_block(&[".endrepeat", ".endrep"])?;
                    let end = self.mark_end();
                    return Ok(Some(Statement {
                        kind: StatementKind::Repeat(max, iter, commands),
                        span: Span::new(start, end),
                    }));
                }
                ".tag" => {
                    let right = self.parse_expression()?;
                    let end = self.mark_end();
                    self.consume_newline()?;

                    // Todo: add type
                    Ok(Some(Statement {
                        kind: StatementKind::Tag(right),
                        span: Span::new(start, end),
                    }))
                }
                ".res" => {
                    let amount = self.parse_expression()?;
                    let val = if match_token!(self.tokens, TokenType::Comma) {
                        Some(self.parse_expression()?)
                    } else {
                        None
                    };
                    let end = self.mark_end();
                    self.consume_newline()?;

                    Ok(Some(Statement {
                        kind: StatementKind::Reserve(amount, val),
                        span: Span::new(start, end),
                    }))
                }
                ".zeropage" | ".code" | ".bss" | ".data" | ".rodata" => {
                    let end = self.mark_end();
                    self.consume_newline()?;

                    let segment_name = macro_matcher[1..].to_string();

                    Ok(Some(Statement {
                        kind: StatementKind::Segment(Segment::Literal(segment_name)),
                        span: Span::new(start, end),
                    }))
                }
                ".db" | ".dw" | ".byte" | ".word" | ".dword" | ".lobytes" => {
                    let parameters = self.parse_parameters()?;
                    let end = self.mark_end();
                    self.consume_newline()?;

                    // TODO: Add kind
                    Ok(Some(Statement {
                        kind: StatementKind::Data(parameters),
                        span: Span::new(start, end),
                    }))
                }
                ".asciiz" => {
                    let string = self.consume_token(TokenType::String)?;
                    let end = self.mark_end();
                    self.consume_newline()?;

                    Ok(Some(Statement {
                        kind: StatementKind::Ascii(string),
                        span: Span::new(start, end),
                    }))
                }
                ".struct" => Ok(Some(self.parse_struct()?)),
                ".define" => Ok(Some(self.parse_define()?)),
                ".if" | ".ifconst" | ".ifblank" | ".ifnblank" | ".ifdef" | ".ifndef" | ".ifref"
                | ".ifnref" | ".ifp02" | ".ifp4510" | ".ifp816" | ".ifpC02" => {
                    Ok(Some(self.parse_if()?))
                }
                ".smart" | ".autoimport" => {
                    if match_token!(self.tokens, TokenType::Plus | TokenType::Minus) {}
                    Ok(None)
                }
                // Ignored for now
                ".local" | ".index" | ".mem" | ".align" | ".addr" | ".charmap" | ".assert"
                | ".p816" | ".i8" | ".i16" | ".a8" | ".a16" | ".error" => {
                    self.parse_parameters()?;
                    Ok(None)
                }
                _ => Err(ParseError::UnexpectedToken(mac)),
            };
        }

        Ok(Some(self.parse_assignment()?))
    }

    fn parse_if(&mut self) -> Result<Statement> {
        let start = self.mark_start();
        let if_token = self.last();
        let if_kind = match if_token.lexeme.as_str() {
            ".if" | ".ifconst" => IfKind::WithExpression(self.parse_expression()?),
            ".ifblank" | ".ifnblank" | ".ifdef" | ".ifndef" | ".ifref" | ".ifnref" => {
                IfKind::WithTokens(self.parse_parameters_tokens()?)
            }
            ".ifp02" | ".ifp4510" | ".ifp816" | ".ifpC02" => IfKind::NoParams,
            _ => {
                unreachable!(".if strings in parse_if() do not match .if strings in parse_macro()")
            }
        };
        self.consume_newline()?;

        let mut commands: Vec<Statement> = vec![];

        while !self.tokens.at_end() {
            if check_token!(self.tokens, TokenType::Macro) {
                let tok_lexeme = self.peek()?.lexeme;
                match tok_lexeme.as_str() {
                    ".elseif" => {
                        self.tokens.advance();
                        self.parse_expression()?;
                        self.consume_newline()?;
                    }
                    ".else" => {
                        self.tokens.advance();
                        self.consume_newline()?;
                    }
                    ".endif" => {
                        self.tokens.advance();
                        let end = self.mark_end();
                        return Ok(Statement {
                            kind: StatementKind::If(if_kind, commands),
                            span: Span::new(start, end),
                        });
                    }
                    _ => (),
                }
            }
            if let Some(line) = self.parse_line()? {
                commands.push(line);
            }
        }

        Err(ParseError::Expected {
            received: self.peek()?,
            expected: TokenType::Macro,
        })
    }

    fn parse_macro_def(&mut self) -> Result<Statement> {
        let start = self.mark_start();
        self.consume_token(TokenType::Identifier)?;
        let ident = self.last();
        let mut parameters = vec![];

        while match_token!(self.tokens, TokenType::Identifier) {
            parameters.push(self.last());
            if self.consume_token(TokenType::Comma).is_err() {
                break;
            }
        }
        self.consume_newline()?;

        let commands = self.parse_statement_block(&[".endmacro"])?;
        let end = self.mark_end();
        Ok(Statement {
            kind: StatementKind::MacroDefinition(ident, parameters, commands),
            span: Span::new(start, end),
        })
    }

    fn parse_enum(&mut self) -> Result<Statement> {
        let start = self.mark_start();

        // enums can either be named or unnamed
        let ident: Option<Token> = if check_token!(self.tokens, TokenType::Identifier) {
            self.consume_token(TokenType::Identifier)?;
            Some(self.last())
        } else {
            None
        };

        self.consume_newline()?;

        let mut members: Vec<EnumMember> = Vec::new();
        while !self.tokens.at_end() {
            if check_token!(self.tokens, TokenType::Macro) {
                let macro_lexeme = self.peek()?.lexeme;
                if macro_lexeme == ".endenum" {
                    self.tokens.advance();
                    let end = self.mark_end();
                    return Ok(Statement {
                        kind: StatementKind::Enum(ident, members),
                        span: Span::new(start, end),
                    });
                }
            }
            let name = self.consume_token(TokenType::Identifier)?;
            let value = if match_token!(self.tokens, TokenType::Equal) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            members.push(EnumMember { name, value });
            self.consume_newline()?;
        }

        // never encountered ".endenum"
        Err(ParseError::Expected {
            received: self.peek()?,
            expected: TokenType::Macro,
        })
    }

    fn parse_struct(&mut self) -> Result<Statement> {
        let start = self.mark_start();
        let ident = self.consume_token(TokenType::Identifier)?;

        self.consume_newline()?;

        let mut members: Vec<StructMember> = Vec::new();
        while !self.tokens.at_end() {
            if check_token!(self.tokens, TokenType::Macro) {
                let macro_lexeme = self.peek()?.lexeme;
                match macro_lexeme.as_str() {
                    ".endstruct" => {
                        self.tokens.advance();
                        let end = self.mark_end();
                        return Ok(Statement {
                            kind: StatementKind::Struct(ident, members),
                            span: Span::new(start, end),
                        });
                    }
                    ".struct" => {
                        self.tokens.advance();
                        members.push(StructMember::Struct(self.parse_struct()?));
                    }
                    _ => {
                        return Err(ParseError::Expected {
                            received: self.peek()?,
                            expected: TokenType::Macro,
                        });
                    }
                }
            } else {
                let ident = self.consume_token(TokenType::Identifier)?;
                let _data_type = self.consume_token(TokenType::Macro)?; // TODO: add data type
                match_token!(self.tokens, TokenType::Identifier);
                match_token!(self.tokens, TokenType::Number);
                members.push(StructMember::Field(ident));
                self.consume_newline()?;
            }
        }

        // never encountered ".endenum"
        Err(ParseError::Expected {
            received: self.peek()?,
            expected: TokenType::Macro,
        })
    }

    fn parse_define(&mut self) -> Result<Statement> {
        let start = self.mark_start();
        let ident = self.consume_token(TokenType::Identifier)?;
        let params = if match_token!(self.tokens, TokenType::LeftParen) {
            let mut params = vec![self.consume_token(TokenType::Identifier)?];
            while !self.tokens.at_end() && match_token!(self.tokens, TokenType::Comma) {
                params.push(self.consume_token(TokenType::Identifier)?);
            }
            self.consume_token(TokenType::RightParen)?;
            Some(params)
        } else {
            None
        };
        let value = self.parse_expression()?;
        let end = self.mark_end();
        self.consume_newline()?;

        Ok(Statement {
            kind: StatementKind::Define(ident, params, value),
            span: Span::new(start, end),
        })
    }

    fn parse_assignment(&mut self) -> Result<Statement> {
        if let Some(token) = self.tokens.peek() {
            if match_token!(self.tokens, TokenType::Identifier) {
                let start = self.last().span.start;
                if match_token!(self.tokens, TokenType::Equal) {
                    let value = self.parse_expression()?;
                    let end = self.tokens.previous()?.span.end;
                    let operation = StatementKind::ConstantAssign(ConstantAssign {
                        name: token,
                        value,
                        span: Span::new(start, end),
                    });

                    self.consume_newline()?;

                    return Ok(Statement {
                        kind: operation,
                        span: Span::new(start, end),
                    });
                }
                if match_token!(self.tokens, TokenType::ConstAssign) {
                    let value = self.parse_expression()?;
                    let end = self.tokens.previous()?.span.end;
                    let operation = StatementKind::ConstantAssign(ConstantAssign {
                        name: token,
                        value,
                        span: Span::new(start, end),
                    });

                    self.consume_newline()?;

                    return Ok(Statement {
                        kind: operation,
                        span: Span::new(start, end),
                    });
                }
                if check_token!(self.tokens, TokenType::Colon) {
                    return self.parse_label();
                }
                return self.parse_macro_invocation();
            }
        }
        self.parse_instruction()
    }

    fn parse_instruction(&mut self) -> Result<Statement> {
        if match_token!(self.tokens, TokenType::Instruction) {
            let mnemonic = self.last().lexeme;
            let start = self.mark_start();
            let parameters = self.parse_parameters()?;
            let end = self.mark_end();

            self.consume_newline()?;

            return Ok(Statement {
                kind: StatementKind::Instruction(Instruction {
                    mnemonic,
                    parameters,
                }),
                span: Span::new(start, end),
            });
        }
        Err(ParseError::Expected {
            expected: TokenType::Instruction,
            received: self.peek()?,
        })
    }

    fn parse_label(&mut self) -> Result<Statement> {
        let start = self.mark_start();
        let name = self.tokens.previous()?;
        self.consume_token(TokenType::Colon)?;

        let end = self.mark_end();
        Ok(Statement {
            kind: StatementKind::Label(name),
            span: Span::new(start, end),
        })
    }

    fn parse_unnamed_label(&mut self) -> Result<Option<Statement>> {
        let start = self.mark_start();
        self.consume_token(TokenType::Colon)?;
        let end = self.mark_end();
        Ok(Some(Statement {
            kind: StatementKind::UnnamedLabel,
            span: Span::new(start, end),
        }))
    }

    fn parse_macro_invocation(&mut self) -> Result<Statement> {
        let start = self.mark_start();
        let name = self.tokens.previous()?;
        let parameters = self.parse_macro_parameters()?;
        let end = self.mark_end();
        Ok(Statement {
            kind: StatementKind::MacroInvocation(MacroInvocation { name, parameters }),
            span: Span::new(start, end),
        })
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_expr0()
    }

    fn parse_expr0(&mut self) -> Result<Expression> {
        if match_token!(self.tokens, TokenType::Not) {
            let start = self.mark_start();
            let expr = self.parse_expr0()?;
            let end = self.mark_end();
            Ok(Expression {
                kind: ExpressionKind::Not(Box::from(expr)),
                span: Span::new(start, end),
            })
        } else {
            self.parse_expr1()
        }
    }

    fn parse_expr1(&mut self) -> Result<Expression> {
        let mut root = self.parse_expr2()?;
        while match_token!(self.tokens, TokenType::Or) {
            let right = self.parse_expr2()?;
            root = Expression {
                kind: ExpressionKind::Or(Box::from(root.clone()), Box::from(right.clone())),
                span: Span::new(root.span.start, right.span.end),
            };
        }
        Ok(root)
    }

    fn parse_expr2(&mut self) -> Result<Expression> {
        let mut root = self.parse_bool_expr()?;
        while match_token!(
            self.tokens,
            TokenType::And | TokenType::Xor | TokenType::Caret
        ) {
            let tok = self.tokens.previous()?;
            let right = self.parse_bool_expr()?;
            match tok.token_type {
                TokenType::And => {
                    root = Expression {
                        kind: ExpressionKind::And(
                            Box::from(root.clone()),
                            Box::from(right.clone()),
                        ),
                        span: Span::new(root.span.start, right.span.end),
                    };
                }
                TokenType::Xor | TokenType::Caret => {
                    root = Expression {
                        kind: ExpressionKind::Xor(
                            Box::from(root.clone()),
                            Box::from(right.clone()),
                        ),
                        span: Span::new(root.span.start, right.span.end),
                    };
                }
                _ => {
                    unreachable!("NANI")
                }
            }
        }

        Ok(root)
    }

    fn parse_bool_expr(&mut self) -> Result<Expression> {
        let mut root = self.parse_simple_expression()?;

        while match_token!(
            self.tokens,
            TokenType::Equal
                | TokenType::NotEqual
                | TokenType::LessThan
                | TokenType::GreaterThan
                | TokenType::LessThanEq
                | TokenType::GreaterThanEq
        ) {
            let right = self.parse_simple_expression()?;
            root = Expression {
                kind: ExpressionKind::Comparison(
                    self.tokens.previous()?.token_type,
                    Box::from(root.clone()),
                    Box::from(right.clone()),
                ),
                span: Span::new(root.span.start, right.span.end),
            };
        }

        Ok(root)
    }

    fn parse_simple_expression(&mut self) -> Result<Expression> {
        let mut root = self.parse_term()?;

        while match_token!(
            self.tokens,
            TokenType::Plus | TokenType::Minus | TokenType::BitwiseOr
        ) {
            let operand = self.last();
            let right = self.parse_term()?;
            let end = self.mark_end();
            root = Expression {
                kind: ExpressionKind::SimpleExpression(
                    operand,
                    Box::from(root.clone()),
                    Box::from(right.clone()),
                ),
                span: Span::new(root.span.start, end),
            };
        }

        Ok(root)
    }

    fn parse_term(&mut self) -> Result<Expression> {
        let mut root = self.parse_factor()?;

        while match_token!(
            self.tokens,
            TokenType::Multiply
                | TokenType::Divide
                | TokenType::Mod
                | TokenType::BitwiseAnd
                | TokenType::BitwiseXor
                | TokenType::ShiftLeft
                | TokenType::ShiftRight
        ) {
            let right = self.parse_factor()?;

            root = Expression {
                kind: ExpressionKind::Term(
                    self.tokens.previous()?.token_type,
                    Box::from(root.clone()),
                    Box::from(right.clone()),
                ),
                span: Span::new(root.span.start, right.span.end),
            };
        }

        Ok(root)
    }

    fn parse_factor(&mut self) -> Result<Expression> {
        let start = self.mark_start();
        if match_token!(self.tokens, TokenType::Hash) {
            let right = self.parse_factor()?;
            let end = self.mark_end();
            return Ok(Expression {
                kind: ExpressionKind::Immediate(Box::from(right)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::Plus) {
            let right = self.parse_factor()?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::Unary(TokenType::Plus, Box::from(right)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::Minus) {
            let right = self.parse_factor()?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::Unary(TokenType::Minus, Box::from(right)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::BitwiseNot) {
            let right = self.parse_factor()?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::Unary(TokenType::BitwiseNot, Box::from(right)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::LessThan) {
            let right = self.parse_factor()?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::Unary(TokenType::LessThan, Box::from(right)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::GreaterThan) {
            let right = self.parse_factor()?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::Unary(TokenType::GreaterThan, Box::from(right)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::Bank) {
            self.consume_token(TokenType::LeftParen)?;
            let expr = self.parse_expression()?;
            self.consume_token(TokenType::RightParen)?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::Bank(Box::from(expr)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::SizeOf) {
            self.consume_token(TokenType::LeftParen)?;
            let expr = self.parse_expression()?;
            self.consume_token(TokenType::RightParen)?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::SizeOf(Box::from(expr)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::WordOp) {
            let variant = self.last();
            self.consume_token(TokenType::LeftParen)?;
            let expr = self.parse_expression()?;
            self.consume_token(TokenType::RightParen)?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::WordOp(variant, Box::from(expr)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::Match) {
            self.consume_token(TokenType::LeftParen)?;
            let expr1 = self.parse_token_list(TokenType::Comma)?;
            self.consume_token(TokenType::Comma)?;
            let expr2 = self.parse_token_list(TokenType::RightParen)?;
            self.consume_token(TokenType::RightParen)?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::Match(Box::new(expr1), Box::new(expr2)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::Ident) {
            self.consume_token(TokenType::LeftParen)?;
            let expr = self.parse_expression()?;
            self.consume_token(TokenType::RightParen)?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::Ident(Box::from(expr)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::Sprintf) {
            self.consume_token(TokenType::LeftParen)?;
            let expr = self.parse_expression()?;
            let mut args = vec![];
            while match_token!(self.tokens, TokenType::Comma) {
                args.push(self.parse_expression()?);
            }
            self.consume_token(TokenType::RightParen)?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::Sprintf(Box::from(expr), args),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::Extract) {
            return self.parse_extract();
        }
        if match_token!(self.tokens, TokenType::Def) {
            self.consume_token(TokenType::LeftParen)?;
            self.tokens.advance();
            let tok = self.last();
            self.consume_token(TokenType::RightParen)?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::Def(tok),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::Caret) {
            let right = self.parse_factor()?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::Unary(TokenType::Caret, Box::from(right)),
                span: Span::new(start, end),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression> {
        let start = self.mark_start();
        if match_token!(self.tokens, TokenType::LeftBrace) {
            // let start = self.mark_start();
            let ident = self.parse_identifier()?;
            self.consume_token(TokenType::RightBrace)?;
            // let end = self.mark_end();
            return Ok(ident);
        }
        if match_token!(self.tokens, TokenType::Multiply) {
            let end = self.mark_end();
            return Ok(Expression {
                kind: ExpressionKind::Literal("*".to_owned()),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::Number) {
            let num = self.last().lexeme;
            let end = self.mark_end();
            return Ok(Expression {
                kind: ExpressionKind::Literal(num),
                span: Span::new(start, end),
            });
        }
        if check_token!(self.tokens, TokenType::ScopeSeparator)
            || check_token!(self.tokens, TokenType::Identifier)
        {
            return self.parse_identifier();
        }
        if match_token!(self.tokens, TokenType::String) {
            let string = self.last().lexeme;
            let end = self.mark_end();
            return Ok(Expression {
                kind: ExpressionKind::String(string),
                span: Span::new(start, end),
            });
        }
        if check_token!(self.tokens, TokenType::UnnamedLabelReference) {
            return self.parse_unnamed_label_reference();
        }
        if match_token!(self.tokens, TokenType::LeftParen) {
            let start = self.mark_start();
            let expr = self.parse_expression()?;
            // TODO: this needs to be handled. Used for instructions like JSR (label, x)
            while match_token!(self.tokens, TokenType::Comma) {
                self.parse_expression()?;
            }
            self.consume_token(TokenType::RightParen)?;
            let end = self.mark_end();
            return Ok(Expression {
                kind: ExpressionKind::Group(Box::from(expr)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::LeftBrace) {
            let expr = self.parse_expression()?;
            self.consume_token(TokenType::RightBrace)?;
            return Ok(expr);
        }
        if match_token!(self.tokens, TokenType::LeftBracket) {
            let start = self.mark_start();
            let expr = self.parse_expression()?;
            self.consume_token(TokenType::RightBracket)?;
            let end = self.mark_end();
            return Ok(Expression {
                kind: ExpressionKind::MemoryAccess(Box::from(expr)),
                span: Span::new(start, end),
            });
        }
        if check_token!(self.tokens, TokenType::Macro) {
            let start = self.mark_start();
            let macro_name = self.consume_token(TokenType::Macro)?.lexeme;
            let end = self.mark_end();
            return match macro_name.as_str() {
                ".asize" | ".isize" => Ok(Expression {
                    kind: ExpressionKind::Literal(macro_name),
                    span: Span::new(start, end),
                }),
                _ => Err(ParseError::UnexpectedToken(self.peek()?)),
            };
        }
        Err(ParseError::UnexpectedToken(self.peek()?))
    }

    fn parse_macro_parameters(&mut self) -> Result<Vec<MacroParameter>> {
        let mut parameters = vec![];
        if !check_token!(self.tokens, TokenType::EOL) {
            if matches!(
                self.tokens.peek(),
                Some(Token {
                    token_type: TokenType::Instruction,
                    ..
                })
            ) {
                parameters.push(MacroParameter::Opcode(self.tokens.advance().unwrap()));
            } else {
                parameters.push(MacroParameter::Expression(self.parse_expression()?));
            }
            while !self.tokens.at_end() && !check_token!(self.tokens, TokenType::EOL) {
                self.consume_token(TokenType::Comma)?;
                if matches!(
                    self.tokens.peek(),
                    Some(Token {
                        token_type: TokenType::Instruction,
                        ..
                    })
                ) {
                    parameters.push(MacroParameter::Opcode(self.tokens.advance().unwrap()));
                } else {
                    parameters.push(MacroParameter::Expression(self.parse_expression()?));
                }
            }
        }
        Ok(parameters)
    }
    fn parse_parameters(&mut self) -> Result<Vec<Expression>> {
        let mut parameters = vec![];
        if !self.tokens.at_end() && !check_token!(self.tokens, TokenType::EOL) {
            parameters.push(self.parse_expression()?);
            while !self.tokens.at_end() && match_token!(self.tokens, TokenType::Comma) {
                parameters.push(self.parse_expression()?);
            }
        }
        Ok(parameters)
    }

    fn parse_parameters_tokens(&mut self) -> Result<Vec<Token>> {
        let mut parameters: Vec<Token> = vec![];
        parameters.push(self.consume_token(TokenType::Identifier)?);
        while !self.tokens.at_end() && match_token!(self.tokens, TokenType::Comma) {
            parameters.push(self.consume_token(TokenType::Identifier)?);
        }
        Ok(parameters)
    }

    fn consume_newline(&mut self) -> Result<()> {
        if check_token!(self.tokens, TokenType::EOL) {
            self.tokens.advance();
            while check_token!(self.tokens, TokenType::EOL) {
                self.tokens.advance();
            }
            Ok(())
        } else if self.tokens.at_end() {
            // Do nothing
            Ok(())
        } else {
            Err(ParseError::Expected {
                expected: TokenType::EOL,
                received: self.peek()?,
            })
        }
    }

    fn parse_identifier(&mut self) -> Result<Expression> {
        let mut token_string = "".to_owned();
        let mut start = 0;
        if match_token!(self.tokens, TokenType::ScopeSeparator) {
            start = self.mark_start();
            token_string = "::".to_string();
        }

        self.consume_token(TokenType::Identifier)?;
        let start_token = self.last().lexeme;
        // TODO: redo matching beginning of identifier
        if start == 0 {
            start = self.mark_start();
        }
        token_string.push_str(start_token.as_str());

        while !self.tokens.at_end() && match_token!(self.tokens, TokenType::ScopeSeparator) {
            self.consume_token(TokenType::Identifier)?;
            let start = self.last().lexeme;
            token_string.push_str("::");
            token_string.push_str(start.as_str());
        }
        let end = self.mark_end();

        if matches!(token_string.to_lowercase().as_str(), "z" | "a" | "f")
            && match_token!(self.tokens, TokenType::Colon)
        {
            // TODO: Handle addressing modes?
            self.parse_expression()
        } else if matches!(token_string.to_lowercase().as_str(), "y" | "x" | "a" | "s") {
            // TODO: Available registers should rely on target processor
            // Reserved registers
            Ok(Expression {
                kind: ExpressionKind::Literal(token_string),
                span: Span::new(start, end),
            })
        } else if match_token!(self.tokens, TokenType::LeftParen) {
            let params = self.parse_parameters()?;
            self.consume_token(TokenType::RightParen)?;
            let end = self.mark_end();
            Ok(Expression {
                kind: ExpressionKind::Call(token_string.to_string(), params),
                span: Span::new(start, end),
            })
        } else {
            Ok(Expression {
                kind: ExpressionKind::Identifier(token_string.to_string()),
                span: Span::new(start, end),
            })
        }
    }

    fn parse_unnamed_label_reference(&mut self) -> Result<Expression> {
        let start = self.mark_start();

        self.consume_token(TokenType::UnnamedLabelReference)?;

        let distance_abs = self.last().lexeme.trim_start_matches(':').len() as i8;
        let distance = match self.last().lexeme.chars().nth(1) {
            Some('+') | Some('>') => distance_abs,
            Some('-') | Some('<') => -distance_abs,
            _ => 0,
        };

        let end = self.mark_end();

        Ok(Expression {
            kind: ExpressionKind::UnnamedLabelReference(distance),
            span: Span::new(start, end),
        })
    }

    #[inline]
    fn parse_statement_block(&mut self, macro_end: &[&str]) -> Result<Vec<Statement>> {
        let mut commands: Vec<Statement> = vec![];
        while !self.tokens.at_end() {
            if check_token!(self.tokens, TokenType::Macro) {
                let m = self.peek()?.lexeme;
                if macro_end.contains(&m.as_str()) {
                    self.tokens.advance();
                    return Ok(commands);
                }
            }
            if let Some(line) = self.parse_line()? {
                commands.push(line);
            }
        }

        Err(ParseError::Expected {
            received: self.peek()?,
            expected: TokenType::Macro,
        })
    }

    fn parse_token_list(&mut self, terminator: TokenType) -> Result<Expression> {
        let start = self.mark_start();
        let mut tokens = vec![];
        while !self.tokens.at_end() && self.tokens.peek().unwrap().token_type != terminator {
            let tok = self.tokens.advance().unwrap();
            if matches!(tok.token_type, TokenType::Extract) {
                self.parse_extract()?; // TODO: add into token list
            } else {
                tokens.push(tok);
            }
        }
        let end = self.mark_end();

        Ok(Expression {
            kind: ExpressionKind::TokenList(tokens),
            span: Span::new(start, end),
        })
    }

    fn parse_extract(&mut self) -> Result<Expression> {
        let variant = self.last();
        let start = self.mark_start();
        self.consume_token(TokenType::LeftParen)?;
        let left = self.parse_token_list(TokenType::Comma)?;
        self.consume_token(TokenType::Comma)?;
        let right = self.parse_token_list(TokenType::RightParen)?;
        self.consume_token(TokenType::RightParen)?;
        let end = self.mark_end();

        Ok(Expression {
            kind: ExpressionKind::Extract(variant, Box::new(left), Box::new(right)),
            span: Span::new(start, end),
        })
    }

    fn parse_import_export_list(&mut self) -> Result<Vec<ImportExport>> {
        let mut exports = vec![];
        exports.push(self.parse_import_export()?);
        while match_token!(self.tokens, TokenType::Comma) {
            exports.push(self.parse_import_export()?);
        }

        Ok(exports)
    }

    fn parse_import_export(&mut self) -> Result<ImportExport> {
        let start = self.mark_start();
        self.consume_token(TokenType::Identifier)?;
        let name = self.last();

        let far = if match_token!(self.tokens, TokenType::Colon) {
            let ident = self.consume_token(TokenType::Identifier)?;
            ident.lexeme == "far"
        } else {
            false
        };

        // Ignore value for now
        let value = if match_token!(self.tokens, TokenType::Equal | TokenType::ConstAssign) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let end = self.mark_end();

        Ok(ImportExport {
            name,
            far,
            value,
            span: Span::new(start, end),
        })
    }

    // Return current position
    #[inline]
    fn mark_start(&self) -> usize {
        self.tokens.previous().unwrap().span.start
    }

    #[inline]
    fn mark_end(&self) -> usize {
        self.mark_start() + self.tokens.previous().unwrap().lexeme.len()
    }

    #[inline]
    fn last(&self) -> Token {
        self.tokens.previous().unwrap()
    }

    #[inline]
    fn peek(&self) -> Result<Token> {
        if let Some(token) = self.tokens.peek() {
            Ok(token)
        } else {
            Err(ParseError::EOF)
        }
    }

    fn consume_token(&mut self, type_: TokenType) -> Result<Token> {
        if self.peek()?.token_type == type_ {
            Ok(self.tokens.advance().unwrap())
        } else {
            Err(ParseError::Expected {
                expected: type_,
                received: self.peek()?,
            })
        }
    }

    fn error_recovery(&mut self) {
        loop {
            if self.tokens.at_end() {
                break;
            }
            if let Some(token) = self.tokens.peek() {
                if token.token_type == TokenType::EOL {
                    break;
                } else {
                    self.tokens.advance();
                }
            } else {
                break;
            }
        }
    }
}
