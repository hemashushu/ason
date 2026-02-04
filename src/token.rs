// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::fmt::Display;

use chrono::{DateTime, FixedOffset};

use crate::{position::Position, range::Range};

#[derive(Debug, PartialEq)]
pub enum Token {
    NewLine,      // `\n` or `\r\n`
    Comma,        // `,`
    Colon,        // `:`
    LeftBrace,    // `{`
    RightBrace,   // `}`
    LeftBracket,  // `[`
    RightBracket, // `]`
    LeftParen,    // `(`
    RightParen,   // `)`
    Plus,         // `+`
    Minus,        // `-`

    Number(NumberToken),
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

    Date(DateTime<FixedOffset>),

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
    Comment(Comment),
}

#[derive(Debug, PartialEq)]
pub enum NumberToken {
    // Sign token (minus `-` and plus `+`) is not part of the `NumberToken`,
    // e.g., in `-128`, the `-` is a `Token::Minus` token,
    // and the `128` is a `NumberToken::I8(128)` token.
    // Since `128` overflows `i8`, so using `u8` to represent the value part.
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
pub enum Comment {
    // `//...`
    //
    // note that the trailing '\n' or '\r\n' does not belong to line comment token,
    // a line comment (e.g., `... // comment\n`) will be lexed into a
    // `Comment::Line` token and a `Token::NewLine` token.
    Line(String),

    // `/*...*/`
    //
    // A block comment can span multiple lines.
    Block(String),
}

#[derive(Debug, PartialEq)]
pub enum NumberType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

impl NumberType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        let t = match s {
            "i8" => NumberType::I8,
            "i16" => NumberType::I16,
            "i32" => NumberType::I32,
            "i64" => NumberType::I64,
            "u8" => NumberType::U8,
            "u16" => NumberType::U16,
            "u32" => NumberType::U32,
            "u64" => NumberType::U64,
            "f32" => NumberType::F32,
            "f64" => NumberType::F64,
            _ => {
                return Err(format!("Invalid number type \"{}\".", s));
            }
        };

        Ok(t)
    }
}

impl Display for NumberType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumberType::I8 => write!(f, "i8"),
            NumberType::I16 => write!(f, "i16"),
            NumberType::I32 => write!(f, "i32"),
            NumberType::I64 => write!(f, "i64"),
            NumberType::U8 => write!(f, "u8"),
            NumberType::U16 => write!(f, "u16"),
            NumberType::U32 => write!(f, "u32"),
            NumberType::U64 => write!(f, "u64"),
            NumberType::F32 => write!(f, "f32"),
            NumberType::F64 => write!(f, "f64"),
        }
    }
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

    // pub fn from_position_and_length(token: Token, position: &Position, length: usize) -> Self {
    //     Self {
    //         token,
    //         range: Range::from_position_and_length(position, length),
    //     }
    // }
}
