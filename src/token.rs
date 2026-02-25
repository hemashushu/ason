// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use chrono::{DateTime, FixedOffset};

use crate::range::Range;

#[derive(Debug, PartialEq)]
pub enum Token {
    Colon,              // `:`
    OpeningBrace,       // `{`
    ClosingBrace,       // `}`
    OpeningBracket,     // `[`
    ClosingBracket,     // `]`
    OpeningParenthesis, // `(`
    ClosingParenthesis, // `)`

    Number(NumberToken),

    // ASON only supports certain escaped characters in character and string literals,
    // which is similar to Rust's character and string literals. The supported escape sequences are:
    //
    // - `\\` (backslash)
    // - `\'` (single quote)
    // - `\"` (double quote)
    // - `\t` (tab)
    // - `\r` (carriage return)
    // - `\n` (newline)
    // - `\0` (null character)
    // - `\u{...}` (Unicode code point, where `...` is a hexadecimal number)
    Char(char),
    String(String),

    // ASON has a few keywords:
    // - `true`
    // - `false`
    // - `Inf (Inf_f32, Inf_f64)`
    // - `NaN (NaN_f32, NaN_f64)`
    //
    // `true` and `false` are interpreted to `Token::Boolean`,
    // `NaN` and `Inf` are interpreted to `Token::NumberToken`.
    Boolean(bool),

    DateTime(DateTime<FixedOffset>),

    // An identifier is used for object (or struct) field name,
    // It is a sequence of letters, digits, underscores:
    // - `[a-zA-Z0-9_]`
    // - '\u{a0}' - '\u{d7ff}'
    // - '\u{e000}' - '\u{10ffff}'
    Identifier(String),

    // A variant is consisted of a variant type name and a member name, e.g.,
    // `Option::None`, the "Option" is type name, and "None" is member name.
    Variant(String, String),

    HexadecimalByteData(Vec<u8>),

    // Sign tokens, used for number literals.
    // Note that they are removed after normalization.
    _Plus,  // `+`
    _Minus, // `-`
}

// Tokens for number literals.
//
// Note that the sign token (minus `-` and plus `+`) is not part of the `NumberToken` in
// the first stage of lexing. For example, `-128` is tokenized into two tokens:
//
// - `Token::Minus`
// - `Token::Number(NumberToken::I8(128))`
//
// Since `128` overflows `i8`, so using `u8` to represent the value part.
// After normalization, the sign token is merged into the `NumberToken`,
// so `-128` will be normalized to `Token::Number(NumberToken::I8(128))`.
// Where `128` is Two's Complement Representation of `-128` in `i8`.
//
// Another example, `-3` is tokenized into two tokens:
// - `Token::Minus`
// - `Token::Number(NumberToken::I32(3))`
//
// After normalization, it will be normalized to `Token::Number(NumberToken::I32(253))`.
// Where `253` is Two's Complement Representation of `-3` in `i32`.
//
// The reason for this design is that it simplifies the lexing process, as we can first tokenize
// the number literal without worrying about the sign and the valid range.
#[derive(Debug, PartialEq)]
pub enum NumberToken {
    I8(u8),
    U8(u8),
    I16(u16),
    U16(u16),
    I32(u32),
    U32(u32),
    I64(u64),
    U64(u64),
    F32(f32),
    F64(f64),
}

#[derive(Debug, PartialEq)]
pub struct TokenWithRange {
    pub token: Token,
    pub range: Range,
}

impl TokenWithRange {
    pub fn new(token: Token, range: Range) -> Self {
        Self { token, range }
    }
}
