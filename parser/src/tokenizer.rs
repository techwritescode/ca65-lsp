use crate::instructions::Instructions;
use crate::stream::Stream;

#[derive(Debug, Clone)]
pub enum TokenType {
    Label(String),
    Instruction(String),
    Identifier(String),
    Number(String),
    Hash,
    Plus,
    Minus,
    LeftParen,
    RightParen,
    Comma,
    Dot,
    Colon,
    Equal,
    EOF,
    EOL,
    String(String),
    Macro(String),
    BitwiseOr,
    BitwiseAnd,
    Not,
    LessThan,
    GreaterThan,
    Caret,
    And,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub index: usize,
}

pub struct Tokenizer<'a> {
    input: Stream,
    start: usize,
    instructions: &'a Instructions,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: String, instructions: &'a Instructions) -> Self {
        Tokenizer {
            input: Stream::new(input),
            start: 0,
            instructions,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Token>, Box<dyn std::error::Error>> {
        let mut result = vec![];
        while !self.input.at_end() {
            self.start = self.input.pos();
            if let Some(token) = self.next_token() {
                result.push(token);
            }
        }
        Ok(result)
    }

    fn next_token(&mut self) -> Option<Token> {
        let c = self.input.advance();
        match c {
            Some(';') => {
                self.comment();
                None
            }
            Some('.') => {
                self.input.advance();
                let m = self.identifier();
                match m.as_str() {
                    ".bitor" => self.make_token(TokenType::BitwiseOr),
                    ".bitand" => self.make_token(TokenType::BitwiseAnd),
                    _ => self.make_token(TokenType::Macro(m))
                }
            }
            Some('"') => {
                let text = self.string();
                self.make_token(TokenType::String(text))
            }
            Some('(') => self.make_token(TokenType::LeftParen),
            Some(')') => self.make_token(TokenType::RightParen),
            Some('a'..='z' | 'A'..='Z') => {
                let name = self.identifier();
                if self.instructions.is_instruction(&name) {
                    self.make_token(TokenType::Instruction(name))
                } else {
                    self.make_token(TokenType::Identifier(name))
                }
            }
            Some('@') => {
                self.input.advance();
                let ident = self.identifier();
                self.make_token(TokenType::Identifier(ident))
            }
            Some(':') => self.make_token(TokenType::Colon),
            Some('0'..='9') => {
                let number = self.number();
                self.make_token(TokenType::Number(number))
            }
            Some('$') => {
                let number = self.hex_number();
                self.make_token(TokenType::Number(number))
            }
            Some('%') => {
                let number = self.bin_number();
                self.make_token(TokenType::Number(number))
            }
            Some('=') => self.make_token(TokenType::Equal),
            Some('#') => self.make_token(TokenType::Hash),
            Some(',') => self.make_token(TokenType::Comma),
            Some('|') => self.make_token(TokenType::BitwiseOr),
            Some('&') => {
                if self.input.peek() == Some('&') {
                    self.make_token(TokenType::And)
                } else {
                    self.make_token(TokenType::BitwiseAnd)
                }
            },
            Some('~') => self.make_token(TokenType::Not),
            Some('<') => self.make_token(TokenType::LessThan),
            Some('>') => self.make_token(TokenType::GreaterThan),
            Some('^') => self.make_token(TokenType::Caret),
            Some(' ' | '\t' | '\r') => None,
            Some('\n') => self.make_token(TokenType::EOL),
            None => self.make_token(TokenType::EOF),
            _ => {
                unreachable!("Unexpected character {:?}", c)
            }
        }
    }

    fn identifier(&mut self) -> String {
        while self
            .input
            .peek()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            self.input.advance();
        }

        let text = String::from_utf8(self.input[self.start..self.input.pos()].to_vec())
            .expect("Failed to read string");

        text
    }

    fn number(&mut self) -> String {
        while self.input.peek().is_some_and(|c| c.is_numeric()) {
            self.input.advance();
        }

        let text = String::from_utf8(self.input[self.start..self.input.pos()].to_vec())
            .expect("Failed to read string");

        text
    }

    fn hex_number(&mut self) -> String {
        while !self.input.at_end()
            && self.input.peek().is_some_and(|c| match c {
                '0'..='9' | 'A'..='F' => true,
                _ => false,
            })
        {
            self.input.advance();
        }

        let text = String::from_utf8(self.input[self.start..self.input.pos()].to_vec())
            .expect("Failed to read string");
        text
    }

    fn bin_number(&mut self) -> String {
        while !self.input.at_end()
            && self.input.peek().is_some_and(|c| match c {
                '0' | '1' => true,
                _ => false,
            })
        {
            self.input.advance();
        }

        let text = String::from_utf8(self.input[self.start..self.input.pos()].to_vec())
            .expect("Failed to read string");
        text
    }

    fn comment(&mut self) {
        while !self.input.at_end() && self.input.peek().unwrap() != '\n' {
            self.input.advance();
        }

        // println!(
        //     "Comment: {}",
        //     String::from_utf8(self.input[self.start..self.input.pos()].to_vec()).unwrap()
        // );
    }

    fn string(&mut self) -> String {
        self.input.advance();
        while !self.input.at_end() && self.input.advance() != Some('"') {}

        let string = String::from_utf8(self.input[self.start..self.input.pos()].to_vec()).unwrap();

        string
    }

    fn make_token(&self, token_type: TokenType) -> Option<Token> {
        Some(Token {
            token_type,
            lexeme: String::from_utf8(self.input[self.start..self.input.pos()].to_vec()).unwrap(),
            index: self.start,
        })
    }
}
