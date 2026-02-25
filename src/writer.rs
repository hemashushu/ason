// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::io::{Result, Write};

use chrono::{DateTime, FixedOffset};

use crate::ast::{AsonNode, KeyValuePair, NamedListEntry, Number, Variant, VariantValue};

pub const DEFAULT_INDENT_CHARS: &str = "    ";
pub const DEFAULT_NEWLINE_CHARS: &str = "\n";

pub struct Writer<T>
where
    T: Write,
{
    upstream: T,
    indent_level: usize,
}

impl<T> Writer<T>
where
    T: Write,
{
    pub fn new(upstream: T) -> Self {
        Self {
            upstream,
            indent_level: 0,
        }
    }

    fn print_newline(&mut self) -> Result<()> {
        self.print_str(DEFAULT_NEWLINE_CHARS)?;
        for _ in 0..self.indent_level {
            self.print_str(DEFAULT_INDENT_CHARS)?;
        }
        Ok(())
    }

    fn print_space(&mut self) -> Result<()> {
        self.print_str(" ")?;
        Ok(())
    }

    fn print_opening_brace(&mut self) -> Result<()> {
        self.print_str("{")?;
        self.increase_indent();
        self.print_newline()?;
        Ok(())
    }

    fn print_closing_brace(&mut self) -> Result<()> {
        self.decrease_indent();
        self.print_newline()?;
        self.print_str("}")?;
        Ok(())
    }

    fn print_opening_bracket(&mut self) -> Result<()> {
        self.print_str("[")?;
        self.increase_indent();
        self.print_newline()?;
        Ok(())
    }

    fn print_closing_bracket(&mut self) -> Result<()> {
        self.decrease_indent();
        self.print_newline()?;
        self.print_str("]")?;
        Ok(())
    }

    fn print_opening_parenthesis(&mut self) -> Result<()> {
        self.print_str("(")?;
        Ok(())
    }

    fn print_closing_parenthesis(&mut self) -> Result<()> {
        self.print_str(")")?;
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

    fn print_number(&mut self, number: &Number) -> Result<()> {
        let str = match number {
            Number::I8(v) => {
                format!("{}_i8", v)
            }
            Number::U8(v) => {
                format!("{}_u8", v)
            }
            Number::I16(v) => {
                format!("{}_i16", v)
            }
            Number::U16(v) => {
                format!("{}_u16", v)
            }
            Number::I32(v) => {
                // `i32` is the default type for integer numbers,
                // so the suffix `_i32` can be omitted.
                format!("{}", v)
            }
            Number::U32(v) => {
                format!("{}_u32", v)
            }
            Number::I64(v) => {
                format!("{}_i64", v)
            }
            Number::U64(v) => {
                format!("{}_u64", v)
            }
            Number::F32(v) => {
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
            Number::F64(v) => {
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

    fn print_char(&mut self, c: &char) -> Result<()> {
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

    fn print_boolean(&mut self, v: &bool) -> Result<()> {
        self.print_str(if *v { "true" } else { "false" })?;
        Ok(())
    }

    fn print_datetime(&mut self, v: &DateTime<FixedOffset>) -> Result<()> {
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

    fn print_list(&mut self, items: &[AsonNode]) -> Result<()> {
        self.print_opening_bracket()?;

        for (index, item) in items.iter().enumerate() {
            self.print_node(item)?;
            if index < items.len() - 1 {
                self.print_newline()?;
            }
        }

        self.print_closing_bracket()?;
        Ok(())
    }

    fn print_named_list(&mut self, items: &[NamedListEntry]) -> Result<()> {
        self.print_opening_bracket()?;

        for (index, item) in items.iter().enumerate() {
            self.print_node(&item.name)?;
            self.print_str(": ")?;
            self.print_node(&item.value)?;
            if index < items.len() - 1 {
                self.print_newline()?;
            }
        }

        self.print_closing_bracket()?;
        Ok(())
    }

    fn print_tuple(&mut self, items: &[AsonNode]) -> Result<()> {
        self.print_opening_parenthesis()?;

        for (index, item) in items.iter().enumerate() {
            self.print_node(item)?;
            if index < items.len() - 1 {
                self.print_space()?;
            }
        }

        self.print_closing_parenthesis()?;
        Ok(())
    }

    fn print_object(&mut self, kvps: &[KeyValuePair]) -> Result<()> {
        self.print_opening_brace()?;

        for (index, kvp) in kvps.iter().enumerate() {
            self.print_str(&kvp.key)?;
            self.print_str(": ")?;
            self.print_node(&kvp.value)?;
            if index < kvps.len() - 1 {
                self.print_newline()?;
            }
        }

        self.print_closing_brace()?;
        Ok(())
    }

    fn print_variant(&mut self, var: &Variant) -> Result<()> {
        let (type_name, member_name, value) = (&var.type_name, &var.member_name, &var.value);

        self.print_str(&format!("{}::{}", type_name, member_name))?;

        match value {
            VariantValue::Empty => {
                // do nothing
            }
            VariantValue::Value(val) => {
                self.print_str("(")?;
                self.print_node(val)?;
                self.print_str(")")?;
            }
            VariantValue::Tuple(items) => {
                self.print_tuple(items)?;
            }
            VariantValue::Object(kvps) => {
                self.print_object(kvps)?;
            }
        }

        Ok(())
    }

    fn print_node(&mut self, node: &AsonNode) -> Result<()> {
        match node {
            AsonNode::Number(n) => self.print_number(n),
            AsonNode::Char(c) => self.print_char(c),
            AsonNode::String(s) => self.print_string(s),
            AsonNode::Boolean(v) => self.print_boolean(v),
            AsonNode::DateTime(d) => self.print_datetime(d),
            AsonNode::HexadecimalByteData(data) => self.print_hexadecimal_byte_data(data),
            AsonNode::List(items) => self.print_list(items),
            AsonNode::NamedList(items) => self.print_named_list(items),
            AsonNode::Tuple(items) => self.print_tuple(items),
            AsonNode::Object(kvps) => self.print_object(kvps),
            AsonNode::Variant(var) => self.print_variant(var),
        }
    }
}

pub fn write_to_string(node: &AsonNode) -> String {
    let mut buf: Vec<u8> = vec![];
    let mut writer = Writer::new(&mut buf);
    writer.print_node(node).unwrap();
    String::from_utf8(buf).unwrap()
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use pretty_assertions::assert_eq;

    use crate::{
        ast::{AsonNode, KeyValuePair, NamedListEntry, Number, Variant},
        writer::write_to_string,
    };

    impl AsonNode {
        fn new_string(s: &str) -> AsonNode {
            AsonNode::String(s.to_owned())
        }

        fn new_number(n: i32) -> AsonNode {
            AsonNode::Number(Number::I32(n))
        }
    }

    #[test]
    fn test_print_number() {
        // test print `i32` positive number
        assert_eq!(
            write_to_string(&AsonNode::Number(Number::I32(123))),
            "123".to_owned()
        );

        // test print `i32` negative number
        assert_eq!(
            write_to_string(&AsonNode::Number(Number::I32(-123))),
            "-123".to_owned()
        );

        assert_eq!(
            write_to_string(&AsonNode::Number(Number::I8(123))),
            "123_i8".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::Number(Number::I8(-123))),
            "-123_i8".to_owned()
        );

        assert_eq!(
            write_to_string(&AsonNode::Number(Number::U8(123))),
            "123_u8".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::Number(Number::I16(123))),
            "123_i16".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::Number(Number::I16(-123))),
            "-123_i16".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::Number(Number::U16(123))),
            "123_u16".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::Number(Number::U32(123))),
            "123_u32".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::Number(Number::I64(123))),
            "123_i64".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::Number(Number::I64(-123))),
            "-123_i64".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::Number(Number::U64(123))),
            "123_u64".to_owned()
        );

        assert_eq!(
            write_to_string(&AsonNode::Number(Number::F32(3.5))),
            "3.5_f32".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::Number(Number::F32(f32::NAN))),
            "NaN_f32".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::Number(Number::F32(f32::INFINITY))),
            "Inf_f32".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::Number(Number::F32(f32::NEG_INFINITY))),
            "-Inf_f32".to_owned()
        );

        assert_eq!(
            write_to_string(&AsonNode::Number(Number::F64(3.5))),
            "3.5".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::Number(Number::F64(3.0))),
            "3.0".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::Number(Number::F64(f64::NAN))),
            "NaN".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::Number(Number::F64(f64::INFINITY))),
            "Inf".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::Number(Number::F64(f64::NEG_INFINITY))),
            "-Inf".to_owned()
        );
    }

    #[test]
    fn test_print_char() {
        // test print general char
        assert_eq!(write_to_string(&AsonNode::Char('A')), "'A'".to_owned());

        // test print special chars that need to be escaped
        assert_eq!(write_to_string(&AsonNode::Char('\\')), "'\\\\'".to_owned());
        assert_eq!(write_to_string(&AsonNode::Char('\'')), "'\\\''".to_owned());
        assert_eq!(write_to_string(&AsonNode::Char('\t')), "'\\t'".to_owned());
        assert_eq!(write_to_string(&AsonNode::Char('\r')), "'\\r'".to_owned());
        assert_eq!(write_to_string(&AsonNode::Char('\n')), "'\\n'".to_owned());
        assert_eq!(write_to_string(&AsonNode::Char('\0')), "'\\0'".to_owned());
    }

    #[test]
    fn test_print_string() {
        // test print general string
        assert_eq!(
            write_to_string(&AsonNode::String("hello".to_owned())),
            "\"hello\"".to_owned()
        );

        // test print string with special characters that need to be escaped
        assert_eq!(
            write_to_string(&AsonNode::String("\\\"\t\0".to_owned())),
            "\"\\\\\\\"\\t\\0\"".to_owned()
        );

        // test print string with unicode characters
        assert_eq!(
            write_to_string(&AsonNode::String("Hello, 世界! ❤️".to_owned())),
            "\"Hello, 世界! ❤️\"".to_owned()
        );

        // test print empty string
        assert_eq!(
            write_to_string(&AsonNode::String("".to_owned())),
            "\"\"".to_owned()
        );

        // test print multiline string
        assert_eq!(
            write_to_string(&AsonNode::String("line1\nline2".to_owned())),
            "\"line1\nline2\"".to_owned()
        );
    }

    #[test]
    fn test_print_boolean() {
        assert_eq!(write_to_string(&AsonNode::Boolean(true)), "true".to_owned());
        assert_eq!(
            write_to_string(&AsonNode::Boolean(false)),
            "false".to_owned()
        );
    }

    #[test]
    fn test_print_datetime() {
        let d1 = DateTime::parse_from_rfc3339("2024-03-16T16:30:50+08:00").unwrap();
        let d2 = DateTime::parse_from_rfc3339("2024-03-16T08:30:50Z").unwrap();

        assert_eq!(
            write_to_string(&AsonNode::DateTime(d1)),
            "d\"2024-03-16T16:30:50+08:00\"".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::DateTime(d2)),
            "d\"2024-03-16T08:30:50+00:00\"".to_owned()
        );
    }

    #[test]
    fn test_print_hexadecimal_byte_data() {
        assert_eq!(
            write_to_string(&AsonNode::HexadecimalByteData(vec![0x00, 0x11, 0x22])),
            "h\"00 11 22\"".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::HexadecimalByteData(vec![
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
            ])),
            "h\"00 11 22 33  44 55 66 77\"".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::HexadecimalByteData(vec![
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
            ])),
            "h\"00 11 22 33  44 55 66 77\n88\"".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::HexadecimalByteData(vec![
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
                0xee, 0xff, 0x10,
            ])),
            "h\"00 11 22 33  44 55 66 77\n88 99 aa bb  cc dd ee ff\n10\"".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::HexadecimalByteData(vec![])),
            "h\"\"".to_owned()
        );
    }

    #[test]
    fn test_print_list() {
        // test print list with one element, e.g., [1]
        assert_eq!(
            write_to_string(&AsonNode::List(vec![AsonNode::new_number(1)])),
            "[\n    1\n]".to_owned()
        );

        // test print list with multiple elements, e.g., [1, 2, 3]
        assert_eq!(
            write_to_string(&AsonNode::List(vec![
                AsonNode::new_number(1),
                AsonNode::new_number(2),
                AsonNode::new_number(3),
            ])),
            "[\n    1\n    2\n    3\n]".to_owned()
        );

        // test print empty list, e.g., []
        assert_eq!(
            write_to_string(&AsonNode::List(vec![])),
            "[\n    \n]".to_owned()
        );
    }

    #[test]
    fn test_print_named_list() {
        // test print named list with one item, e.g., `["foo": 123]`
        assert_eq!(
            write_to_string(&AsonNode::NamedList(vec![NamedListEntry {
                name: Box::new(AsonNode::new_string("foo")),
                value: Box::new(AsonNode::new_number(123)),
            }])),
            "[\n    \"foo\": 123\n]".to_owned()
        );

        // test print named list with multiple items, e.g., ["foo": 123, "bar": "Alice"]
        assert_eq!(
            write_to_string(&AsonNode::NamedList(vec![
                NamedListEntry {
                    name: Box::new(AsonNode::new_string("foo")),
                    value: Box::new(AsonNode::new_number(123)),
                },
                NamedListEntry {
                    name: Box::new(AsonNode::new_string("bar")),
                    value: Box::new(AsonNode::new_string("Alice")),
                },
            ])),
            "[\n    \"foo\": 123\n    \"bar\": \"Alice\"\n]".to_owned()
        );
    }

    #[test]
    fn test_print_tuple() {
        // test print tuple with multiple elements, e.g., (1, "Alice", true)
        assert_eq!(
            write_to_string(&AsonNode::Tuple(vec![
                AsonNode::new_number(1),
                AsonNode::new_string("Alice"),
                AsonNode::Boolean(true),
            ])),
            "(1 \"Alice\" true)".to_owned()
        );
    }

    #[test]
    fn test_print_object() {
        // test print object with single field, e.g., {id: 123}
        assert_eq!(
            write_to_string(&AsonNode::Object(vec![KeyValuePair::new(
                "id",
                AsonNode::new_number(123),
            )])),
            "{\n    id: 123\n}".to_owned()
        );

        // test print object with multiple fields, e.g., {id: 123 name: "Alice"}
        assert_eq!(
            write_to_string(&AsonNode::Object(vec![
                KeyValuePair::new("id", AsonNode::new_number(123)),
                KeyValuePair::new("name", AsonNode::new_string("Alice")),
            ])),
            "{\n    id: 123\n    name: \"Alice\"\n}".to_owned()
        );

        // test print nested object
        assert_eq!(
            write_to_string(&AsonNode::Object(vec![
                KeyValuePair::new("id", AsonNode::new_number(123)),
                KeyValuePair::new(
                    "address",
                    AsonNode::Object(vec![
                        KeyValuePair::new("city", AsonNode::new_string("New York")),
                        KeyValuePair::new("zip", AsonNode::new_string("10001")),
                    ]),
                ),
            ])),
            "{\n    id: 123\n    address: {\n        city: \"New York\"\n        zip: \"10001\"\n    }\n}".to_owned()
        );

        // test print object with list value, e.g., `{ id: 123 tags: ["tag1", "tag2", "tag3"] }`
        assert_eq!(
            write_to_string(&AsonNode::Object(vec![
                KeyValuePair::new("id", AsonNode::new_number(123)),
                KeyValuePair::new(
                    "tags",
                    AsonNode::List(vec![
                        AsonNode::new_string("tag1"),
                        AsonNode::new_string("tag2"),
                        AsonNode::new_string("tag3"),
                    ]),
                ),
            ])),
            "{\n    id: 123\n    tags: [\n        \"tag1\"\n        \"tag2\"\n        \"tag3\"\n    ]\n}".to_owned()
        );
    }

    #[test]
    fn test_print_variant_without_value() {
        assert_eq!(
            write_to_string(&AsonNode::Variant(Variant::new("Option", "None"))),
            "Option::None".to_owned()
        );
        assert_eq!(
            write_to_string(&AsonNode::Variant(Variant::new("Result", "Ok"))),
            "Result::Ok".to_owned()
        );
    }

    #[test]
    fn test_print_variant_with_single_value() {
        // test print variant with single integer value, e.g., `Option::Some(123)`
        assert_eq!(
            write_to_string(&AsonNode::Variant(Variant::with_value(
                "Option",
                "Some",
                AsonNode::new_number(123),
            ))),
            "Option::Some(123)".to_owned()
        );

        // test print variant with a string value, e.g., `Result::Err("error message")`
        assert_eq!(
            write_to_string(&AsonNode::Variant(Variant::with_value(
                "Result",
                "Err",
                AsonNode::new_string("Error message"),
            ))),
            "Result::Err(\"Error message\")".to_owned()
        );

        // test print variant with list value, e.g., `Variant::List([1, 2, 3])`
        assert_eq!(
            write_to_string(&AsonNode::Variant(Variant::with_value(
                "Variant",
                "List",
                AsonNode::List(vec![
                    AsonNode::new_number(1),
                    AsonNode::new_number(2),
                    AsonNode::new_number(3),
                ]),
            ))),
            "Variant::List([\n    1\n    2\n    3\n])".to_owned()
        );

        // test print variant with object value, e.g., `Variant::Object{id: 123 name: "Alice"}`
        assert_eq!(
            write_to_string(&AsonNode::Variant(Variant::with_value(
                "Variant",
                "Object",
                AsonNode::Object(vec![
                    KeyValuePair::new("id", AsonNode::new_number(123)),
                    KeyValuePair::new("name", AsonNode::new_string("Alice")),
                ]),
            ))),
            "Variant::Object({\n    id: 123\n    name: \"Alice\"\n})".to_owned()
        );
    }

    #[test]
    fn test_print_tuple_like_variant() {
        // test print tuple-like variant, e.g., `Variant::Tuple(1, "Alice", true)`
        assert_eq!(
            write_to_string(&AsonNode::Variant(Variant::with_tuple_like(
                "Variant",
                "Tuple",
                vec![
                    AsonNode::new_number(1),
                    AsonNode::new_string("Alice"),
                    AsonNode::Boolean(true),
                ],
            ))),
            "Variant::Tuple(1 \"Alice\" true)".to_owned()
        );
    }

    #[test]
    fn test_print_object_like_variant() {
        // test print object-like variant, e.g., `Variant::Object{id: 123, name: "Alice"}`
        assert_eq!(
            write_to_string(&AsonNode::Variant(Variant::with_object_like(
                "Variant",
                "Object",
                vec![
                    KeyValuePair::new("id", AsonNode::new_number(123)),
                    KeyValuePair::new("name", AsonNode::new_string("Alice")),
                ],
            ))),
            "Variant::Object{\n    id: 123\n    name: \"Alice\"\n}".to_owned()
        );
    }
}
