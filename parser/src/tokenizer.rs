use crate::instructions::Instructions;
use crate::stream::Stream;
use codespan::Span;
use std::fmt::{Display, Formatter};

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
    ConstAssign,
    LeftBrace,
    RightBrace,
    Bank,
    SizeOf,
    Match,
    Def,
    UnnamedLabelReference,
    Extract,
    WordOp,
    LeftBracket,
    RightBracket,
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

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lexeme.clone())
    }
}

#[derive(Debug)]
pub enum TokenizerErrorKind {
    UnexpectedToken,
}

impl Display for TokenizerErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct TokenizerError {
    pub kind: TokenizerErrorKind,
    pub offset: usize,
}

impl Display for TokenizerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} Error at position: {}", self.kind, self.offset)
    }
}

type Result<T> = std::result::Result<T, TokenizerError>;

pub struct Tokenizer<'a> {
    input: Stream,
    start: usize,
    instructions: &'a Instructions,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str, instructions: &'a Instructions) -> Self {
        Tokenizer {
            input: Stream::new(input.to_string()),
            start: 0,
            instructions,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Token>> {
        let mut result = vec![];
        while !self.input.at_end() {
            self.start = self.input.pos();
            if let Some(token) = self.next_token()? {
                result.push(token);
            }
        }
        Ok(result)
    }

    fn next_token(&mut self) -> Result<Option<Token>> {
        let c = self.input.advance();
        let token: Result<Option<Token>> = match c {
            Some(';') => {
                self.comment();
                Ok(None)
            }
            Some('.') => {
                self.input.advance();
                self.identifier();
                Ok(Some(match self.get_lexeme().to_lowercase().as_str() {
                    ".bitor" => self.make_token(TokenType::BitwiseOr),
                    ".bitand" => self.make_token(TokenType::BitwiseAnd),
                    ".bitxor" => self.make_token(TokenType::BitwiseXor),
                    ".bitnot" => self.make_token(TokenType::BitwiseNot),
                    ".or" => self.make_token(TokenType::Or),
                    ".and" => self.make_token(TokenType::And),
                    ".mod" => self.make_token(TokenType::Mod),
                    ".shr" => self.make_token(TokenType::ShiftRight),
                    ".shl" => self.make_token(TokenType::ShiftLeft),
                    ".xor" => self.make_token(TokenType::Xor),
                    ".not" => self.make_token(TokenType::Not),
                    ".bank" => self.make_token(TokenType::Bank),
                    ".sizeof" => self.make_token(TokenType::SizeOf),
                    ".loword" | ".hiword" => self.make_token(TokenType::WordOp),
                    ".match" => self.make_token(TokenType::Match),
                    ".def" | ".defined" => self.make_token(TokenType::Def),
                    ".and" => self.make_token(TokenType::And),
                    ".not" => self.make_token(TokenType::Not),
                    ".left" | ".mid" | ".right" => self.make_token(TokenType::Extract),
                    _ => self.make_token(TokenType::Macro),
                }))
            }
            Some('"'|'\'') => {
                self.string(c.unwrap());
                Ok(Some(self.make_token(TokenType::String)))
            }
            Some('(') => Ok(Some(self.make_token(TokenType::LeftParen))),
            Some(')') => Ok(Some(self.make_token(TokenType::RightParen))),
            Some('{') => Ok(Some(self.make_token(TokenType::LeftBrace))),
            Some('}') => Ok(Some(self.make_token(TokenType::RightBrace))),
            Some('[') => Ok(Some(self.make_token(TokenType::LeftBracket))),
            Some(']') => Ok(Some(self.make_token(TokenType::RightBracket))),
            Some('a'..='z' | 'A'..='Z' | '_') => {
                self.identifier();
                Ok(Some(
                    if self.instructions.is_instruction(self.get_lexeme()) {
                        self.make_token(TokenType::Instruction)
                    } else {
                        self.make_token(TokenType::Identifier)
                    },
                ))
            }
            Some('@') => {
                self.input.advance();
                self.identifier();
                Ok(Some(self.make_token(TokenType::Identifier)))
            }
            Some(':') => {
                if let Some(char_following_colon) = self.input.peek() {
                    Ok(Some(match char_following_colon {
                        ':' => {
                            self.input.advance();
                            self.make_token(TokenType::ScopeSeparator)
                        }
                        '=' => {
                            self.input.advance();
                            self.make_token(TokenType::ConstAssign)
                        }
                        '+' | '-' | '>' | '<' => {
                            loop {
                                let c = self.input.peek();
                                if c.is_some_and(|c| c.is_whitespace()) {
                                    break;
                                } else if c.is_some_and(|c| c != char_following_colon) {
                                    return Err(TokenizerError {
                                        kind: TokenizerErrorKind::UnexpectedToken,
                                        offset: self.input.pos(),
                                    });
                                }

                                self.input.advance();
                            }
                            self.make_token(TokenType::UnnamedLabelReference)
                        }
                        _ => self.make_token(TokenType::Colon),
                    }))
                } else {
                    return Ok(Some(self.make_token(TokenType::Colon)));
                }
            }
            Some('0'..='9') => {
                self.number();
                Ok(Some(self.make_token(TokenType::Number)))
            }
            Some('$') => {
                self.hex_number();
                Ok(Some(self.make_token(TokenType::Number)))
            }
            Some('%') => {
                self.bin_number();
                Ok(Some(self.make_token(TokenType::Number)))
            }
            Some('|') => Ok(Some(if self.input.peek() == Some('|') {
                self.input.advance();
                self.make_token(TokenType::Or)
            } else {
                self.make_token(TokenType::BitwiseOr)
            })),
            Some('&') => Ok(Some(if self.input.peek() == Some('&') {
                self.input.advance();
                self.make_token(TokenType::And)
            } else {
                self.make_token(TokenType::BitwiseAnd)
            })),
            Some('<') => Ok(Some(if self.input.peek() == Some('<') {
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
            })),
            Some('>') => Ok(Some(if self.input.peek() == Some('>') {
                self.input.advance();
                self.make_token(TokenType::ShiftRight)
            } else if self.input.peek() == Some('=') {
                self.input.advance();
                self.make_token(TokenType::GreaterThanEq)
            } else {
                self.make_token(TokenType::GreaterThan)
            })),
            Some('!') => Ok(Some(self.make_token(TokenType::Not))),
            Some('=') => Ok(Some(self.make_token(TokenType::Equal))),
            Some('#') => Ok(Some(self.make_token(TokenType::Hash))),
            Some(',') => Ok(Some(self.make_token(TokenType::Comma))),
            Some('-') => Ok(Some(self.make_token(TokenType::Minus))),
            Some('+') => Ok(Some(self.make_token(TokenType::Plus))),
            Some('*') => Ok(Some(self.make_token(TokenType::Multiply))),
            Some('/') => Ok(Some(self.make_token(TokenType::Divide))),
            Some('~') => Ok(Some(self.make_token(TokenType::BitwiseNot))),
            Some('^') => Ok(Some(self.make_token(TokenType::Caret))),
            Some('\n') => Ok(Some(self.make_token(TokenType::EOL))),
            None => Ok(Some(self.make_token(TokenType::EOF))),
            Some(' ' | '\t' | '\r') => Ok(None),
            _ => Err(TokenizerError {
                kind: TokenizerErrorKind::UnexpectedToken,
                offset: self.start,
            }),
        };

        token
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
        while !self.input.at_end() && self.input.peek().is_some_and(|c| c.is_ascii_hexdigit()) {
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

    fn string(&mut self, variant: char) -> String {
        self.input.advance();
        while !self.input.at_end() && self.input.advance() != Some(variant) {}

        let string = String::from_utf8(self.input[self.start..self.input.pos()].to_vec()).unwrap();

        string
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        let lexeme = self.get_lexeme();
        let span = Span::new(self.start, self.start + lexeme.len());
        Token {
            token_type,
            lexeme,
            span,
        }
    }

    #[inline]
    fn get_lexeme(&self) -> String {
        String::from_utf8(self.input[self.start..self.input.pos()].to_vec()).unwrap()
    }
}
