// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

//! Function `print_token()` in this module is used to print a token to the upstream writer,
//! which can be a file, a string buffer, or any other type that implements the `Write` trait.
//!
//! This writer does not perform any validation of the token stream, it simply formats and
//! writes tokens as they are provided.
//!
//! Note that do not print `Token::_Plus` and `Token::_Minus` tokens directly because they are
//! not part of the normalized token stream, use the `Token::Number(NumberToken::I32(-123_i32 as u32))` instead
//! if you want to print a negative number.

use std::io::{Result, Write};

use chrono::{DateTime, FixedOffset};

use crate::token::{NumberToken, Token};

pub const DEFAULT_INDENT_CHARS: &str = "    ";
pub const DEFAULT_NEWLINE_CHARS: &str = "\n";

pub struct TokenStreamWriter<T>
where
    T: Write,
{
    upstream: T,
    indent_level: usize,
}

impl<T> TokenStreamWriter<T>
where
    T: Write,
{
    pub fn new(upstream: T) -> Self {
        Self {
            upstream,
            indent_level: 0,
        }
    }

    pub fn print_token(&mut self, token: &Token) -> Result<()> {
        match token {
            Token::OpeningBrace => {
                self.print_str("{")?;
                self.increase_indent();
                self.print_newline()?;
            }
            Token::ClosingBrace => {
                self.decrease_indent();
                self.print_newline()?;
                self.print_str("}")?;
            }
            Token::OpeningBracket => {
                self.print_str("[")?;
                self.increase_indent();
                self.print_newline()?;
            }
            Token::ClosingBracket => {
                self.decrease_indent();
                self.print_newline()?;
                self.print_str("]")?;
            }
            Token::OpeningParenthesis => {
                self.print_str("(")?;
            }
            Token::ClosingParenthesis => {
                self.print_str(")")?;
            }
            Token::Number(number_token) => {
                self.print_number(number_token)?;
            }
            Token::Char(c) => {
                self.print_char(*c)?;
            }
            Token::String(s) => {
                self.print_string(s)?;
            }
            Token::Boolean(b) => {
                self.print_str(if *b { "true" } else { "false" })?;
            }
            Token::DateTime(date) => {
                self.print_date(date)?;
            }
            Token::Identifier(ident) => {
                self.print_str(ident)?;
            }
            Token::Colon => {
                self.print_str(":")?;
            }
            Token::Enumeration(type_name, member_name) => {
                self.print_str(type_name)?;
                self.print_str("::")?;
                self.print_str(member_name)?;
            }
            Token::HexadecimalByteData(bytes) => {
                self.print_hexadecimal_byte_data(bytes)?;
            }
            _ => {
                unreachable!()
            }
        }
        Ok(())
    }

    pub fn print_newline(&mut self) -> Result<()> {
        self.print_str(DEFAULT_NEWLINE_CHARS)?;
        for _ in 0..self.indent_level {
            self.print_str(DEFAULT_INDENT_CHARS)?;
        }
        Ok(())
    }

    pub fn print_space(&mut self) -> Result<()> {
        self.print_str(" ")?;
        Ok(())
    }

    fn print_str(&mut self, s: &str) -> Result<()> {
        self.upstream.write_all(s.as_bytes())?;
        Ok(())
    }

    fn increase_indent(&mut self) {
        self.indent_level += 1;
    }

    fn decrease_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    fn print_number(&mut self, number_token: &NumberToken) -> Result<()> {
        let str = match number_token {
            NumberToken::I8(v) => {
                format!("{}_i8", *v as i8)
            }
            NumberToken::U8(v) => {
                format!("{}_u8", v)
            }
            NumberToken::I16(v) => {
                format!("{}_i16", *v as i16)
            }
            NumberToken::U16(v) => {
                format!("{}_u16", v)
            }
            NumberToken::I32(v) => {
                // `i32` is the default type for integer numbers,
                // so the suffix `_i32` can be omitted.
                format!("{}", *v as i32)
            }
            NumberToken::U32(v) => {
                format!("{}_u32", v)
            }
            NumberToken::I64(v) => {
                format!("{}_i64", *v as i64)
            }
            NumberToken::U64(v) => {
                format!("{}_u64", v)
            }
            NumberToken::F32(v) => {
                if v.is_nan() {
                    "NaN_f32".to_owned()
                } else if v == &f32::INFINITY {
                    "Inf_f32".to_owned()
                } else if v == &f32::NEG_INFINITY {
                    "-Inf_f32".to_owned()
                } else {
                    format!("{}_f32", v)
                }
            }
            NumberToken::F64(v) => {
                // `f64` is the default type for floating-point numbers,
                // so the suffix `_f64` can be omitted.
                if v.is_nan() {
                    "NaN".to_owned()
                } else if v == &f64::INFINITY {
                    "Inf".to_owned()
                } else if v == &f64::NEG_INFINITY {
                    "-Inf".to_owned()
                } else {
                    // a decimal point needs to be appended to indicate that it is a floating-point literal
                    // if there is no decimal point.
                    // For example, `3` is an integer literal, while `3.0` is a floating-point literal.
                    let mut s = v.to_string();
                    if !s.contains('.') {
                        s.push_str(".0");
                    }
                    s
                }
            }
        };

        self.print_str(&str)?;
        Ok(())
    }

    fn print_char(&mut self, c: char) -> Result<()> {
        // escape single char
        let str = match c {
            '\\' => "\\\\".to_owned(),
            '\'' => "\\'".to_owned(),
            '\t' => {
                // horizontal tabulation
                "\\t".to_owned()
            }
            '\r' => {
                // carriage return, jump to the beginning of the line (CR)
                "\\r".to_owned()
            }
            '\n' => {
                // new line/line feed (LF)
                "\\n".to_owned()
            }
            '\0' => {
                // null char
                "\\0".to_owned()
            }
            _ => c.to_string(),
        };

        self.print_str(&format!("'{}'", str))?;
        Ok(())
    }

    fn print_string(&mut self, s: &str) -> Result<()> {
        let str = s
            .chars()
            .map(|c| match c {
                '\\' => "\\\\".to_owned(),
                '"' => "\\\"".to_owned(),
                '\t' => "\\t".to_owned(),

                // null char is allowed in the source code,
                // it is used to represent the null-terminated string.
                '\0' => "\\0".to_owned(),

                _ => c.to_string(),
            })
            .collect::<Vec<String>>()
            .join("");

        self.print_str(&format!("\"{}\"", str))?;
        Ok(())
    }

    fn print_date(&mut self, v: &DateTime<FixedOffset>) -> Result<()> {
        self.print_str(&format!("d\"{}\"", v.to_rfc3339()))?;
        Ok(())
    }

    /// Format the byte array as a hexadecimal string with the following format:
    ///
    /// ```text
    /// - Each byte is represented as a two-digit hexadecimal number (00 to ff).
    /// - Bytes are separated by a space.
    /// - Every 4 bytes, an additional space is added for readability.
    /// - Every 8 bytes, a newline is added, and the subsequent lines are indented.
    /// - The entire byte array is enclosed in `h"..."`.
    ///
    ///   h"00 11 22 33  44 55 66 77
    ///     88 99 aa bb  cc dd ee ff"
    /// ^^^^__ indent
    /// ```
    ///
    fn print_hexadecimal_byte_data(&mut self, data: &[u8]) -> Result<()> {
        let leading_space_chars = DEFAULT_INDENT_CHARS.repeat(self.indent_level);
        let line_separator = format!("\n{}", leading_space_chars);
        let str = data
            .chunks(8)
            .map(|chunk| {
                // each line
                chunk
                    .iter()
                    .enumerate()
                    .map(|(idx, byte)| {
                        // Rust std format!()
                        // https://doc.rust-lang.org/std/fmt/
                        if idx == 4 {
                            format!("  {:02x}", byte)
                        } else if idx == 0 {
                            format!("{:02x}", byte)
                        } else {
                            format!(" {:02x}", byte)
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("")
            })
            .collect::<Vec<String>>()
            .join(&line_separator);

        self.print_str(&format!("h\"{}\"", str))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Result, Write};

    use chrono::DateTime;
    use pretty_assertions::assert_eq;

    use crate::{
        token::{NumberToken, Token},
        token_stream_writer::TokenStreamWriter,
    };

    /// Helper function to create a token stream writer that writes to a string.
    fn print_token_to_string(token: &Token) -> String {
        let mut output = Vec::new();
        let mut writer = TokenStreamWriter::new(&mut output);
        writer.print_token(token).unwrap();
        String::from_utf8(output).unwrap()
    }

    fn print_tokens_with_space_separated_to_string<W: Write>(
        writer: &mut TokenStreamWriter<W>,
        tokens: &[Token],
    ) {
        for (index, token) in tokens.iter().enumerate() {
            writer.print_token(token).unwrap();
            if index < tokens.len() - 1 {
                writer.print_space().unwrap();
            }
        }
    }

    #[test]
    fn test_print_number() {
        // test print `i32` positive number
        assert_eq!(
            print_token_to_string(&Token::Number(NumberToken::I32(123))),
            "123"
        );

        // test print `i32` negative number
        assert_eq!(
            print_token_to_string(&Token::Number(NumberToken::I32(-123_i32 as u32))),
            "-123"
        );

        assert_eq!(
            print_token_to_string(&Token::Number(NumberToken::U32(123))),
            "123_u32"
        );

        assert_eq!(
            print_token_to_string(&Token::Number(NumberToken::I64(123))),
            "123_i64"
        );

        assert_eq!(
            print_token_to_string(&Token::Number(NumberToken::I64(-123_i64 as u64))),
            "-123_i64"
        );

        assert_eq!(
            print_token_to_string(&Token::Number(NumberToken::U64(123))),
            "123_u64"
        );

        assert_eq!(
            print_token_to_string(&Token::Number(NumberToken::F32(3.5))),
            "3.5_f32"
        );
        assert_eq!(
            print_token_to_string(&Token::Number(NumberToken::F32(f32::NAN))),
            "NaN_f32"
        );
        assert_eq!(
            print_token_to_string(&Token::Number(NumberToken::F32(f32::INFINITY))),
            "Inf_f32"
        );
        assert_eq!(
            print_token_to_string(&Token::Number(NumberToken::F32(f32::NEG_INFINITY))),
            "-Inf_f32"
        );

        assert_eq!(
            print_token_to_string(&Token::Number(NumberToken::F64(3.5))),
            "3.5"
        );
        assert_eq!(
            print_token_to_string(&Token::Number(NumberToken::F64(3.0))),
            "3.0"
        );
        assert_eq!(
            print_token_to_string(&Token::Number(NumberToken::F64(f64::NAN))),
            "NaN"
        );
        assert_eq!(
            print_token_to_string(&Token::Number(NumberToken::F64(f64::INFINITY))),
            "Inf"
        );
        assert_eq!(
            print_token_to_string(&Token::Number(NumberToken::F64(f64::NEG_INFINITY))),
            "-Inf"
        );
    }

    #[test]
    fn test_print_char() {
        // test print general char
        assert_eq!(print_token_to_string(&Token::Char('A')), "'A'");

        // test print special chars that need to be escaped
        assert_eq!(print_token_to_string(&Token::Char('\\')), "'\\\\'");
        assert_eq!(print_token_to_string(&Token::Char('\'')), "'\\\''");
        assert_eq!(print_token_to_string(&Token::Char('\t')), "'\\t'");
        assert_eq!(print_token_to_string(&Token::Char('\r')), "'\\r'");
        assert_eq!(print_token_to_string(&Token::Char('\n')), "'\\n'");
        assert_eq!(print_token_to_string(&Token::Char('\0')), "'\\0'");
    }

    #[test]
    fn test_print_string() {
        // test print general string
        assert_eq!(
            print_token_to_string(&Token::String("hello".to_owned())),
            "\"hello\""
        );

        // test print string with special characters that need to be escaped
        assert_eq!(
            print_token_to_string(&Token::String("\\\"\t\0".to_owned())),
            "\"\\\\\\\"\\t\\0\""
        );

        // test print string with unicode characters
        assert_eq!(
            print_token_to_string(&Token::String("Hello, 世界! ❤️".to_owned())),
            "\"Hello, 世界! ❤️\""
        );

        // test print empty string
        assert_eq!(print_token_to_string(&Token::String("".to_owned())), "\"\"");

        // test print multiline string
        assert_eq!(
            print_token_to_string(&Token::String("line1\nline2".to_owned())),
            "\"line1\nline2\""
        );
    }

    #[test]
    fn test_print_boolean() {
        assert_eq!(print_token_to_string(&Token::Boolean(true)), "true");
        assert_eq!(print_token_to_string(&Token::Boolean(false)), "false");
    }

    #[test]
    fn test_print_date() {
        let d1 = DateTime::parse_from_rfc3339("2024-03-16T16:30:50+08:00").unwrap();
        let d2 = DateTime::parse_from_rfc3339("2024-03-16T08:30:50Z").unwrap();

        assert_eq!(
            print_token_to_string(&Token::DateTime(d1)),
            "d\"2024-03-16T16:30:50+08:00\""
        );
        assert_eq!(
            print_token_to_string(&Token::DateTime(d2)),
            "d\"2024-03-16T08:30:50+00:00\""
        );
    }

    #[test]
    fn test_print_hexadecimal_byte_data() {
        assert_eq!(
            print_token_to_string(&Token::HexadecimalByteData(vec![0x00, 0x11, 0x22])),
            "h\"00 11 22\""
        );
        assert_eq!(
            print_token_to_string(&Token::HexadecimalByteData(vec![
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
            ])),
            "h\"00 11 22 33  44 55 66 77\""
        );
        assert_eq!(
            print_token_to_string(&Token::HexadecimalByteData(vec![
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
            ])),
            "h\"00 11 22 33  44 55 66 77\n88\""
        );
        assert_eq!(
            print_token_to_string(&Token::HexadecimalByteData(vec![
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
                0xee, 0xff, 0x10,
            ])),
            "h\"00 11 22 33  44 55 66 77\n88 99 aa bb  cc dd ee ff\n10\""
        );
        assert_eq!(
            print_token_to_string(&Token::HexadecimalByteData(vec![])),
            "h\"\""
        );
    }

    #[test]
    fn test_print_list() {
        // test print list with one element, e.g., [1]
        let t = || -> Result<String> {
            let mut output = Vec::new();
            let mut writer = TokenStreamWriter::new(&mut output);
            writer.print_token(&Token::OpeningBracket)?;
            print_tokens_with_space_separated_to_string(
                &mut writer,
                &[Token::Number(NumberToken::I32(1))],
            );
            writer.print_token(&Token::ClosingBracket)?;

            Ok(String::from_utf8(output).unwrap())
        };

        assert_eq!(
            t().unwrap(),
            "\
[
    1
]"
        );

        // test print list with multiple elements, e.g., [1, 2, 3]
        let t = || -> Result<String> {
            let mut output = Vec::new();
            let mut writer = TokenStreamWriter::new(&mut output);
            writer.print_token(&Token::OpeningBracket)?;
            print_tokens_with_space_separated_to_string(
                &mut writer,
                &[
                    Token::Number(NumberToken::I32(1)),
                    Token::Number(NumberToken::I32(2)),
                    Token::Number(NumberToken::I32(3)),
                ],
            );
            writer.print_token(&Token::ClosingBracket)?;

            Ok(String::from_utf8(output).unwrap())
        };

        assert_eq!(
            t().unwrap(),
            "\
[
    1 2 3
]"
        );

        // test print empty list, e.g., []
        let t = || -> Result<String> {
            let mut output = Vec::new();
            let mut writer = TokenStreamWriter::new(&mut output);
            writer.print_token(&Token::OpeningBracket)?;
            writer.print_token(&Token::ClosingBracket)?;

            Ok(String::from_utf8(output).unwrap())
        };

        assert_eq!(t().unwrap(), "[\n    \n]");
    }

    #[test]
    fn test_print_named_list() {
        // test print named list with one item, e.g., `["foo": 123]`
        let t = || -> Result<String> {
            let mut output = Vec::new();
            let mut writer = TokenStreamWriter::new(&mut output);
            writer.print_token(&Token::OpeningBracket)?;
            writer.print_token(&Token::String("foo".to_owned()))?;
            writer.print_token(&Token::Colon)?;
            writer.print_space()?;
            writer.print_token(&Token::Number(NumberToken::I32(123)))?;
            writer.print_token(&Token::ClosingBracket)?;

            Ok(String::from_utf8(output).unwrap())
        };

        assert_eq!(
            t().unwrap(),
            "\
[
    \"foo\": 123
]"
        );

        // test print named list with multiple items, e.g., ["foo": 123, "bar": "Alice"]
        let t = || -> Result<String> {
            let mut output = Vec::new();
            let mut writer = TokenStreamWriter::new(&mut output);
            writer.print_token(&Token::OpeningBracket)?;
            writer.print_token(&Token::String("foo".to_owned()))?;
            writer.print_token(&Token::Colon)?;
            writer.print_space()?;
            writer.print_token(&Token::Number(NumberToken::I32(123)))?;
            writer.print_space()?;
            writer.print_token(&Token::String("bar".to_owned()))?;
            writer.print_token(&Token::Colon)?;
            writer.print_space()?;
            writer.print_token(&Token::String("Alice".to_owned()))?;
            writer.print_token(&Token::ClosingBracket)?;

            Ok(String::from_utf8(output).unwrap())
        };

        assert_eq!(
            t().unwrap(),
            "\
[
    \"foo\": 123 \"bar\": \"Alice\"
]"
        );
    }

    #[test]
    fn test_print_tuple() {
        // test print tuple with multiple elements, e.g., (1, "Alice", true)
        let t = || -> Result<String> {
            let mut output = Vec::new();
            let mut writer = TokenStreamWriter::new(&mut output);
            writer.print_token(&Token::OpeningParenthesis)?;
            print_tokens_with_space_separated_to_string(
                &mut writer,
                &[
                    Token::Number(NumberToken::I32(1)),
                    Token::String("Alice".to_owned()),
                    Token::Boolean(true),
                ],
            );
            writer.print_token(&Token::ClosingParenthesis)?;

            Ok(String::from_utf8(output).unwrap())
        };

        assert_eq!(t().unwrap(), "(1 \"Alice\" true)");
    }

    #[test]
    fn test_print_object() {
        // test print object with single field, e.g., {id: 123}
        let t = || -> Result<String> {
            let mut output = Vec::new();
            let mut writer = TokenStreamWriter::new(&mut output);
            writer.print_token(&Token::OpeningBrace)?;
            writer.print_token(&Token::Identifier("id".to_owned()))?;
            writer.print_token(&Token::Colon)?;
            writer.print_space()?;
            writer.print_token(&Token::Number(NumberToken::I32(123)))?;
            writer.print_token(&Token::ClosingBrace)?;
            Ok(String::from_utf8(output).unwrap())
        };

        assert_eq!(
            t().unwrap(),
            "\
{
    id: 123
}"
        );

        // test print object with multiple fields, e.g., {id: 123 name: "Alice"}
        let t = || -> Result<String> {
            let mut output = Vec::new();
            let mut writer = TokenStreamWriter::new(&mut output);
            writer.print_token(&Token::OpeningBrace)?;
            writer.print_token(&Token::Identifier("id".to_owned()))?;
            writer.print_token(&Token::Colon)?;
            writer.print_space()?;
            writer.print_token(&Token::Number(NumberToken::I32(123)))?;
            writer.print_space()?;
            writer.print_token(&Token::Identifier("name".to_owned()))?;
            writer.print_token(&Token::Colon)?;
            writer.print_space()?;
            writer.print_token(&Token::String("Alice".to_owned()))?;
            writer.print_token(&Token::ClosingBrace)?;

            Ok(String::from_utf8(output).unwrap())
        };

        assert_eq!(
            t().unwrap(),
            "\
{
    id: 123 name: \"Alice\"
}"
        );

        // test print nested object
        let t = || -> Result<String> {
            let mut output = Vec::new();
            let mut writer = TokenStreamWriter::new(&mut output);
            writer.print_token(&Token::OpeningBrace)?;
            writer.print_token(&Token::Identifier("id".to_owned()))?;
            writer.print_token(&Token::Colon)?;
            writer.print_space()?;
            writer.print_token(&Token::Number(NumberToken::I32(123)))?;
            writer.print_space()?;
            writer.print_token(&Token::Identifier("address".to_owned()))?;
            writer.print_token(&Token::Colon)?;
            writer.print_space()?;
            writer.print_token(&Token::OpeningBrace)?;
            writer.print_token(&Token::Identifier("city".to_owned()))?;
            writer.print_token(&Token::Colon)?;
            writer.print_space()?;
            writer.print_token(&Token::String("New York".to_owned()))?;
            writer.print_space()?;
            writer.print_token(&Token::Identifier("zip".to_owned()))?;
            writer.print_token(&Token::Colon)?;
            writer.print_space()?;
            writer.print_token(&Token::String("10001".to_owned()))?;
            writer.print_token(&Token::ClosingBrace)?;
            writer.print_token(&Token::ClosingBrace)?;
            Ok(String::from_utf8(output).unwrap())
        };

        assert_eq!(
            t().unwrap(),
            "\
{
    id: 123 address: {
        city: \"New York\" zip: \"10001\"
    }
}"
        );

        // test print object with list value, e.g., `{ id: 123 tags: ["tag1", "tag2", "tag3"] }`
        let t = || -> Result<String> {
            let mut output = Vec::new();
            let mut writer = TokenStreamWriter::new(&mut output);
            writer.print_token(&Token::OpeningBrace)?;
            writer.print_token(&Token::Identifier("id".to_owned()))?;
            writer.print_token(&Token::Colon)?;
            writer.print_space()?;
            writer.print_token(&Token::Number(NumberToken::I32(123)))?;
            writer.print_space()?;
            writer.print_token(&Token::Identifier("tags".to_owned()))?;
            writer.print_token(&Token::Colon)?;
            writer.print_space()?;
            writer.print_token(&Token::OpeningBracket)?;
            print_tokens_with_space_separated_to_string(
                &mut writer,
                &[
                    Token::String("tag1".to_owned()),
                    Token::String("tag2".to_owned()),
                    Token::String("tag3".to_owned()),
                ],
            );
            writer.print_token(&Token::ClosingBracket)?;
            writer.print_token(&Token::ClosingBrace)?;
            Ok(String::from_utf8(output).unwrap())
        };
        assert_eq!(
            t().unwrap(),
            "\
{
    id: 123 tags: [
        \"tag1\" \"tag2\" \"tag3\"
    ]
}"
        );
    }

    #[test]
    fn test_print_variant_without_value() {
        assert_eq!(
            print_token_to_string(&Token::Enumeration("Option".to_owned(), "None".to_owned())),
            "Option::None"
        );
        assert_eq!(
            print_token_to_string(&Token::Enumeration("Result".to_owned(), "Ok".to_owned())),
            "Result::Ok"
        );
    }

    #[test]
    fn test_print_variant_with_single_value() {
        // test print variant with single integer value, e.g., `Option::Some(123)`
        let t = || -> Result<String> {
            let mut output = Vec::new();
            let mut writer = TokenStreamWriter::new(&mut output);
            writer.print_token(&Token::Enumeration("Option".to_owned(), "Some".to_owned()))?;
            writer.print_token(&Token::OpeningParenthesis)?;
            writer.print_token(&Token::Number(NumberToken::I32(123)))?;
            writer.print_token(&Token::ClosingParenthesis)?;
            Ok(String::from_utf8(output).unwrap())
        };
        assert_eq!(t().unwrap(), "Option::Some(123)");

        // test print variant with a string value, e.g., `Result::Err("error message")`
        let t = || -> Result<String> {
            let mut output = Vec::new();
            let mut writer = TokenStreamWriter::new(&mut output);
            writer.print_token(&Token::Enumeration("Result".to_owned(), "Err".to_owned()))?;
            writer.print_token(&Token::OpeningParenthesis)?;
            writer.print_token(&Token::String("Error message".to_owned()))?;
            writer.print_token(&Token::ClosingParenthesis)?;
            Ok(String::from_utf8(output).unwrap())
        };
        assert_eq!(t().unwrap(), "Result::Err(\"Error message\")");

        // test print variant with list value, e.g., `Enumeration::List([1, 2, 3])`
        let t = || -> Result<String> {
            let mut output = Vec::new();
            let mut writer = TokenStreamWriter::new(&mut output);
            writer.print_token(&Token::Enumeration("Enumeration".to_owned(), "List".to_owned()))?;
            writer.print_token(&Token::OpeningParenthesis)?;
            writer.print_token(&Token::OpeningBracket)?;
            print_tokens_with_space_separated_to_string(
                &mut writer,
                &[
                    Token::Number(NumberToken::I32(1)),
                    Token::Number(NumberToken::I32(2)),
                    Token::Number(NumberToken::I32(3)),
                ],
            );
            writer.print_token(&Token::ClosingBracket)?;
            writer.print_token(&Token::ClosingParenthesis)?;
            Ok(String::from_utf8(output).unwrap())
        };
        assert_eq!(t().unwrap(), "Enumeration::List([\n    1 2 3\n])");

        // test print variant with object value, e.g., `Enumeration::Object{id: 123 name: "Alice"}`
        let t = || -> Result<String> {
            let mut output = Vec::new();
            let mut writer = TokenStreamWriter::new(&mut output);
            writer.print_token(&Token::Enumeration("Enumeration".to_owned(), "Object".to_owned()))?;
            writer.print_token(&Token::OpeningParenthesis)?;
            writer.print_token(&Token::OpeningBrace)?;
            writer.print_token(&Token::Identifier("id".to_owned()))?;
            writer.print_token(&Token::Colon)?;
            writer.print_space()?;
            writer.print_token(&Token::Number(NumberToken::I32(123)))?;
            writer.print_space()?;
            writer.print_token(&Token::Identifier("name".to_owned()))?;
            writer.print_token(&Token::Colon)?;
            writer.print_space()?;
            writer.print_token(&Token::String("Alice".to_owned()))?;
            writer.print_token(&Token::ClosingBrace)?;
            writer.print_token(&Token::ClosingParenthesis)?;
            Ok(String::from_utf8(output).unwrap())
        };
        assert_eq!(
            t().unwrap(),
            "Enumeration::Object({\n    id: 123 name: \"Alice\"\n})"
        );
    }

    #[test]
    fn test_print_tuple_like_variant() {
        // test print tuple-like variant, e.g., `Enumeration::Tuple(1, "Alice", true)`
        let t = || -> Result<String> {
            let mut output = Vec::new();
            let mut writer = TokenStreamWriter::new(&mut output);
            writer.print_token(&Token::Enumeration("Enumeration".to_owned(), "Tuple".to_owned()))?;
            writer.print_token(&Token::OpeningParenthesis)?;
            print_tokens_with_space_separated_to_string(
                &mut writer,
                &[
                    Token::Number(NumberToken::I32(1)),
                    Token::String("Alice".to_owned()),
                    Token::Boolean(true),
                ],
            );
            writer.print_token(&Token::ClosingParenthesis)?;

            Ok(String::from_utf8(output).unwrap())
        };

        assert_eq!(t().unwrap(), "Enumeration::Tuple(1 \"Alice\" true)");
    }

    #[test]
    fn test_print_object_like_variant() {
        // test print object-like variant, e.g., `Enumeration::Object{id: 123, name: "Alice"}`
        let t = || -> Result<String> {
            let mut output = Vec::new();
            let mut writer = TokenStreamWriter::new(&mut output);
            writer.print_token(&Token::Enumeration("Enumeration".to_owned(), "Object".to_owned()))?;
            writer.print_token(&Token::OpeningBrace)?;
            writer.print_token(&Token::Identifier("id".to_owned()))?;
            writer.print_token(&Token::Colon)?;
            writer.print_space()?;
            writer.print_token(&Token::Number(NumberToken::I32(123)))?;
            writer.print_space()?;
            writer.print_token(&Token::Identifier("name".to_owned()))?;
            writer.print_token(&Token::Colon)?;
            writer.print_space()?;
            writer.print_token(&Token::String("Alice".to_owned()))?;
            writer.print_token(&Token::ClosingBrace)?;

            Ok(String::from_utf8(output).unwrap())
        };

        assert_eq!(
            t().unwrap(),
            "Enumeration::Object{\n    id: 123 name: \"Alice\"\n}"
        );
    }
}
