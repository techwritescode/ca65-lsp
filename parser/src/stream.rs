use std::ops::{Index, Range};

pub struct Stream {
    input: String,
    position: usize,
}

impl Stream {
    pub fn new(input: String) -> Self {
        Stream { input, position: 0 }
    }

    pub fn pos(&self) -> usize {
        self.position
    }


    pub fn peek(&self) -> Option<char> {
        if self.position >= self.input.len() {
            None
        } else {
            Some(self.input.as_bytes()[self.position] as char)
        }
    }

    pub fn peek_next(&self) -> Option<char> {
        if self.position + 1 >= self.input.len() {
            None
        } else {
            Some(self.input.as_bytes()[self.position + 1] as char)
        }
    }

    pub fn at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    pub fn advance(&mut self) -> Option<char> {
        let c = self.input.as_bytes()[self.position] as char;
        self.position += 1;
        Some(c)
    }

    pub fn match_char(&mut self, expected: char) -> bool {
        if self.at_end() || self.peek() != Some(expected) {
            false
        } else {
            self.advance();
            true
        }
    }
}

impl Index<Range<usize>> for Stream {
    type Output = [u8];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.input.as_bytes()[index]
    }
}