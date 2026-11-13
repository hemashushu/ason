// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

//! Lossless lexical classification with half-open UTF-8 byte ranges (`[start, end)`).

use std::ops::Range;

use crate::{
    AsonError, char_with_position::CharsWithPositionIterator, lexer::Lexer,
    peekable_iterator::PeekableIterator, token::{Token, shorthand_enumeration},
};

/// The lexical category of a span in an ASON source string.
///
/// Trivia is included, so whitespace, commas, and comments can be preserved
/// alongside value and punctuation tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    /// A space, tab, carriage return, or line feed.
    Whitespace,
    /// A comma separator.
    Comma,
    /// A `//` line comment.
    LineComment,
    /// A `/* ... */` block comment, which may contain nested block comments.
    BlockComment,
    /// A `:` separating a key from a value.
    Colon,
    /// An opening `{`.
    BraceOpen,
    /// A closing `}`.
    BraceClose,
    /// An opening `[`.
    BracketOpen,
    /// A closing `]`.
    BracketClose,
    /// An opening `(`.
    ParenthesisOpen,
    /// A closing `)`.
    ParenthesisClose,
    /// A `+` or `-` sign token, including a leading numeric sign.
    Sign,
    /// A numeric literal, excluding a separate leading sign token.
    Number,
    /// A `true` or `false` literal.
    Boolean,
    /// A character literal.
    Char,
    /// A string literal, including raw or multiline forms.
    String,
    /// A DateTime literal beginning with `d`.
    DateTime,
    /// Hexadecimal byte data beginning with `h`.
    Bytes,
    /// An identifier, such as an Object field name.
    Identifier,
    /// An enumeration name and variant, or a supported shorthand.
    Enumeration,
}

/// A lexical category and its location in the original UTF-8 source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxToken {
    /// The kind of source text covered by this token.
    pub kind: SyntaxKind,
    /// A half-open UTF-8 byte range suitable for slicing the input string.
    pub span: Range<usize>,
}

/// Classifies an ASON source string without discarding any of its text.
///
/// Returned spans are ordered, contiguous, and cover the entire input,
/// including whitespace, commas, and comments. Each span uses half-open UTF-8
/// byte offsets and can be used to slice `source`. An empty input produces an
/// empty vector.
///
/// This performs lexical analysis only; it does not validate the document's
/// structure, target types, or signed-number normalization.
///
/// # Errors
///
/// Returns an error for an invalid or incomplete lexical token.
///
/// # Examples
///
/// ```
/// use ason::syntax::{SyntaxKind, tokenize};
///
/// let source = r#"{name: "Ada"}"#;
/// let tokens = tokenize(source)?;
/// assert_eq!(tokens[0].kind, SyntaxKind::BraceOpen);
/// let reconstructed: String = tokens.iter()
///     .map(|token| &source[token.span.clone()])
///     .collect();
/// assert_eq!(reconstructed, source);
/// # Ok::<(), ason::AsonError>(())
/// ```
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
    classify_shorthands(source, &mut tokens);
    Ok(tokens)
}

/// Reclassifies the variant shorthands (`Some`, `None`, `Ok`, and `Err`) as
/// enumerations, except where they are Object keys (an identifier directly
/// inside `{ ... }` and followed by `:`).
fn classify_shorthands(source: &str, tokens: &mut [SyntaxToken]) {
    let is_trivia = |kind: SyntaxKind| {
        matches!(
            kind,
            SyntaxKind::Whitespace
                | SyntaxKind::Comma
                | SyntaxKind::LineComment
                | SyntaxKind::BlockComment
        )
    };
    let mut open_delimiters = Vec::new();
    for index in 0..tokens.len() {
        match tokens[index].kind {
            kind @ (SyntaxKind::BraceOpen
            | SyntaxKind::BracketOpen
            | SyntaxKind::ParenthesisOpen) => open_delimiters.push(kind),
            SyntaxKind::BraceClose | SyntaxKind::BracketClose | SyntaxKind::ParenthesisClose => {
                open_delimiters.pop();
            }
            SyntaxKind::Identifier
                if shorthand_enumeration(&source[tokens[index].span.clone()]).is_some() =>
            {
                let is_object_key = open_delimiters.last() == Some(&SyntaxKind::BraceOpen)
                    && tokens[index + 1..]
                        .iter()
                        .find(|token| !is_trivia(token.kind))
                        .is_some_and(|token| token.kind == SyntaxKind::Colon);
                if !is_object_key {
                    tokens[index].kind = SyntaxKind::Enumeration;
                }
            }
            _ => {}
        }
    }
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

    #[test]
    fn classifies_shorthands_by_position() {
        let source = "{Some: Some(1), None /* c */ : [None: Ok(Err(2))]}";
        let kinds: Vec<_> = tokenize(source)
            .unwrap()
            .into_iter()
            .filter(|token| {
                matches!(token.kind, SyntaxKind::Identifier | SyntaxKind::Enumeration)
            })
            .map(|token| (&source[token.span], token.kind))
            .collect();
        assert_eq!(
            kinds,
            [
                ("Some", SyntaxKind::Identifier),
                ("Some", SyntaxKind::Enumeration),
                ("None", SyntaxKind::Identifier),
                ("None", SyntaxKind::Enumeration),
                ("Ok", SyntaxKind::Enumeration),
                ("Err", SyntaxKind::Enumeration),
            ]
        );
    }
}
