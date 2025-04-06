use crate::instructions::Instructions;
use crate::stream::Stream;
use crate::Span;

#[derive(Debug, Clone, PartialEq)]
#[repr(u32)]
pub enum TokenType {
    Label,
    Instruction,
    Identifier,
    Number,
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
    String,
    Macro,
    BitwiseOr,
    BitwiseAnd,
    BitwiseNot,
    Not,
    LessThan,
    GreaterThan,
    Caret,
    And,
    Multiply,
    Divide,
    ScopeSeparator,
    Mod,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,
    Or,
    Xor,
    NotEqual,
    LessThanEq,
    GreaterThanEq,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub span: Span,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, index: usize) -> Token {
        let span = Span::new(index, index + lexeme.len());
        Token {
            token_type,
            lexeme,
            span,
        }
    }
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
                self.identifier();
                match self.get_lexeme().to_lowercase().as_str() {
                    ".bitor" => self.make_token(TokenType::BitwiseOr),
                    ".bitand" => self.make_token(TokenType::BitwiseAnd),
                    ".bitxor" => self.make_token(TokenType::BitwiseXor),
                    ".mod" => self.make_token(TokenType::Mod),
                    ".shr" => self.make_token(TokenType::ShiftRight),
                    ".shl" => self.make_token(TokenType::ShiftLeft),
                    ".xor" => self.make_token(TokenType::Xor),
                    _ => self.make_token(TokenType::Macro),
                }
            }
            Some('"') => {
                self.string();
                self.make_token(TokenType::String)
            }
            Some('(') => self.make_token(TokenType::LeftParen),
            Some(')') => self.make_token(TokenType::RightParen),
            Some('a'..='z' | 'A'..='Z' | '_') => {
                self.identifier();
                if self.instructions.is_instruction(self.get_lexeme()) {
                    self.make_token(TokenType::Instruction)
                } else {
                    self.make_token(TokenType::Identifier)
                }
            }
            Some('@') => {
                self.input.advance();
                self.identifier();
                self.make_token(TokenType::Identifier)
            }
            Some(':') => {
                if self.input.peek() == Some(':') {
                    self.input.advance();
                    self.make_token(TokenType::ScopeSeparator)
                } else {
                    self.make_token(TokenType::Colon)
                }
            }
            Some('0'..='9') => {
                self.number();
                self.make_token(TokenType::Number)
            }
            Some('$') => {
                self.hex_number();
                self.make_token(TokenType::Number)
            }
            Some('!') => self.make_token(TokenType::Not),
            Some('%') => {
                self.bin_number();
                self.make_token(TokenType::Number)
            }
            Some('=') => self.make_token(TokenType::Equal),
            Some('#') => self.make_token(TokenType::Hash),
            Some(',') => self.make_token(TokenType::Comma),
            Some('|') => {
                if self.input.peek() == Some('|') {
                    self.input.advance();
                    self.make_token(TokenType::Or)
                } else {
                    self.make_token(TokenType::BitwiseOr)
                }
            }
            Some('&') => {
                if self.input.peek() == Some('&') {
                    self.input.advance();
                    self.make_token(TokenType::And)
                } else {
                    self.make_token(TokenType::BitwiseAnd)
                }
            }
            Some('-') => self.make_token(TokenType::Minus),
            Some('+') => self.make_token(TokenType::Plus),
            Some('*') => self.make_token(TokenType::Multiply),
            Some('/') => self.make_token(TokenType::Divide),
            Some('~') => self.make_token(TokenType::BitwiseNot),
            Some('<') => {
                if self.input.peek() == Some('<') {
                    self.input.advance();
                    self.make_token(TokenType::ShiftLeft)
                } else if self.input.peek() == Some('>') {
                    self.input.advance();
                    self.make_token(TokenType::NotEqual)
                } else if self.input.peek() == Some('=') {
                    self.input.advance();
                    self.make_token(TokenType::LessThanEq)
                } else {
                    self.make_token(TokenType::LessThan)
                }
            }
            Some('>') => {
                if self.input.peek() == Some('>') {
                    self.input.advance();
                    self.make_token(TokenType::ShiftRight)
                } else if self.input.peek() == Some('=') {
                    self.input.advance();
                    self.make_token(TokenType::GreaterThanEq)
                } else {
                    self.make_token(TokenType::GreaterThan)
                }
            }
            Some('^') => self.make_token(TokenType::Caret),
            Some(' ' | '\t' | '\r') => None,
            Some('\n') => self.make_token(TokenType::EOL),
            None => self.make_token(TokenType::EOF),
            _ => {
                unreachable!("Unexpected character {:?} at {}", c, self.input.pos())
            }
        }
    }

    fn identifier(&mut self) {
        while self
            .input
            .peek()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            self.input.advance();
        }
    }

    fn number(&mut self) {
        while self.input.peek().is_some_and(|c| c.is_numeric()) {
            self.input.advance();
        }
    }

    fn hex_number(&mut self) {
        while !self.input.at_end()
            && self
                .input
                .peek()
                .is_some_and(|c| matches!(c, '0'..='9' | 'A'..='F'))
        {
            self.input.advance();
        }
    }

    fn bin_number(&mut self) {
        while !self.input.at_end() && self.input.peek().is_some_and(|c| matches!(c, '0' | '1')) {
            self.input.advance();
        }
    }

    fn comment(&mut self) {
        while !self.input.at_end() && self.input.peek().unwrap() != '\n' {
            self.input.advance();
        }
    }

    fn string(&mut self) -> String {
        self.input.advance();
        while !self.input.at_end() && self.input.advance() != Some('"') {}

        let string = String::from_utf8(self.input[self.start..self.input.pos()].to_vec()).unwrap();

        string
    }

    fn make_token(&self, token_type: TokenType) -> Option<Token> {
        let lexeme = self.get_lexeme();
        let span = Span::new(self.start, self.start + lexeme.len());
        Some(Token {
            token_type,
            lexeme,
            span,
        })
    }

    #[inline]
    fn get_lexeme(&self) -> String {
        String::from_utf8(self.input[self.start..self.input.pos()].to_vec()).unwrap()
    }
}
