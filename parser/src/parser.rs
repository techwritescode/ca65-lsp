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
        println!("Advancing {:#?}", token);
        token
    }

    pub fn peek(&self) -> Option<Token> {
        if !self.at_end() {
            println!("Peeking {:#?}", self.tokens[self.position]);
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

#[derive(Debug)]
pub struct ConstantAssign {
    pub name: Token,
    pub value: Expression,
}

#[derive(Debug)]
pub enum Expression {
    Immediate(Box<Expression>),
    Unary(TokenType, Box<Expression>),
    Literal(String),
    Group(Box<Expression>),
    UnaryPositive(Box<Expression>),
    Math(TokenType, Box<Expression>, Box<Expression>),
}

#[derive(Debug)]
pub struct Instruction {
    pub mnemonic: String,
    pub parameters: Vec<Expression>,
}

#[derive(Debug)]
pub enum Operation {
    ConstantAssign(ConstantAssign),
    Include(String),
    Label(Token),
    Instruction(Instruction),
    ControlCommand(ControlCommand),
    MacroInvocation(MacroInvocation),
    MacroPack(String),
    Scope(String, Vec<Option<Operation>>),
}

#[derive(Debug)]
pub enum ControlCommandType {
    Procedure(String),
    Macro,
    Scope,
    Enum,
    SetCPU(String),
    Segment(String),
    Reserve(Expression),
}

#[derive(Debug)]
pub struct ControlCommand {
    pub control_type: ControlCommandType,
}

#[derive(Debug)]
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
            match ident.as_str() {
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
                    while !self.tokens.at_end() {
                        if let Some(m) = check_token2!(self.tokens, TokenType::Macro(i) => i) {
                            if m == ".endproc" {
                                self.tokens.advance();
                                return Some(Operation::ControlCommand(ControlCommand {
                                    control_type: ControlCommandType::Procedure(ident),
                                }));
                            }
                        }
                        self.tokens.advance();
                    }
                    panic!(
                        "Syntax Error: Unexpected token {:#?}, expected .endproc",
                        self.tokens.peek()
                    );
                }
                ".scope" => {
                    let ident = consume_token2!(self.tokens, TokenType::Identifier(s) => s, "Expected Identifier");
                    self.consume_newline();
                    let mut commands = vec![];
                    while !self.tokens.at_end() {
                        if let Some(m) = check_token2!(self.tokens, TokenType::Macro(m) => m) {
                            if m == ".endscope" {
                                self.tokens.advance();
                                return Some(Operation::Scope(ident, commands));
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
        let parameters = self.parse_parameters();
        Some(Operation::MacroInvocation(MacroInvocation {
            name: self.tokens.previous().unwrap(),
            parameters,
        }))
    }

    fn parse_expression(&mut self) -> Expression {
        self.parse_math_expression()
    }

    fn parse_math_expression(&mut self) -> Expression {
        let left = self.parse_unary();

        if match_token!(self.tokens, TokenType::Multiply) {
            let right = self.parse_math_expression();
            return Expression::Math(TokenType::Multiply, Box::new(left), Box::new(right));
        }
        if match_token!(self.tokens, TokenType::Divide) {
            let right = self.parse_math_expression();
            return Expression::Math(TokenType::Divide, Box::new(left), Box::new(right));
        }
        if match_token!(self.tokens, TokenType::BitwiseAnd) {
            let right = self.parse_math_expression();
            return Expression::Math(TokenType::BitwiseAnd, Box::new(left), Box::new(right));
        }
        if match_token!(self.tokens, TokenType::Minus) {
            let right = self.parse_math_expression();
            return Expression::Math(TokenType::Minus, Box::new(left), Box::new(right));
        }
        if match_token!(self.tokens, TokenType::Plus) {
            let right = self.parse_math_expression();
            return Expression::Math(TokenType::Plus, Box::new(left), Box::new(right));
        }
        if match_token!(self.tokens, TokenType::BitwiseOr) {
            let right = self.parse_math_expression();
            return Expression::Math(TokenType::BitwiseOr, Box::new(left), Box::new(right));
        }

        left
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
        if match_token!(self.tokens, TokenType::Not) {
            return Expression::Unary(TokenType::Not, Box::from(self.parse_unary()));
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
