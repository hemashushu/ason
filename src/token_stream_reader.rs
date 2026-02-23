// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::io::Read;

use crate::{
    char_with_position::CharsWithPositionIter,
    error::AsonError,
    lexer::{Lexer, PEEK_BUFFER_LENGTH_LEX},
    normalizer::NormalizeSignedNumberIter,
    peekable_iter::PeekableIter,
    token::TokenWithRange,
    utf8_char_iterator::UTF8CharIterator,
};

pub struct TokenStreamReader {
    // todo
}

impl TokenStreamReader {
    pub fn new() -> Self {
        todo!()
    }
}

impl Iterator for TokenStreamReader {
    type Item = Result<TokenWithRange, AsonError>;

    fn next(&mut self) -> Option<Self::Item> {
        // self.upstream.next()
        todo!()
    }
}

pub fn stream_from_str(s: &str) -> TokenStreamReader {
    let mut chars = s.chars();
    stream_from_char_iterator(&mut chars)
}

pub fn stream_from_reader<R: Read>(mut r: R) -> TokenStreamReader {
    let mut char_stream = UTF8CharIterator::new(&mut r);
    stream_from_char_iterator(&mut char_stream)
}

pub fn stream_from_char_iterator(char_iterator: impl Iterator<Item = char>) -> TokenStreamReader {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::token::Token;

    use super::stream_from_str;

    #[test]
    fn stream_from_str_works() {
        let mut reader = stream_from_str("{id:123}");

        let first = reader.next().unwrap().unwrap();
        assert_eq!(first.token, Token::OpeningBrace);
    }
}
