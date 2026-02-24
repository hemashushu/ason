// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

//! Function `reader.next()` returns `Option<Result<Token, AsonError>>`.
//!
//! Returning `Option::None` indicates the end of the stream,
//! while `Some(Err)` indicates a lexing error, and `Some(Ok)` contains the next token.
//!
//! This reader does not perform any parsing or validation of the token stream,
//! it simply reads and returns tokens as they are lexed and normalized.
//!
//! Note that the `Token::_Plus` and `Token::_Minus` tokens wouldn't be returned because
//! they are normalized into signed numbers by NormalizeSignedNumberIter.

use std::io::Read;

use crate::{
    char_with_position::CharsWithPositionIterator,
    error::AsonError,
    lexer::{Lexer, PEEK_BUFFER_LENGTH_LEX},
    normalizer::{NormalizeSignedNumberIter, PEEK_BUFFER_LENGTH_NORMALIZE},
    peekable_iterator::PeekableIterator,
    token::{Token, TokenWithRange},
    utf8_char_iterator::UTF8CharIterator,
};

pub struct TokenStreamReader<T>
where
    T: Iterator<Item = Result<TokenWithRange, AsonError>>,
{
    upstream: T,
}

impl<T> TokenStreamReader<T>
where
    T: Iterator<Item = Result<TokenWithRange, AsonError>>,
{
    pub fn new(upstream: T) -> Self {
        Self { upstream }
    }
}

impl<T> Iterator for TokenStreamReader<T>
where
    T: Iterator<Item = Result<TokenWithRange, AsonError>>,
{
    type Item = Result<Token, AsonError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.upstream
            .next()
            .map(|result| result.map(|token_with_range| token_with_range.token))
    }
}

pub fn stream_from_reader<R>(
    reader: R,
) -> TokenStreamReader<
    NormalizeSignedNumberIter<Lexer<CharsWithPositionIterator<UTF8CharIterator<R>>>>,
>
where
    R: Read,
{
    let char_iter = UTF8CharIterator::new(reader);
    stream_from_char_iterator(char_iter)
}

pub fn stream_from_char_iterator<I>(
    char_iterator: I,
) -> TokenStreamReader<NormalizeSignedNumberIter<Lexer<CharsWithPositionIterator<I>>>>
where
    I: Iterator<Item = char>,
{
    let char_position_iter = CharsWithPositionIterator::new(char_iterator);

    // Lex
    let peekable_char_position_iter =
        PeekableIterator::new(char_position_iter, PEEK_BUFFER_LENGTH_LEX);
    let lexer = Lexer::new(peekable_char_position_iter);

    // Normalize signed numbers
    let peekable_lexer_iter = PeekableIterator::new(lexer, PEEK_BUFFER_LENGTH_NORMALIZE);
    let normalizer_iter = NormalizeSignedNumberIter::new(peekable_lexer_iter);

    TokenStreamReader::new(normalizer_iter)
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, str::Chars};

    use crate::{
        char_with_position::CharsWithPositionIterator,
        lexer::Lexer,
        normalizer::NormalizeSignedNumberIter,
        token::{NumberToken, Token},
        token_stream_reader::{TokenStreamReader, stream_from_char_iterator, stream_from_reader},
    };

    /// Helper function to create a token stream reader from a string literal.
    fn stream_from_str(
        s: &str,
    ) -> TokenStreamReader<NormalizeSignedNumberIter<Lexer<CharsWithPositionIterator<Chars<'_>>>>>
    {
        stream_from_char_iterator(s.chars())
    }

    #[test]
    fn test_stream_from_str() {
        let str = "{id:123}";
        let mut reader = stream_from_str(str);

        let first_result = reader.next().unwrap();
        assert!(first_result.is_ok());

        let first_token = first_result.unwrap();
        assert_eq!(first_token, Token::OpeningBrace);

        assert_eq!(
            reader.next().unwrap().unwrap(),
            Token::Identifier("id".to_string())
        );
        assert_eq!(reader.next().unwrap().unwrap(), Token::Colon);
        assert_eq!(
            reader.next().unwrap().unwrap(),
            Token::Number(NumberToken::I32(123))
        );
        assert_eq!(reader.next().unwrap().unwrap(), Token::ClosingBrace);
        assert!(reader.next().is_none());
    }

    #[test]
    fn test_stream_from_reader() {
        let str = "123";
        let reader = Cursor::new(str);
        let mut token_stream_reader = stream_from_reader(reader);

        let first_result = token_stream_reader.next().unwrap();
        assert!(first_result.is_ok());

        let first_token = first_result.unwrap();
        assert_eq!(first_token, Token::Number(NumberToken::I32(123)));
    }
}
