// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use chrono::{DateTime, FixedOffset};

use crate::range::Range;

#[derive(Debug, PartialEq)]
pub enum Token {
    Colon,            // `:`
    BraceOpen,        // `{`
    BraceClose,       // `}`
    BracketOpen,      // `[`
    BracketClose,     // `]`
    ParenthesisOpen,  // `(`
    ParenthesisClose, // `)`

    Number(NumberToken),

    // ASON supports only a few escape sequences in char and string literals,
    // similar to Rust. The supported escape sequences are:
    //
    // - `\\` (backslash)
    // - `\'` (single quote)
    // - `\"` (double quote)
    // - `\t` (horizontal tab)
    // - `\r` (carriage return)
    // - `\n` (line feed)
    // - `\0` (null character)
    // - `\u{...}` (Unicode code point, where `...` is 1 to 6 hexadecimal digits)
    Char(char),
    String(String),

    // ASON has a few keywords:
    //
    // - `true` and `false`, which are lexed into `Token::Boolean`.
    // - `NaN` (`NaN_f32`, `NaN_f64`) and `Inf` (`Inf_f32`, `Inf_f64`),
    //   which are lexed into `Token::Number`.
    Boolean(bool),

    DateTime(DateTime<FixedOffset>),

    // An identifier is used as the key of an Object field.
    // It starts with a letter, an underscore, or a Unicode character in the range
    // '\u{a0}' to '\u{d7ff}' or '\u{e000}' to '\u{10ffff}', followed by any number of
    // these characters and digits.
    Identifier(String),

    // An enumeration consists of a type name and a variant name, for example,
    // `Option::None`, where "Option" is the type name and "None" is the variant name.
    Enumeration(String, String),

    HexadecimalByteData(Vec<u8>),

    // Sign tokens for numbers.
    // They are merged into the following number tokens and removed by the normalizer.
    _Plus,  // `+`
    _Minus, // `-`
}

// Tokens for number literals.
//
// In the lexing stage, the sign (`+` or `-`) is a separate token rather than part of
// `NumberToken`, so numbers can be lexed without worrying about the sign and the valid range.
// For example, `-128_i8` is lexed into two tokens:
//
// - `Token::_Minus`
// - `Token::Number(NumberToken::I8(128))`
//
// Since `128` overflows `i8`, signed integers are stored in their unsigned counterparts.
// After normalization, the sign is merged into the number token, and `-128_i8` becomes
// `Token::Number(NumberToken::I8(128))`, where `128` (0x80) is the two's complement
// representation of `-128` in `i8`.
//
// Another example, `-3` is lexed into two tokens:
//
// - `Token::_Minus`
// - `Token::Number(NumberToken::I32(3))`
//
// After normalization, it becomes `Token::Number(NumberToken::I32(4294967293))`,
// where `4294967293` (0xFFFF_FFFD) is the two's complement representation of `-3` in `i32`.
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
