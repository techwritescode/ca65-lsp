use std::fmt;
use crate::instructions::Instructions;
use crate::stream::Stream;

#[derive(Debug, Clone)]
pub enum Token {
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
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Tokenizer {
    input: Stream,
    start: usize,
    instructions: Instructions,
}

impl Tokenizer {
    pub fn new(input: String, instructions: Instructions) -> Self {
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
                Some(Token::Macro(m))
            }
            Some('"') => {
                let text = self.string();
                Some(Token::String(text))
            }
            Some('(') => Some(Token::LeftParen),
            Some(')') => Some(Token::RightParen),
            Some('a'..='z' | 'A'..='Z') => {
                let name = self.identifier();
                if self.instructions.is_instruction(&name) {
                    Some(Token::Instruction(name))
                } else {
                    Some(Token::Identifier(name))
                }
            }
            Some('@') => {
                self.input.advance();
                let ident = self.identifier();
                Some(Token::Identifier(ident))
            }
            Some(':') => Some(Token::Colon),
            Some('0'..='9') => {
                let number = self.number();
                Some(Token::Number(number))
            }
            Some('$') => {
                let number = self.hex_number();
                Some(Token::Number(number))
            }
            Some('=') => Some(Token::Equal),
            Some('#') => Some(Token::Hash),
            Some(',') => Some(Token::Comma),
            Some(' ' | '\t' | '\r') => None,
            Some('\n') => Some(Token::EOL),
            None => Some(Token::EOF),
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
        while !self.input.at_end() && self.input.peek().is_some_and(|c| match c {
            '0'..='9' | 'A'..='F' => true,
            _ => false,
        }) {
            self.input.advance();
        }

        let text = String::from_utf8(self.input[self.start..self.input.pos()].to_vec()).expect("Failed to read string");
        text
    }

    fn comment(&mut self) {
        while !self.input.at_end() && self.input.peek() != Some('\n') {
            self.input.advance();
        }

        println!(
            "Comment: {}",
            String::from_utf8(self.input[self.start..self.input.pos()].to_vec()).unwrap()
        );
    }

    fn string(&mut self) -> String {
        self.input.advance();
        while !self.input.at_end() && self.input.advance() != Some('"') {}

        let string = String::from_utf8(self.input[self.start..self.input.pos()].to_vec()).unwrap();
        println!("String: {string}");

        string
    }
}
