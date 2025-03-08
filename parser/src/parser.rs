use crate::tokenizer::{Token, TokenType};

macro_rules! consume_token {
    ($stream:expr, $token:pat, $error:literal) => {
        if let Some(Token { token_type: $token, .. }) = $stream.peek() {
            $stream.advance();
        } else {
            panic!("Syntax Error: {}", $error);
        }
    };
}

macro_rules! consume_token2 {
    ($stream:expr, $token:pat => $out:ident, $error:literal) => {
        if let Some(Token { token_type: $token, .. }) = $stream.peek() {
            $stream.advance();
            $out
        } else {
            unreachable!();
        }
    };
}

macro_rules! check_token {
    ($stream:expr, $token:pat) => {
        if let Some(Token { token_type: $token, .. }) = $stream.peek() {
            $stream.advance();
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
        self.previous()
    }

    pub fn peek(&self) -> Option<Token> {
        if !self.at_end() {
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
    pub value: Token,
}

#[derive(Debug)]
pub enum Expression {
    Immediate(Box<Expression>),
    Unary(TokenType, Box<Expression>),
    Literal(String),
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
            if self.tokens.peek().is_some_and(|t| matches!(t.token_type, TokenType::EOL)) {
                self.tokens.advance();
                continue;
            }
            if let Some(operation) = self.parse_macro() {
                operations.push(operation);
            }
        }

        operations
    }

    fn parse_macro(&mut self) -> Option<Operation> {
        if let Some(ident) = check_token2!(self.tokens, TokenType::Macro(i) => i) {
            match ident.as_str() {
                ".include" => {
                    let path =
                        consume_token2!(self.tokens, TokenType::String(s) => s, "Expected String");
                    consume_token!(self.tokens, TokenType::EOL, "Expected EOL");
                    // println!("Include {path}");
                    return Some(Operation::Include(path));
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
                    let operation = Operation::ConstantAssign(ConstantAssign {
                        name: token,
                        value: self.tokens.advance().expect("Unexpected EOF"),
                    });

                    consume_token!(self.tokens, TokenType::EOL, "Missing Newline");

                    return Some(operation);
                }
                if check_token!(self.tokens, TokenType::Colon) {
                    return self.parse_label(token);
                }
            }
        }
        self.parse_instruction()
    }

    fn parse_instruction(&mut self) -> Option<Operation> {
        if let Some(mnemonic) = check_token2!(self.tokens, TokenType::Instruction(i) => i) {
            let mut parameters = vec![];
            if check_token!(self.tokens, TokenType::EOL) {
                // println!("No Parameters")
            } else {
                parameters.push(self.parse_expression());
                while !self.tokens.at_end() && !check_token!(self.tokens, TokenType::EOL) {
                    consume_token!(self.tokens, TokenType::Comma, "Expected Comma");
                    parameters.push(self.parse_expression());
                }
            }

            return Some(Operation::Instruction(Instruction {
                mnemonic,
                parameters,
            }));
        }
        panic!("Syntax Error: {:?}", self.tokens.peek());
    }

    fn parse_label(&mut self, name: Token) -> Option<Operation> {
        Some(Operation::Label(name))
    }

    fn parse_expression(&mut self) -> Expression {
        self.parse_unary()
    }

    fn parse_unary(&mut self) -> Expression {
        if check_token!(self.tokens, TokenType::Hash) {
            return Expression::Immediate(Box::from(self.parse_unary()));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Expression {
        if let Some(num) = check_token2!(self.tokens, TokenType::Number(i) => i) {
            return Expression::Literal(num);
        }
        if let Some(ident) = check_token2!(self.tokens, TokenType::Identifier(i) => i) {
            return Expression::Literal(ident);
        }
        panic!("Syntax Error: {:?}", self.tokens.peek());
    }
}
