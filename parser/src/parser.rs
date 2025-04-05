use crate::tokenizer::{Token, TokenType};

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

macro_rules! match_token2 {
    ($stream:expr, $token:pat => $out:ident) => {
        if let Some(tok) = $stream.peek() {
            match tok.token_type {
                $token => {
                    $stream.advance();
                    Some($out)
                }
                _ => None,
            }
        } else {
            None
        }
    };
}

macro_rules! consume_token {
    ($stream:expr, $token:pat, $error:literal) => {
        if let Some(Token {
            token_type: $token, ..
        }) = $stream.peek()
        {
            $stream.advance();
        } else {
            panic!("Syntax Error: {} {:#?}", $error, $stream.peek());
        }
    };
}

macro_rules! consume_token2 {
    ($stream:expr, $token:pat => $out:ident, $error:literal) => {
        if let Some(Token {
            token_type: $token, ..
        }) = $stream.peek()
        {
            $stream.advance();
            $out
        } else {
            panic!("Syntax Error: {} {:#?}", $error, $stream.peek());
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

macro_rules! check_token2 {
    ($stream:expr, $token:pat => $out:ident) => {
        if let Some(tok) = $stream.peek() {
            match tok.token_type {
                $token => Some($out),
                _ => None,
            }
        } else {
            None
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start_offset: usize,
    pub end_offset: usize,
}

impl Span {
    fn new(start_offset: usize, end_offset: usize) -> Span {
        Self {
            start_offset,
            end_offset,
        }
    }
}

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
        let token = self.previous();
        // println!("Advancing {:#?}", token);
        token
    }

    pub fn peek(&self) -> Option<Token> {
        if !self.at_end() {
            // println!("Peeking {:#?}", self.tokens[self.position]);
            return Some(self.tokens[self.position].clone());
        }
        None
    }

    pub fn previous(&self) -> Option<Token> {
        if self.position > 0 {
            Some(self.tokens[self.position - 1].clone())
        } else {
            None
        }
    }

    fn at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }
}

#[derive(Debug, Clone)]
pub struct ConstantAssign {
    pub name: Token,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone)]
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
    SimpleExpression(TokenType, Box<Expression>, Box<Expression>),
    Term(TokenType, Box<Expression>, Box<Expression>),
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub mnemonic: String,
    pub parameters: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub enum OperationKind {
    ConstantAssign(ConstantAssign),
    Include(String),
    Label(Token),
    Instruction(Instruction),
    ControlCommand(ControlCommand),
    MacroInvocation(MacroInvocation),
    MacroPack(String),
    Scope(String, Vec<Operation>),
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub kind: OperationKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ControlCommandType {
    Procedure(String, Vec<Operation>),
    Macro,
    Scope(String, Vec<Operation>),
    Enum,
    SetCPU(String),
    Segment(String),
    Reserve(Expression),
}

#[derive(Debug, Clone)]
pub struct ControlCommand {
    pub control_type: ControlCommandType,
}

#[derive(Debug, Clone)]
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

    pub fn parse(&mut self) -> Vec<Operation> {
        let mut operations = vec![];

        while !self.tokens.at_end() {
            if let Some(operation) = self.parse_command() {
                // println!("{:#?}", operation);
                operations.push(operation);
            }
        }

        operations
    }

    fn parse_command(&mut self) -> Option<Operation> {
        if let Some(token) = self.tokens.peek() {
            let operation = match token.token_type {
                TokenType::Instruction(_) => self.parse_instruction(),
                TokenType::Macro(_) => self.parse_macro(),
                TokenType::Identifier(_) => self.parse_assignment(),
                TokenType::EOL => {
                    self.tokens.advance();
                    None
                }
                _ => None,
            };

            return operation;
        }
        panic!("Syntax Error: Unexpected token {:#?}", self.tokens.peek());
    }

    fn parse_macro(&mut self) -> Option<Operation> {
        if let Some(ident) = match_token2!(self.tokens, TokenType::Macro(i) => i) {
            let start = self.mark_start();
            match ident.to_lowercase().as_str() {
                ".macpack" => {
                    let pack = consume_token2!(self.tokens, TokenType::Identifier(i) => i, "Expected Identifier");
                    let end = self.mark_end();
                    self.consume_newline();
                    return Some(Operation {
                        kind: OperationKind::MacroPack(pack),
                        span: Span::new(start, end),
                    });
                }
                ".include" => {
                    let path =
                        consume_token2!(self.tokens, TokenType::String(s) => s, "Expected String");
                    let end = self.mark_end();
                    self.consume_newline();
                    // println!("Include {path}");
                    return Some(Operation {
                        kind: OperationKind::Include(path),
                        span: Span::new(start, end),
                    });
                }
                ".setcpu" => {
                    let cpu =
                        consume_token2!(self.tokens, TokenType::String(s) => s, "Expected String");
                    let end = self.mark_end();
                    self.consume_newline();

                    return Some(Operation {
                        kind: OperationKind::ControlCommand(ControlCommand {
                            control_type: ControlCommandType::SetCPU(cpu),
                        }),
                        span: Span::new(start, end),
                    });
                }
                ".segment" => {
                    let segment =
                        consume_token2!(self.tokens, TokenType::String(s) => s, "Expected String");
                    let end = self.mark_end();
                    self.consume_newline();

                    return Some(Operation {
                        kind: OperationKind::ControlCommand(ControlCommand {
                            control_type: ControlCommandType::Segment(segment),
                        }),
                        span: Span::new(start, end),
                    });
                }
                ".proc" => {
                    let ident = consume_token2!(self.tokens, TokenType::Identifier(s) => s, "Expected Identifier");
                    self.consume_newline();
                    let mut commands: Vec<Option<Operation>> = vec![];
                    while !self.tokens.at_end() {
                        if let Some(m) = check_token2!(self.tokens, TokenType::Macro(m) => m) {
                            if m == ".endproc" {
                                self.tokens.advance();
                                let end = self.mark_end();
                                return Some(Operation {
                                    kind: OperationKind::ControlCommand(ControlCommand {
                                        control_type: ControlCommandType::Procedure(
                                            ident,
                                            commands
                                                .iter()
                                                .filter(|c| c.is_some())
                                                .cloned()
                                                .map(|c| c.unwrap())
                                                .collect(),
                                        ),
                                    }),
                                    span: Span::new(start, end),
                                });
                            }
                        }
                        commands.push(self.parse_command());
                    }
                    panic!(
                        "Syntax Error: Unexpected token {:#?}, expected .endproc",
                        self.tokens.peek()
                    );
                }
                ".scope" => {
                    let ident = consume_token2!(self.tokens, TokenType::Identifier(s) => s, "Expected Identifier");
                    self.consume_newline();
                    let mut commands: Vec<Option<Operation>> = vec![];
                    while !self.tokens.at_end() {
                        if let Some(m) = check_token2!(self.tokens, TokenType::Macro(m) => m) {
                            if m == ".endscope" {
                                self.tokens.advance();
                                let end = self.mark_end();
                                return Some(Operation {
                                    kind: OperationKind::Scope(
                                        ident,
                                        commands
                                            .iter()
                                            .filter(|c| c.is_some())
                                            .cloned()
                                            .map(|c| c.unwrap())
                                            .collect(),
                                    ),
                                    span: Span::new(start, end),
                                });
                            }
                        }
                        commands.push(self.parse_command());
                    }
                    panic!(
                        "Syntax Error: Unexpected token {:#?}, expected .endproc",
                        self.tokens.peek()
                    );
                }
                ".res" => {
                    let right = self.parse_expression();
                    let end = self.mark_end();
                    self.consume_newline();

                    return Some(Operation {
                        kind: OperationKind::ControlCommand(ControlCommand {
                            control_type: ControlCommandType::Reserve(right),
                        }),
                        span: Span::new(start, end),
                    });
                }
                ".zeropage" => {
                    let end = self.mark_end();
                    self.consume_newline();

                    return Some(Operation {
                        kind: OperationKind::ControlCommand(ControlCommand {
                            control_type: ControlCommandType::Segment("zeropage".to_string()),
                        }),
                        span: Span::new(start, end),
                    });
                }
                _ => panic!("Unexpected Macro: {}", ident),
            }
        }

        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Option<Operation> {
        if let Some(token) = self.tokens.peek() {
            if let TokenType::Identifier(_) = token.token_type.clone() {
                let start = self.tokens.previous()?.index;
                self.tokens.advance();
                if check_token!(self.tokens, TokenType::Equal) {
                    consume_token!(self.tokens, TokenType::Equal, "Expected Equal");
                    let value = self.parse_expression();
                    let end = self.tokens.previous()?.index;
                    let operation = OperationKind::ConstantAssign(ConstantAssign {
                        name: token,
                        value,
                        span: Span::new(start, end),
                    });

                    self.consume_newline();

                    return Some(Operation {
                        kind: operation,
                        span: Span::new(start, end),
                    });
                }
                if check_token!(self.tokens, TokenType::Colon) {
                    return Some(self.parse_label());
                }
                return self.parse_macro_invocation();
            }
        }
        self.parse_instruction()
    }

    fn parse_instruction(&mut self) -> Option<Operation> {
        if let Some(mnemonic) = match_token2!(self.tokens, TokenType::Instruction(i) => i) {
            let start = self.mark_start();
            let parameters = self.parse_parameters();
            let end = self.mark_end();

            return Some(Operation {
                kind: OperationKind::Instruction(Instruction {
                    mnemonic,
                    parameters,
                }),
                span: Span::new(start, end),
            });
        }
        panic!("Syntax Error: {:?}", self.tokens.peek());
    }

    fn parse_label(&mut self) -> Operation {
        let start = self.mark_start();
        let name = self.tokens.previous().unwrap();
        consume_token!(self.tokens, TokenType::Colon, "Expected Colon");
        let end = self.mark_end();
        Operation {
            kind: OperationKind::Label(name),
            span: Span::new(start, end),
        }
    }

    fn parse_macro_invocation(&mut self) -> Option<Operation> {
        let start = self.mark_start();
        let name = self.tokens.previous().unwrap();
        let parameters = self.parse_parameters();
        let end = self.mark_end();
        Some(Operation {
            kind: OperationKind::MacroInvocation(MacroInvocation { name, parameters }),
            span: Span::new(start, end),
        })
    }

    fn parse_expression(&mut self) -> Expression {
        self.parse_expr0()
    }

    fn parse_expr0(&mut self) -> Expression {
        let start = self.mark_start();
        if match_token!(self.tokens, TokenType::Not) {
            let expr = self.parse_expr0();
            let end = self.mark_end();
            Expression {
                kind: ExpressionKind::Not(Box::from(expr)),
                span: Span::new(start, end),
            }
        } else {
            self.parse_expr1()
        }
    }

    fn parse_expr1(&mut self) -> Expression {
        let mut root = self.parse_expr2();
        while match_token!(self.tokens, TokenType::Or) {
            let right = self.parse_expr2();
            root = Expression {
                kind: ExpressionKind::Or(Box::from(root.clone()), Box::from(right.clone())),
                span: Span::new(root.span.start_offset, right.span.end_offset),
            };
        }
        root
    }

    fn parse_expr2(&mut self) -> Expression {
        let mut root = self.parse_bool_expr();
        while match_token!(self.tokens, TokenType::And | TokenType::Xor) {
            let right = self.parse_expr2();
            match self.tokens.previous().unwrap().token_type {
                TokenType::And => {
                    root = Expression {
                        kind: ExpressionKind::And(
                            Box::from(root.clone()),
                            Box::from(right.clone()),
                        ),
                        span: Span::new(root.span.start_offset, right.span.end_offset),
                    };
                }
                TokenType::Xor => {
                    root = Expression {
                        kind: ExpressionKind::Xor(
                            Box::from(root.clone()),
                            Box::from(right.clone()),
                        ),
                        span: Span::new(root.span.start_offset, right.span.end_offset),
                    };
                }
                _ => {
                    unreachable!("NANI")
                }
            }
        }

        root
    }

    fn parse_bool_expr(&mut self) -> Expression {
        let mut root = self.parse_simple_expression();

        while match_token!(
            self.tokens,
            TokenType::Equal
                | TokenType::NotEqual
                | TokenType::LessThan
                | TokenType::GreaterThan
                | TokenType::LessThanEq
                | TokenType::GreaterThanEq
        ) {
            let right = self.parse_simple_expression();
            root = Expression {
                kind: ExpressionKind::Comparison(
                    self.tokens.previous().unwrap().token_type,
                    Box::from(root.clone()),
                    Box::from(right.clone()),
                ),
                span: Span::new(root.span.start_offset, right.span.end_offset),
            };
        }

        root
    }

    fn parse_simple_expression(&mut self) -> Expression {
        let mut root = self.parse_term();

        while match_token!(
            self.tokens,
            TokenType::Plus | TokenType::Minus | TokenType::BitwiseOr
        ) {
            let right = self.parse_term();
            root = Expression {
                kind: ExpressionKind::SimpleExpression(
                    self.tokens.previous().unwrap().token_type,
                    Box::from(root.clone()),
                    Box::from(right.clone()),
                ),
                span: Span::new(root.span.start_offset, root.span.end_offset),
            };
        }

        root
    }

    fn parse_term(&mut self) -> Expression {
        let mut root = self.parse_factor();

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
            let right = self.parse_factor();

            root = Expression {
                kind: ExpressionKind::Term(
                    self.tokens.previous().unwrap().token_type,
                    Box::from(root.clone()),
                    Box::from(right.clone()),
                ),
                span: Span::new(root.span.start_offset, right.span.end_offset),
            };
        }

        root
    }

    fn parse_factor(&mut self) -> Expression {
        self.parse_unary()
    }

    fn parse_unary(&mut self) -> Expression {
        let start = self.mark_start();
        if match_token!(self.tokens, TokenType::Hash) {
            let right = self.parse_unary();
            let end = self.mark_end();
            return Expression {
                kind: ExpressionKind::Immediate(Box::from(right)),
                span: Span::new(start, end),
            };
        }
        if match_token!(self.tokens, TokenType::Plus) {
            let right = self.parse_unary();
            let end = self.mark_end();

            return Expression {
                kind: ExpressionKind::Unary(TokenType::Plus, Box::from(right)),
                span: Span::new(start, end),
            };
        }
        if match_token!(self.tokens, TokenType::Minus) {
            let right = self.parse_unary();
            let end = self.mark_end();

            return Expression {
                kind: ExpressionKind::Unary(TokenType::Minus, Box::from(right)),
                span: Span::new(start, end),
            };
        }
        if match_token!(self.tokens, TokenType::BitwiseNot) {
            let right = self.parse_unary();
            let end = self.mark_end();

            return Expression {
                kind: ExpressionKind::Unary(TokenType::BitwiseNot, Box::from(right)),
                span: Span::new(start, end),
            };
        }
        if match_token!(self.tokens, TokenType::LessThan) {
            let right = self.parse_unary();
            let end = self.mark_end();

            return Expression {
                kind: ExpressionKind::Unary(TokenType::LessThan, Box::from(right)),
                span: Span::new(start, end),
            };
        }
        if match_token!(self.tokens, TokenType::GreaterThan) {
            let right = self.parse_unary();
            let end = self.mark_end();

            return Expression {
                kind: ExpressionKind::Unary(TokenType::GreaterThan, Box::from(right)),
                span: Span::new(start, end),
            };
        }
        if match_token!(self.tokens, TokenType::Caret) {
            let right = self.parse_unary();
            let end = self.mark_end();

            return Expression {
                kind: ExpressionKind::Unary(TokenType::Caret, Box::from(right)),
                span: Span::new(start, end),
            };
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Expression {
        let start = self.mark_start();
        if let Some(num) = match_token2!(self.tokens, TokenType::Number(i) => i) {
            let end = self.mark_end();
            return Expression {
                kind: ExpressionKind::Literal(num),
                span: Span::new(start, end),
            };
        }
        if check_token!(self.tokens, TokenType::ScopeSeparator)
            || check_token!(self.tokens, TokenType::Identifier(_))
        {
            return self.parse_identifier();
        }
        if check_token!(self.tokens, TokenType::LeftParen) {
            self.tokens.advance();
            let expr = self.parse_expression();
            consume_token!(self.tokens, TokenType::RightParen, "Expected )");
            let end = self.mark_end();
            return Expression {
                kind: ExpressionKind::Group(Box::from(expr)),
                span: Span::new(start, end),
            };
        }
        panic!("Syntax Error: {:?}", self.tokens.peek());
    }

    fn parse_parameters(&mut self) -> Vec<Expression> {
        let mut parameters = vec![];
        if !check_token!(self.tokens, TokenType::EOL) {
            parameters.push(self.parse_expression());
            while !self.tokens.at_end() && !check_token!(self.tokens, TokenType::EOL) {
                consume_token!(self.tokens, TokenType::Comma, "Expected Comma");
                parameters.push(self.parse_expression());
            }
        }
        parameters
    }

    fn consume_newline(&mut self) {
        if check_token!(self.tokens, TokenType::EOL) {
            self.tokens.advance();
        } else if self.tokens.at_end() {
            // Do nothing
        } else {
            panic!(
                "Syntax Error: Expected newline, got {:#?}",
                self.tokens.peek()
            );
        }
    }

    fn parse_identifier(&mut self) -> Expression {
        let mut token_string = "".to_owned();
        let mut start = 0;
        if match_token!(self.tokens, TokenType::ScopeSeparator) {
            start = self.mark_start();
            token_string = "::".to_string();
        } else {
            start = self.mark_start();
        }

        let start_token =
            consume_token2!(self.tokens, TokenType::Identifier(i) => i, "Expected Identifier");
        token_string.push_str(start_token.as_str());

        while !self.tokens.at_end() && match_token!(self.tokens, TokenType::ScopeSeparator) {
            let start =
                consume_token2!(self.tokens, TokenType::Identifier(i) => i, "Expected Identifier");
            token_string.push_str("::");
            token_string.push_str(start.as_str());
        }
        let end = self.mark_end();

        Expression {
            kind: ExpressionKind::Literal(token_string.to_string()),
            span: Span::new(start, end),
        }
    }

    // Return current position
    #[inline]
    fn mark_start(&self) -> usize {
        self.tokens.previous().unwrap().index
    }

    #[inline]
    fn mark_end(&self) -> usize {
        self.mark_start() + self.tokens.previous().unwrap().lexeme.len()
    }
}
