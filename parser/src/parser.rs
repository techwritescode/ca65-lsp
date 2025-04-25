use crate::tokenizer::{Token, TokenType};
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
        if let Ok(token) = self.previous() {
            Some(token)
        } else {
            None
        }
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
    Group(Box<Expression>),
    UnaryPositive(Box<Expression>),
    Math(TokenType, Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Xor(Box<Expression>, Box<Expression>),
    Comparison(TokenType, Box<Expression>, Box<Expression>),
    SimpleExpression(Token, Box<Expression>, Box<Expression>),
    Term(TokenType, Box<Expression>, Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub mnemonic: String,
    pub parameters: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineKind {
    ConstantAssign(ConstantAssign),
    Include(Token),
    Label(Token),
    Instruction(Instruction),
    Procedure(Token, Vec<Line>),
    Macro,
    Enum,
    SetCPU(String),
    Segment(String),
    Reserve(Expression),

    MacroInvocation(MacroInvocation),
    MacroPack(String),
    Feature(String),
    Scope(String, Vec<Line>),
    IncludeBinary(Token),
    MacroDefinition(Token, Vec<Token>, Vec<Line>),
    Data(Vec<Expression>),
    Org(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub kind: LineKind,
    pub span: Span,
}

pub type Ast = Vec<Line>;

#[derive(Debug, Clone, PartialEq)]
pub enum ControlCommandType {}

#[derive(Debug, Clone, PartialEq)]
pub struct ControlCommand {
    pub control_type: ControlCommandType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroInvocation {
    pub name: Token,
    pub parameters: Vec<Expression>,
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

    pub fn parse(&mut self) -> Result<Ast> {
        let mut lines = vec![];

        while !self.tokens.at_end() {
            if let Some(operation) = self.parse_line()? {
                lines.push(operation);
            }
        }

        Ok(lines)
    }

    fn parse_line(&mut self) -> Result<Option<Line>> {
        if let Some(token) = self.tokens.peek() {
            let operation = match token.token_type {
                TokenType::Macro => self.parse_macro(),
                TokenType::Identifier => Ok(Some(self.parse_assignment()?)),
                TokenType::Instruction => Ok(Some(self.parse_instruction()?)),
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

    fn parse_macro(&mut self) -> Result<Option<Line>> {
        if match_token!(self.tokens, TokenType::Macro) {
            let mac = self.tokens.previous()?;
            let start = self.mark_start();
            let ident = &mac.lexeme;
            return match ident.to_lowercase().as_str() {
                ".macpack" => {
                    self.consume_token(TokenType::Identifier)?;
                    let pack = self.last().lexeme;
                    let end = self.mark_end();
                    self.consume_newline()?;
                    Ok(Some(Line {
                        kind: LineKind::MacroPack(pack),
                        span: Span::new(start, end),
                    }))
                }
                ".feature" => {
                    self.consume_token(TokenType::Identifier)?;
                    let feature = self.last().lexeme;
                    let end = self.mark_end();
                    self.consume_newline()?;
                    Ok(Some(Line {
                        kind: LineKind::Feature(feature),
                        span: Span::new(start, end),
                    }))
                }
                ".include" => {
                    self.consume_token(TokenType::String)?;
                    let path = self.last();
                    let end = self.mark_end();
                    self.consume_newline()?;
                    Ok(Some(Line {
                        kind: LineKind::Include(path),
                        span: Span::new(start, end),
                    }))
                }
                ".incbin" => {
                    self.consume_token(TokenType::String)?;
                    let path = self.last();
                    let end = self.mark_end();
                    self.consume_newline()?;
                    Ok(Some(Line {
                        kind: LineKind::IncludeBinary(path),
                        span: Span::new(start, end),
                    }))
                }
                ".setcpu" => {
                    self.consume_token(TokenType::String)?;
                    let cpu = self.last().lexeme;
                    let end = self.mark_end();
                    self.consume_newline()?;

                    Ok(Some(Line {
                        kind: LineKind::SetCPU(cpu),
                        span: Span::new(start, end),
                    }))
                }
                ".org" => {
                    let address = self.consume_token(TokenType::Number)?;
                    let end = self.mark_end();
                    self.consume_newline()?;
                    
                    Ok(Some(Line{
                        kind: LineKind::Org(address.lexeme.clone()),
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
                    let segment = self.last().lexeme;
                    let end = self.mark_end();
                    self.consume_newline()?;

                    Ok(Some(Line {
                        kind: LineKind::Segment(segment),
                        span: Span::new(start, end),
                    }))
                }
                ".macro"|".mac" => Ok(Some(self.parse_macro_def()?)),
                ".proc" => {
                    self.consume_token(TokenType::Identifier)?;
                    let ident = self.last();
                    self.consume_newline()?;
                    let mut commands: Vec<Line> = vec![];
                    while !self.tokens.at_end() {
                        if check_token!(self.tokens, TokenType::Macro) {
                            let m = self.peek()?.lexeme;
                            if m == ".endproc" {
                                self.tokens.advance();
                                let end = self.mark_end();
                                return Ok(Some(Line {
                                    kind: LineKind::Procedure(ident, commands),
                                    span: Span::new(start, end),
                                }));
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
                ".scope" => {
                    self.consume_token(TokenType::Identifier)?;
                    let ident = self.last().lexeme;
                    self.consume_newline()?;
                    let mut commands: Vec<Option<Line>> = vec![];
                    while !self.tokens.at_end() {
                        if check_token!(self.tokens, TokenType::Macro) {
                            let m = self.peek()?.lexeme;
                            if m == ".endscope" {
                                self.tokens.advance();
                                let end = self.mark_end();
                                return Ok(Some(Line {
                                    kind: LineKind::Scope(
                                        ident,
                                        commands
                                            .iter()
                                            .filter(|c| c.is_some())
                                            .cloned()
                                            .map(|c| c.unwrap())
                                            .collect(),
                                    ),
                                    span: Span::new(start, end),
                                }));
                            }
                        }
                        commands.push(self.parse_line()?);
                    }
                    Err(ParseError::Expected {
                        received: self.peek()?,
                        expected: TokenType::Macro,
                    })
                }
                ".res" => {
                    let right = self.parse_expression()?;
                    let end = self.mark_end();
                    self.consume_newline()?;

                    Ok(Some(Line {
                        kind: LineKind::Reserve(right),
                        span: Span::new(start, end),
                    }))
                }
                ".zeropage" => {
                    let end = self.mark_end();
                    self.consume_newline()?;

                    Ok(Some(Line {
                        kind: LineKind::Segment("zeropage".to_string()),
                        span: Span::new(start, end),
                    }))
                }
                ".db"|".dw"|".byte"|".word" => {
                    let parameters = self.parse_parameters()?;
                    let end = self.mark_end();
                    self.consume_newline()?;

                    // TODO: Add kind
                    Ok(Some(Line {
                        kind: LineKind::Data(parameters),
                        span: Span::new(start, end),
                    }))
                }
                // Ignored for now
                ".index" | ".mem" => {
                    self.parse_parameters()?;
                    Ok(None)
                }, 
                _ => Err(ParseError::UnexpectedToken(mac)),
            };
        }

        Ok(Some(self.parse_assignment()?))
    }

    fn parse_macro_def(&mut self) -> Result<Line> {
        let start = self.mark_start();
        self.consume_token(TokenType::Identifier)?;
        let ident = self.last();
        let mut parameters = vec![];

        if !check_token!(self.tokens, TokenType::EOL|TokenType::EOF) {
            parameters.push(self.consume_token(TokenType::Identifier)?);
            while !self.tokens.at_end() && match_token!(self.tokens, TokenType::Comma) {
                parameters.push(self.consume_token(TokenType::Identifier)?);
            }
        }
        self.consume_newline()?;

        let mut commands: Vec<Line> = vec![];
        while !self.tokens.at_end() {
            if check_token!(self.tokens, TokenType::Macro) {
                let m = self.peek()?.lexeme;
                if m == ".endmacro" {
                    self.tokens.advance();
                    let end = self.mark_end();
                    return Ok(Line {
                        kind: LineKind::MacroDefinition(ident, parameters, commands),
                        span: Span::new(start, end),
                    });
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

    fn parse_assignment(&mut self) -> Result<Line> {
        if let Some(token) = self.tokens.peek() {
            if match_token!(self.tokens, TokenType::Identifier) {
                let start = self.last().span.start;
                if match_token!(self.tokens, TokenType::Equal) {
                    let value = self.parse_expression()?;
                    let end = self.tokens.previous()?.span.end;
                    let operation = LineKind::ConstantAssign(ConstantAssign {
                        name: token,
                        value,
                        span: Span::new(start, end),
                    });

                    self.consume_newline()?;

                    return Ok(Line {
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

    fn parse_instruction(&mut self) -> Result<Line> {
        if match_token!(self.tokens, TokenType::Instruction) {
            let mnemonic = self.last().lexeme;
            let start = self.mark_start();
            let parameters = self.parse_parameters()?;
            let end = self.mark_end();

            self.consume_newline()?;

            return Ok(Line {
                kind: LineKind::Instruction(Instruction {
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

    fn parse_label(&mut self) -> Result<Line> {
        let start = self.mark_start();
        let name = self.tokens.previous()?;
        self.consume_token(TokenType::Colon)?;

        let end = self.mark_end();
        Ok(Line {
            kind: LineKind::Label(name),
            span: Span::new(start, end),
        })
    }

    fn parse_macro_invocation(&mut self) -> Result<Line> {
        let start = self.mark_start();
        let name = self.tokens.previous()?;
        let parameters = self.parse_parameters()?;
        let end = self.mark_end();
        Ok(Line {
            kind: LineKind::MacroInvocation(MacroInvocation { name, parameters }),
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
        while match_token!(self.tokens, TokenType::And | TokenType::Xor) {
            let right = self.parse_expr2()?;
            match self.tokens.previous()?.token_type {
                TokenType::And => {
                    root = Expression {
                        kind: ExpressionKind::And(
                            Box::from(root.clone()),
                            Box::from(right.clone()),
                        ),
                        span: Span::new(root.span.start, right.span.end),
                    };
                }
                TokenType::Xor => {
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
                    self.tokens.previous().unwrap().token_type,
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
                    self.tokens.previous().unwrap().token_type,
                    Box::from(root.clone()),
                    Box::from(right.clone()),
                ),
                span: Span::new(root.span.start, right.span.end),
            };
        }

        Ok(root)
    }

    fn parse_factor(&mut self) -> Result<Expression> {
        self.parse_unary()
    }

    fn parse_unary(&mut self) -> Result<Expression> {
        let start = self.mark_start();
        if match_token!(self.tokens, TokenType::Hash) {
            let right = self.parse_unary()?;
            let end = self.mark_end();
            return Ok(Expression {
                kind: ExpressionKind::Immediate(Box::from(right)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::Plus) {
            let right = self.parse_unary()?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::Unary(TokenType::Plus, Box::from(right)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::Minus) {
            let right = self.parse_unary()?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::Unary(TokenType::Minus, Box::from(right)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::BitwiseNot) {
            let right = self.parse_unary()?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::Unary(TokenType::BitwiseNot, Box::from(right)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::LessThan) {
            let right = self.parse_unary()?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::Unary(TokenType::LessThan, Box::from(right)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::GreaterThan) {
            let right = self.parse_unary()?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::Unary(TokenType::GreaterThan, Box::from(right)),
                span: Span::new(start, end),
            });
        }
        if match_token!(self.tokens, TokenType::Caret) {
            let right = self.parse_unary()?;
            let end = self.mark_end();

            return Ok(Expression {
                kind: ExpressionKind::Unary(TokenType::Caret, Box::from(right)),
                span: Span::new(start, end),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression> {
        if match_token!(self.tokens, TokenType::Number) {
            let num = self.last().lexeme;
            let start = self.mark_start();
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
        if match_token!(self.tokens, TokenType::LeftParen) {
            let start = self.mark_start();
            let expr = self.parse_expression()?;
            self.consume_token(TokenType::RightParen)?;
            let end = self.mark_end();
            return Ok(Expression {
                kind: ExpressionKind::Group(Box::from(expr)),
                span: Span::new(start, end),
            });
        }
        Err(ParseError::UnexpectedToken(self.peek()?))
    }

    fn parse_parameters(&mut self) -> Result<Vec<Expression>> {
        let mut parameters = vec![];
        if !check_token!(self.tokens, TokenType::EOL) {
            parameters.push(self.parse_expression()?);
            while !self.tokens.at_end() && !check_token!(self.tokens, TokenType::EOL) {
                self.consume_token(TokenType::Comma)?;
                parameters.push(self.parse_expression()?);
            }
        }
        Ok(parameters)
    }

    fn consume_newline(&mut self) -> Result<()> {
        if check_token!(self.tokens, TokenType::EOL) {
            self.tokens.advance();
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

        Ok(Expression {
            kind: ExpressionKind::Literal(token_string.to_string()),
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
}
