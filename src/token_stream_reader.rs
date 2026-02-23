// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::io::{Cursor, Read};

use crate::{
    char_with_position::CharsWithPositionIterator,
    error::AsonError,
    lexer::{Lexer, PEEK_BUFFER_LENGTH_LEX},
    normalizer::NormalizeSignedNumberIter,
    peekable_iterator::PeekableIterator,
    token::TokenWithRange,
    utf8_char_iterator::UTF8CharIterator,
};

pub struct TokenStreamReader {
    upstream: Box<dyn Iterator<Item = Result<TokenWithRange, AsonError>>>,
}

impl TokenStreamReader {
    pub fn new(upstream: Box<dyn Iterator<Item = Result<TokenWithRange, AsonError>>>) -> Self {
        Self { upstream }
    }
}

impl Iterator for TokenStreamReader {
    type Item = Result<TokenWithRange, AsonError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.upstream.next()
    }
}

pub fn stream_from_string(s: String) -> TokenStreamReader {
    let cursor = Cursor::new(s);
    stream_from_reader(Box::new(cursor))
}

pub fn stream_from_reader(reader: Box<dyn Read>) -> TokenStreamReader {
    let char_iter = UTF8CharIterator::new(reader);
    stream_from_char_iterator(Box::new(char_iter))
}

pub fn stream_from_char_iterator(
    char_iterator: Box<dyn Iterator<Item = char>>,
) -> TokenStreamReader {
    let char_position_iter = CharsWithPositionIterator::new(Box::new(char_iterator));
    let peekable_char_position_iter =
        PeekableIterator::new(Box::new(char_position_iter), PEEK_BUFFER_LENGTH_LEX);
    let lexer = Lexer::new(peekable_char_position_iter);

    // Normalize signed numbers
    let peekable_lexer_iter = PeekableIterator::new(Box::new(lexer), 1);
    let normalizer_iter = NormalizeSignedNumberIter::new(peekable_lexer_iter);

    TokenStreamReader::new(Box::new(normalizer_iter))
}

#[cfg(test)]
mod tests {
    use crate::{
        token::{NumberToken, Token},
        token_stream_reader::{TokenStreamReader, stream_from_char_iterator},
    };

    /// Helper function to create a token stream reader from a string literal.
    fn stream_from_str(s: &'static str) -> TokenStreamReader {
        let chars = s.chars();
        stream_from_char_iterator(Box::new(chars))
    }

    #[test]
    fn stream_from_str_works() {
        let mut reader = stream_from_str("{id:123}");

        assert_eq!(reader.next().unwrap().unwrap().token, Token::OpeningBrace);
        assert_eq!(
            reader.next().unwrap().unwrap().token,
            Token::Identifier("id".to_string())
        );
        assert_eq!(reader.next().unwrap().unwrap().token, Token::Colon);
        assert_eq!(
            reader.next().unwrap().unwrap().token,
            Token::Number(NumberToken::I32(123))
        );
        assert_eq!(reader.next().unwrap().unwrap().token, Token::ClosingBrace);
        assert!(reader.next().is_none());
    }
}
