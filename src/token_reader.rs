// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

//! Calling `reader.next()` returns `Option<Result<Token, AsonError>>`.
//!
//! `None` means the token stream has ended.
//! `Some(Err(..))` reports a lexing error.
//! `Some(Ok(..))` yields the next token.
//!
//! This reader does not parse or validate token sequences.
//! It only returns tokens produced by the lexer after normalization.
//!
//! `Token::_Plus` and `Token::_Minus` are not emitted because
//! `NormalizeSignedNumberIter` folds them into signed numeric tokens.

use std::{io::Read, str::Chars};

use crate::{
    char_with_position::CharsWithPositionIterator,
    error::AsonError,
    lexer::{Lexer, PEEK_BUFFER_LENGTH_LEX},
    normalizer::{NormalizeSignedNumberIter, PEEK_BUFFER_LENGTH_NORMALIZE},
    peekable_iterator::PeekableIterator,
    token::{Token, TokenWithRange},
    utf8_char_iterator::UTF8CharIterator,
};

pub fn reader_from_str<'a>(
    s: &'a str,
) -> TokenReader<NormalizeSignedNumberIter<Lexer<CharsWithPositionIterator<Chars<'a>>>>> {
    reader_from_char_iterator(s.chars())
}

pub fn reader_from_reader<R>(
    reader: R,
) -> TokenReader<NormalizeSignedNumberIter<Lexer<CharsWithPositionIterator<UTF8CharIterator<R>>>>>
where
    R: Read,
{
    let char_iter = UTF8CharIterator::new(reader);
    reader_from_char_iterator(char_iter)
}

pub fn reader_from_char_iterator<I>(
    char_iterator: I,
) -> TokenReader<NormalizeSignedNumberIter<Lexer<CharsWithPositionIterator<I>>>>
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

    TokenReader::new(normalizer_iter)
}

pub struct TokenReader<T>
where
    T: Iterator<Item = Result<TokenWithRange, AsonError>>,
{
    upstream: T,
}

impl<T> TokenReader<T>
where
    T: Iterator<Item = Result<TokenWithRange, AsonError>>,
{
    pub fn new(upstream: T) -> Self {
        Self { upstream }
    }
}

impl<T> Iterator for TokenReader<T>
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{
        token::{NumberToken, Token},
        token_reader::{reader_from_reader, reader_from_str},
    };

    #[test]
    fn test_read_from_str() {
        let str = "{id:123}";
        let mut reader = reader_from_str(str);

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
    fn test_read_from_reader() {
        let str = "123";
        let reader = Cursor::new(str);
        let mut token_reader = reader_from_reader(reader);

        let first_result = token_reader.next().unwrap();
        assert!(first_result.is_ok());

        let first_token = first_result.unwrap();
        assert_eq!(first_token, Token::Number(NumberToken::I32(123)));
    }
}
