use std::process::Command;
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
}

#[derive(Debug, Clone)]
pub enum Expression {
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
pub struct Instruction {
    pub mnemonic: String,
    pub parameters: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub enum Operation {
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
                },
                _ => None,
            };

            return operation;
        }
        panic!("Syntax Error: Unexpected token {:#?}", self.tokens.peek());
    }

    fn parse_macro(&mut self) -> Option<Operation> {
        if let Some(ident) = match_token2!(self.tokens, TokenType::Macro(i) => i) {
            match ident.to_lowercase().as_str() {
                ".macpack" => {
                    let pack = consume_token2!(self.tokens, TokenType::Identifier(i) => i, "Expected Identifier");
                    self.consume_newline();
                    return Some(Operation::MacroPack(pack));
                }
                ".include" => {
                    let path =
                        consume_token2!(self.tokens, TokenType::String(s) => s, "Expected String");
                    self.consume_newline();
                    // println!("Include {path}");
                    return Some(Operation::Include(path));
                }
                ".setcpu" => {
                    let cpu =
                        consume_token2!(self.tokens, TokenType::String(s) => s, "Expected String");
                    consume_token!(self.tokens, TokenType::EOL, "Expected EOL");

                    return Some(Operation::ControlCommand(ControlCommand {
                        control_type: ControlCommandType::SetCPU(cpu),
                    }));
                }
                ".segment" => {
                    let segment =
                        consume_token2!(self.tokens, TokenType::String(s) => s, "Expected String");
                    self.consume_newline();

                    return Some(Operation::ControlCommand(ControlCommand {
                        control_type: ControlCommandType::Segment(segment),
                    }));
                }
                ".proc" => {
                    let ident = consume_token2!(self.tokens, TokenType::Identifier(s) => s, "Expected Identifier");
                    self.consume_newline();
                    let mut commands: Vec<Option<Operation>> = vec![];
                    while !self.tokens.at_end() {
                        if let Some(m) = check_token2!(self.tokens, TokenType::Macro(m) => m) {
                            if m == ".endproc" {
                                self.tokens.advance();
                                return Some(Operation::ControlCommand(ControlCommand {
                                    control_type: ControlCommandType::Procedure(ident, commands.iter().filter(|c| c.is_some()).cloned().map(|c| c.unwrap()).collect()),
                                }));
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
                                return Some(Operation::Scope(ident, commands.iter().filter(|c| c.is_some()).cloned().map(|c| c.unwrap()).collect()));
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
                    self.consume_newline();

                    return Some(Operation::ControlCommand(ControlCommand {
                        control_type: ControlCommandType::Reserve(right),
                    }));
                }
                ".zeropage" => {
                    self.consume_newline();

                    return Some(Operation::ControlCommand(ControlCommand {
                        control_type: ControlCommandType::Segment("zeropage".to_string()),
                    }));
                }
                _ => panic!("Unexpected Macro: {}", ident),
            }
        }

        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Option<Operation> {
        if let Some(token) = self.tokens.peek() {
            if let TokenType::Identifier(_) = token.token_type.clone() {
                self.tokens.advance();
                if check_token!(self.tokens, TokenType::Equal) {
                    consume_token!(self.tokens, TokenType::Equal, "Expected Equal");
                    let operation = Operation::ConstantAssign(ConstantAssign {
                        name: token,
                        value: self.parse_expression(),
                    });

                    self.consume_newline();

                    return Some(operation);
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
            let parameters = self.parse_parameters();

            return Some(Operation::Instruction(Instruction {
                mnemonic,
                parameters,
            }));
        }
        panic!("Syntax Error: {:?}", self.tokens.peek());
    }

    fn parse_label(&mut self) -> Operation {
        let name = self.tokens.previous().unwrap();
        consume_token!(self.tokens, TokenType::Colon, "Expected Colon");
        Operation::Label(name)
    }

    fn parse_macro_invocation(&mut self) -> Option<Operation> {
        let name = self.tokens.previous().unwrap();
        let parameters = self.parse_parameters();
        Some(Operation::MacroInvocation(MacroInvocation {
            name,
            parameters,
        }))
    }

    fn parse_expression(&mut self) -> Expression {
        self.parse_expr0()
    }

    fn parse_expr0(&mut self) -> Expression {
        if match_token!(self.tokens, TokenType::Not) {
            Expression::Not(Box::from(self.parse_expr0()))
        } else {
            self.parse_expr1()
        }
    }

    fn parse_expr1(&mut self) -> Expression {
        let mut root = self.parse_expr2();
        while match_token!(self.tokens, TokenType::Or) {
            root = Expression::Or(Box::from(root), Box::from(self.parse_expr2()));
        }
        root
    }

    fn parse_expr2(&mut self) -> Expression {
        let mut root = self.parse_bool_expr();
        while match_token!(self.tokens, TokenType::And|TokenType::Xor) {
            match self.tokens.previous().unwrap().token_type {
                TokenType::And => {
                    root = Expression::And(Box::from(root), Box::from(self.parse_expr2()));
                }
                TokenType::Xor => {
                    root = Expression::Xor(Box::from(root), Box::from(self.parse_expr2()));
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

        while match_token!(self.tokens, TokenType::Equal|TokenType::NotEqual|TokenType::LessThan|TokenType::GreaterThan|TokenType::LessThanEq|TokenType::GreaterThanEq) {
            root = Expression::Comparison(self.tokens.previous().unwrap().token_type, Box::from(root), Box::from(self.parse_simple_expression()));
        }

        root
    }

    fn parse_simple_expression(&mut self) -> Expression {
        let mut root = self.parse_term();

        while match_token!(self.tokens, TokenType::Plus|TokenType::Minus|TokenType::BitwiseOr) {
            root = Expression::SimpleExpression(self.tokens.previous().unwrap().token_type, Box::from(root), Box::from(self.parse_term()));
        }

        root
    }

    fn parse_term(&mut self) -> Expression {
        let mut root = self.parse_factor();

        while match_token!(self.tokens, TokenType::Multiply|TokenType::Divide|TokenType::Mod|TokenType::BitwiseAnd|TokenType::BitwiseXor|TokenType::ShiftLeft|TokenType::ShiftRight) {
            root = Expression::Term(self.tokens.previous().unwrap().token_type, Box::from(root), Box::from(self.parse_factor()));
        }

        root
    }

    fn parse_factor(&mut self) -> Expression {
        self.parse_unary()
    }

    fn parse_unary(&mut self) -> Expression {
        if match_token!(self.tokens, TokenType::Hash) {
            return Expression::Immediate(Box::from(self.parse_unary()));
        }
        if match_token!(self.tokens, TokenType::Plus) {
            return Expression::Unary(TokenType::Plus, Box::from(self.parse_unary()));
        }
        if match_token!(self.tokens, TokenType::Minus) {
            return Expression::Unary(TokenType::Minus, Box::from(self.parse_unary()));
        }
        if match_token!(self.tokens, TokenType::BitwiseNot) {
            return Expression::Unary(TokenType::BitwiseNot, Box::from(self.parse_unary()));
        }
        if match_token!(self.tokens, TokenType::LessThan) {
            return Expression::Unary(TokenType::LessThan, Box::from(self.parse_unary()));
        }
        if match_token!(self.tokens, TokenType::GreaterThan) {
            return Expression::Unary(TokenType::GreaterThan, Box::from(self.parse_unary()));
        }
        if match_token!(self.tokens, TokenType::Caret) {
            return Expression::Unary(TokenType::Caret, Box::from(self.parse_unary()));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Expression {
        if let Some(num) = check_token2!(self.tokens, TokenType::Number(i) => i) {
            self.tokens.advance();
            return Expression::Literal(num);
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
            return Expression::Group(Box::from(expr));
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
        if check_token!(self.tokens, TokenType::ScopeSeparator) {
            self.tokens.advance();
            token_string = "::".to_string();
        }

        let start =
            consume_token2!(self.tokens, TokenType::Identifier(i) => i, "Expected Identifier");
        token_string.push_str(start.as_str());

        while !self.tokens.at_end() && match_token!(self.tokens, TokenType::ScopeSeparator) {
            let start =
                consume_token2!(self.tokens, TokenType::Identifier(i) => i, "Expected Identifier");
            token_string.push_str("::");
            token_string.push_str(start.as_str());
        }

        Expression::Literal(token_string.to_string())
    }
}
