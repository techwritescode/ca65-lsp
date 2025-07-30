use crate::TokenType;
use codespan::Span;
use std::fmt::{Display, Formatter};

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
