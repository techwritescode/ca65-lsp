use crate::tokenizer::Token;

// macro_rules! match_token {
//     ($stream:expr, $token:pat) => {{
//         match $stream.peek() {
//             Some($token) => {
//                 $stream.advance();
//                 true
//             }
//             _ => false,
//         }
//     }};
// }

macro_rules! consume_token {
    ($stream:expr, $token:pat, $error:literal) => {
        match $stream.peek() {
            Some($token) => $stream.advance(),
            _ => {
                panic!("Syntax Error: {}", $error);
            }
        }
    };
}

macro_rules! consume_token2 {
    ($stream:expr, $token:pat => $out:ident, $error:literal) => {
        match $stream.peek() {
            Some($token) => {
                $stream.advance();
                $out
            }
            _ => {
                panic!("Syntax Error: {}", $error);
            }
        }
    };
}

macro_rules! check_token {
    ($stream:expr, $token:pat) => {
        match $stream.peek() {
            Some($token) => {
                $stream.advance();
                true
            }
            _ => false,
        }
    };
}

macro_rules! check_token2 {
    ($stream:expr, $token:pat => $out:ident) => {
        match $stream.peek() {
            Some($token) => {
                $stream.advance();
                Some($out)
            }
            _ => None,
        }
    };
}

pub struct TokenStream {
    tokens: Vec<Token>,
    position: usize,
}

impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> Self {
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
    name: String,
    value: Token,
}

#[derive(Debug)]
pub enum Expression {
    Immediate(Box<Expression>),
    Unary(Token, Box<Expression>),
    Literal(String),
}

#[derive(Debug)]
pub struct Instruction {
    mnemonic: String,
    parameters: Vec<Expression>,
}

#[derive(Debug)]
pub enum Operation {
    ConstantAssign(ConstantAssign),
    Include(String),
    Label(String),
    Instruction(Instruction),
}

pub struct Parser {
    tokens: TokenStream,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: TokenStream::new(tokens),
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Vec<Operation> {
        let mut operations = vec![];

        while !self.tokens.at_end() {
            if self.tokens.peek().is_some_and(|t| match t {
                Token::EOL => true,
                _ => false,
            }) {
                self.tokens.advance();
                continue;
            }
            if let Some(operation) = self.parse_macro() {
                println!("Received Token {:#?}", operation);
                operations.push(operation);
            }
        }

        operations
    }

    fn parse_macro(&mut self) -> Option<Operation> {
        if let Some(ident) = check_token2!(self.tokens, Token::Macro(i) => i) {
            match ident.as_str() {
                ".include" => {
                    let path =
                        consume_token2!(self.tokens, Token::String(s) => s, "Expected String");
                    consume_token!(self.tokens, Token::EOL, "Expected EOL");
                    println!("Include {path}");
                    return Some(Operation::Include(path));
                }
                _ => panic!("Unexpected Macro: {}", ident),
            }
        }

        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Option<Operation> {
        match self.tokens.peek() {
            Some(Token::Identifier(ident)) => {
                self.tokens.advance();
                if check_token!(self.tokens, Token::Equal) {
                    let operation = Operation::ConstantAssign(ConstantAssign {
                        name: ident.to_string(),
                        value: self.tokens.advance().expect("Unexpected EOF"),
                    });

                    consume_token!(self.tokens, Token::EOL, "Missing Newline");

                    return Some(operation);
                }
                if check_token!(self.tokens, Token::Colon) {
                    return self.parse_label(ident);
                }
                // match self.tokens.peek() {
                //     Some(Token::Identifier(ident)) => {
                //         self.tokens.advance();
                //         println!("Assignment to ident: {}", ident);
                //     },
                //     Some(Token::Number(num)) => {
                //         self.tokens.advance();
                //     }
                //     _ => {
                //         panic!("Unexpected Token: {}", self.tokens.peek().unwrap());
                //     }
                // }
            }
            _ => {}
        }
        self.parse_instruction()
    }

    fn parse_instruction(&mut self) -> Option<Operation> {
        if let Some(mnemonic) = check_token2!(self.tokens, Token::Instruction(i) => i) {
            let mut parameters = vec![];
            if check_token!(self.tokens, Token::EOL) {
                println!("No Parameters")
            } else {
                parameters.push(self.parse_expression());
                while !self.tokens.at_end() && !check_token!(self.tokens, Token::EOL) {
                    consume_token!(self.tokens, Token::Comma, "Expected Comma");
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

    fn parse_label(&mut self, name: String) -> Option<Operation> {
        Some(Operation::Label(name))
    }

    fn parse_expression(&mut self) -> Expression {
        self.parse_unary()
    }

    fn parse_unary(&mut self) -> Expression {
        if (check_token!(self.tokens, Token::Hash)) {
            return Expression::Immediate(Box::from(self.parse_unary()));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Expression {
        if let Some(num) = check_token2!(self.tokens, Token::Number(i) => i) {
            return Expression::Literal(num);
        }
        if let Some(ident) = check_token2!(self.tokens, Token::Identifier(i) => i) {
            return Expression::Literal(ident);
        }
        panic!("Syntax Error: {:?}", self.tokens.peek());
    }
}
