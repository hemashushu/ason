// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

//! Lossless lexical classification with half-open UTF-8 byte ranges (`[start, end)`).

use std::ops::Range;

use crate::{
    AsonError, char_with_position::CharsWithPositionIterator, lexer::Lexer,
    peekable_iterator::PeekableIterator, token::Token,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    Whitespace,
    Comma,
    LineComment,
    BlockComment,
    Colon,
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    ParenthesisOpen,
    ParenthesisClose,
    Sign,
    Number,
    Boolean,
    Char,
    String,
    DateTime,
    Bytes,
    Identifier,
    Enumeration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxToken {
    pub kind: SyntaxKind,
    pub span: Range<usize>,
}

/// Classifies a lexically valid input without discarding any source text.
/// This does not validate document structure or value types.
pub fn tokenize(source: &str) -> Result<Vec<SyntaxToken>, AsonError> {
    let boundaries: Vec<_> = source
        .char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(source.len()))
        .collect();
    let characters = CharsWithPositionIterator::new(source.chars().map(Ok));
    let mut lexer = Lexer::new(PeekableIterator::new(characters), true);

    let mut tokens = Vec::new();
    loop {
        let next = lexer.next();
        let sytaxies = lexer.take_trivia();
        for (kind, range) in sytaxies {
            tokens.push(SyntaxToken {
                kind,
                span: boundaries[range.start.index]..boundaries[range.end_inclusive.index + 1],
            });
        }
        let Some(next) = next else { break };
        let token = next?;
        let kind = match token.token {
            Token::Colon => SyntaxKind::Colon,
            Token::BraceOpen => SyntaxKind::BraceOpen,
            Token::BraceClose => SyntaxKind::BraceClose,
            Token::BracketOpen => SyntaxKind::BracketOpen,
            Token::BracketClose => SyntaxKind::BracketClose,
            Token::ParenthesisOpen => SyntaxKind::ParenthesisOpen,
            Token::ParenthesisClose => SyntaxKind::ParenthesisClose,
            Token::_Plus | Token::_Minus => SyntaxKind::Sign,
            Token::Number(_) => SyntaxKind::Number,
            Token::Boolean(_) => SyntaxKind::Boolean,
            Token::Char(_) => SyntaxKind::Char,
            Token::String(_) => SyntaxKind::String,
            Token::DateTime(_) => SyntaxKind::DateTime,
            Token::HexadecimalByteData(_) => SyntaxKind::Bytes,
            Token::Identifier(_) => SyntaxKind::Identifier,
            Token::Enumeration(..) => SyntaxKind::Enumeration,
        };
        tokens.push(SyntaxToken {
            kind,
            span: boundaries[token.range.start.index]
                ..boundaries[token.range.end_inclusive.index + 1],
        });
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lossless_ranges() {
        for source in [
            "",
            " \t,\r\n",
            "// eof",
            "/* outer /* nested */ end */",
            "{中文: -0xff_i32, name: r##\"a\"#中\"##}\r\n// end",
            "(true '中' d\"2024-03-16\" h\"48 65\" Option::None)",
            "\"\"\"\n  hello\n\"\"\"",
            "\"a\\\n  b\"",
        ] {
            let tokens = tokenize(source).unwrap();
            let mut end = 0;
            for token in &tokens {
                assert_eq!(token.span.start, end, "{source:?}");
                assert!(source.get(token.span.clone()).is_some());
                end = token.span.end;
            }
            assert_eq!(end, source.len());
            assert_eq!(
                tokens
                    .iter()
                    .map(|token| &source[token.span.clone()])
                    .collect::<String>(),
                source
            );
        }
    }

    #[test]
    fn reports_lexical_errors() {
        for source in ["/* unfinished", "\"unfinished", "r##\"unfinished\"#", "@"] {
            assert!(tokenize(source).is_err(), "{source:?}");
        }
    }
}
