// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use chrono::{DateTime, FixedOffset};

use crate::range::Range;

/// A lexed ASON token, excluding whitespace, comments, and commas.
///
/// Numeric signs are separate tokens until the normalization stage. String,
/// character, DateTime, and byte payloads contain their decoded values.
/// Character and string literals support escapes including `\\`, `\'`, `\"`,
/// `\t`, `\r`, `\n`, `\0`, and Unicode escapes of the form `\u{...}`.
#[derive(Debug, PartialEq)]
pub enum Token {
    /// A `:` separating a key and value.
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

    /// A numeric literal with its ASON numeric type.
    Number(NumberToken),

    /// A decoded character literal.
    Char(char),
    /// A decoded string literal.
    String(String),

    /// A `true` or `false` literal.
    Boolean(bool),

    /// A decoded DateTime literal with a fixed UTC offset.
    DateTime(DateTime<FixedOffset>),

    /// An identifier, such as the key of an Object field.
    ///
    /// Identifiers start with an ASCII letter, underscore, or a Unicode scalar
    /// in `U+00A0..=U+D7FF` or `U+E000..=U+10FFFF`. Digits are also allowed
    /// after the first character.
    ///
    /// The variant shorthands `Some`, `None`, `Ok`, and `Err` are lexed as
    /// identifiers; see [`shorthand_enumeration`].
    Identifier(String),

    /// An enumeration's type and variant names, such as `Option` and `None`.
    Enumeration(String, String),

    /// Decoded hexadecimal byte data.
    HexadecimalByteData(Vec<u8>),

    /// A `+` sign, merged into the following number during normalization.
    _Plus,
    /// A `-` sign, merged into the following number during normalization.
    _Minus,
}

/// A numeric token preserving its ASON type.
///
/// Signed integers use unsigned storage so the lexer can represent their
/// magnitude before applying a separate sign. After normalization, that
/// storage contains the signed value's two's-complement bits. For example,
/// `-128_i8` becomes `I8(128)`, while `-3` becomes `I32(4294967293)`.
#[derive(Debug, PartialEq)]
pub enum NumberToken {
    /// An `i8` magnitude or normalized two's-complement bit pattern.
    I8(u8),
    /// A `u8` value.
    U8(u8),
    /// An `i16` magnitude or normalized two's-complement bit pattern.
    I16(u16),
    /// A `u16` value.
    U16(u16),
    /// An `i32` magnitude or normalized two's-complement bit pattern.
    I32(u32),
    /// A `u32` value.
    U32(u32),
    /// An `i64` magnitude or normalized two's-complement bit pattern.
    I64(u64),
    /// A `u64` value.
    U64(u64),
    /// An `f32` value.
    F32(f32),
    /// An `f64` value.
    F64(f64),
}

/// A token paired with its inclusive source-character range.
#[derive(Debug, PartialEq)]
pub struct TokenWithRange {
    /// The decoded lexical token.
    pub token: Token,
    /// The source characters occupied by the token.
    pub range: Range,
}

impl TokenWithRange {
    /// Associates `token` with an inclusive source `range`.
    pub fn new(token: Token, range: Range) -> Self {
        Self { token, range }
    }
}

/// Reports whether `name` is an identifier-shaped keyword: a Boolean literal or
/// an unsigned special floating-point literal. Keywords cannot be Object keys,
/// enumeration names, or variant names.
pub fn is_keyword(name: &str) -> bool {
    matches!(
        name,
        "true" | "false" | "NaN" | "NaN_f32" | "NaN_f64" | "Inf" | "Inf_f32" | "Inf_f64"
    )
}

/// Returns the predefined enumeration type of a variant shorthand
/// (`Some`, `None`, `Ok`, or `Err`), which only applies where a value is expected.
pub fn shorthand_enumeration(name: &str) -> Option<&'static str> {
    match name {
        "Some" | "None" => Some("Option"),
        "Ok" | "Err" => Some("Result"),
        _ => None,
    }
}
