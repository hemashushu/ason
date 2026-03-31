// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::{io::Write, marker::PhantomData};

use serde::{Serialize, ser};

use crate::error::AsonError;

pub const DEFAULT_INDENT_CHARS: &str = "    ";
pub const DEFAULT_NEWLINE_CHARS: &str = "\n";

pub fn ser_to_string<T>(value: &T) -> Result<String, AsonError>
where
    T: Serialize,
{
    let mut buf: Vec<u8> = vec![];
    ser_to_writer(value, &mut buf)?;
    let s = String::from_utf8(buf).unwrap();
    Ok(s)
}

pub fn ser_to_writer<T, W: Write>(value: &T, writer: &mut W) -> Result<(), AsonError>
where
    T: Serialize,
{
    let mut serializer = Serializer::new(writer);
    value.serialize(&mut serializer)
}

pub struct Serializer<'a, W>
where
    W: Write,
{
    upstream: &'a mut W,
    indent_level: usize,
    is_first_element: bool,
}

impl<'a, W> Serializer<'a, W>
where
    W: Write,
{
    pub fn new(upstream: &'a mut W) -> Self {
        Self {
            upstream,
            indent_level: 0,
            is_first_element: false,
        }
    }

    fn print_newline(&mut self) -> Result<(), AsonError> {
        self.print_str(DEFAULT_NEWLINE_CHARS)?;
        for _ in 0..self.indent_level {
            self.print_str(DEFAULT_INDENT_CHARS)?;
        }
        Ok(())
    }

    fn print_space(&mut self) -> Result<(), AsonError> {
        self.print_str(" ")?;
        Ok(())
    }

    fn print_opening_brace(&mut self) -> Result<(), AsonError> {
        self.print_str("{")?;
        self.increase_indent();
        self.print_newline()?;
        Ok(())
    }

    fn print_closing_brace(&mut self) -> Result<(), AsonError> {
        self.decrease_indent();
        self.print_newline()?;
        self.print_str("}")?;
        Ok(())
    }

    fn print_opening_bracket(&mut self) -> Result<(), AsonError> {
        self.print_str("[")?;
        self.increase_indent();
        self.print_newline()?;
        Ok(())
    }

    fn print_closing_bracket(&mut self) -> Result<(), AsonError> {
        self.decrease_indent();
        self.print_newline()?;
        self.print_str("]")?;
        Ok(())
    }

    fn print_opening_parenthesis(&mut self) -> Result<(), AsonError> {
        self.print_str("(")?;
        Ok(())
    }

    fn print_closing_parenthesis(&mut self) -> Result<(), AsonError> {
        self.print_str(")")?;
        Ok(())
    }

    fn print_str(&mut self, s: &str) -> Result<(), AsonError> {
        self.upstream
            .write_all(s.as_bytes())
            .map_err(|e| AsonError::Message(e.to_string()))
    }

    fn increase_indent(&mut self) {
        self.indent_level += 1;
    }

    fn decrease_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }
}

pub fn list_to_writer<T, W>(writer: &mut W) -> ListSerializer<'_, W, T>
where
    W: Write,
{
    let serializer = Serializer::new(writer);
    ListSerializer::new(serializer)
}

pub struct ListSerializer<'a, W, T>
where
    W: Write,
{
    serializer: Serializer<'a, W>,
    _marker: PhantomData<T>,
}

impl<'a, W, T> ListSerializer<'a, W, T>
where
    W: Write,
{
    pub fn new(serializer: Serializer<'a, W>) -> Self {
        Self {
            serializer,
            _marker: PhantomData,
        }
    }

    pub fn start_list(&mut self) -> Result<(), AsonError> {
        self.serializer.is_first_element = true;
        self.serializer.print_opening_bracket()
    }

    pub fn end_list(&mut self) -> Result<(), AsonError> {
        self.serializer.print_closing_bracket()
    }

    pub fn serialize_element(&mut self, value: &T) -> Result<(), AsonError>
    where
        T: Serialize,
    {
        if self.serializer.is_first_element {
            self.serializer.is_first_element = false;
        } else {
            self.serializer.print_newline()?;
        }

        value.serialize(&mut self.serializer)
    }
}

impl<'a, W> ser::Serializer for &mut Serializer<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = AsonError;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<(), AsonError> {
        self.print_str(if v { "true" } else { "false" })
    }

    fn serialize_i8(self, v: i8) -> Result<(), AsonError> {
        self.print_str(&format!("{}_i8", v))
    }

    fn serialize_i16(self, v: i16) -> Result<(), AsonError> {
        self.print_str(&format!("{}_i16", v))
    }

    fn serialize_i32(self, v: i32) -> Result<(), AsonError> {
        // `i32` is the default type for integer numbers,
        // so the suffix `_i32` can be omitted.
        self.print_str(&format!("{}", v))
    }

    fn serialize_i64(self, v: i64) -> Result<(), AsonError> {
        self.print_str(&format!("{}_i64", v))
    }

    fn serialize_u8(self, v: u8) -> Result<(), AsonError> {
        self.print_str(&format!("{}_u8", v))
    }

    fn serialize_u16(self, v: u16) -> Result<(), AsonError> {
        self.print_str(&format!("{}_u16", v))
    }

    fn serialize_u32(self, v: u32) -> Result<(), AsonError> {
        self.print_str(&format!("{}_u32", v))
    }

    fn serialize_u64(self, v: u64) -> Result<(), AsonError> {
        self.print_str(&format!("{}_u64", v))
    }

    fn serialize_f32(self, v: f32) -> Result<(), AsonError> {
        let str = if v.is_nan() {
            "NaN_f32".to_owned()
        } else if v == f32::INFINITY {
            "Inf_f32".to_owned()
        } else if v == f32::NEG_INFINITY {
            "-Inf_f32".to_owned()
        } else {
            format!("{}_f32", v)
        };

        self.print_str(&str)
    }

    fn serialize_f64(self, v: f64) -> Result<(), AsonError> {
        // `f64` is the default type for floating-point numbers,
        // so the suffix `_f64` can be omitted.

        let str = if v.is_nan() {
            "NaN".to_owned()
        } else if v == f64::INFINITY {
            "Inf".to_owned()
        } else if v == f64::NEG_INFINITY {
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
        };
        self.print_str(&str)
    }

    fn serialize_char(self, v: char) -> Result<(), AsonError> {
        // escape single char
        let str = match v {
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
            _ => v.to_string(),
        };

        self.print_str(&format!("'{}'", str))
    }

    fn serialize_str(self, v: &str) -> Result<(), AsonError> {
        let str = v
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

        self.print_str(&format!("\"{}\"", str))
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
    /// This format is designed to be human-readable and easy to parse.
    fn serialize_bytes(self, v: &[u8]) -> Result<(), AsonError> {
        let leading_space_chars = DEFAULT_INDENT_CHARS.repeat(self.indent_level);
        let line_separator = format!("\n{}", leading_space_chars);
        let str = v
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

        self.print_str(&format!("h\"{}\"", str))
    }

    fn serialize_none(self) -> Result<(), AsonError> {
        self.print_str("Option::None")
    }

    fn serialize_some<T>(self, value: &T) -> Result<(), AsonError>
    where
        T: ?Sized + Serialize,
    {
        self.print_str("Option::Some")?;
        self.print_opening_parenthesis()?;
        value.serialize(&mut *self)?;
        self.print_closing_parenthesis()?;
        Ok(())
    }

    fn serialize_unit(self) -> Result<(), AsonError> {
        // The type of `()` in Rust.
        Err(AsonError::Message("Does not support Unit.".to_owned()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), AsonError> {
        // A unit struct is a struct that has no fields, for example `struct Unit;`.
        Err(AsonError::Message(
            "Does not support \"Unit\" style Struct.".to_owned(),
        ))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), AsonError> {
        self.print_str(&format!("{}::{}", name, variant))
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<(), AsonError>
    where
        T: ?Sized + Serialize,
    {
        // A newtype struct is a tuple struct with a single field, for example `struct NewType(u8);`.
        Err(AsonError::Message(
            "Does not support \"New-Type\" style Struct.".to_owned(),
        ))
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), AsonError>
    where
        T: ?Sized + Serialize,
    {
        // ```rust
        // enum Name {
        //     Variant(type)
        // }
        // ```

        self.print_str(&format!("{}::{}", name, variant))?;
        self.print_opening_parenthesis()?;
        value.serialize(&mut *self)?;
        self.print_closing_parenthesis()?;
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, AsonError> {
        // `[...]`
        self.print_opening_bracket()?;
        self.is_first_element = true;
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, AsonError> {
        // Note that the Rust fixed length arrays
        // will be treated as tuples, e.g.
        // [i32; 4]
        //
        // per: https://serde.rs/data-model.html

        // `(...)`
        self.print_opening_parenthesis()?;
        self.is_first_element = true;
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, AsonError> {
        // A tuple struct is a struct that looks like a tuple, for example `struct TupleStruct(u8, String);`.
        Err(AsonError::Message(
            "Does not support \"Tuple\" style Struct.".to_owned(),
        ))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, AsonError> {
        // ```rust
        // enum Name {
        //     Variant(type, type, ...)
        // }
        // ```

        self.print_str(&format!("{}::{}", name, variant))?;
        self.print_opening_parenthesis()?;
        self.is_first_element = true;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, AsonError> {
        // `[key: value, ...]`
        self.print_opening_bracket()?;
        self.is_first_element = true;
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, AsonError> {
        // `{key: value, ...}`
        self.print_opening_brace()?;
        self.is_first_element = true;
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, AsonError> {
        // ```rust
        // enum Name {
        //     Variant{key: value, ...}
        // }
        // ```

        self.print_str(&format!("{}::{}", name, variant))?;
        self.print_opening_brace()?;
        self.is_first_element = true;
        Ok(self)
    }
}

impl<W> ser::SerializeSeq for &mut Serializer<'_, W>
where
    W: Write,
{
    type Ok = ();
    type Error = AsonError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), AsonError>
    where
        T: ?Sized + Serialize,
    {
        if self.is_first_element {
            self.is_first_element = false;
        } else {
            self.print_newline()?;
        }

        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), AsonError> {
        self.print_closing_bracket()
    }
}

impl<W> ser::SerializeTuple for &mut Serializer<'_, W>
where
    W: Write,
{
    type Ok = ();
    type Error = AsonError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), AsonError>
    where
        T: ?Sized + Serialize,
    {
        if self.is_first_element {
            self.is_first_element = false;
        } else {
            self.print_space()?;
        }

        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), AsonError> {
        self.print_closing_parenthesis()
    }
}

impl<W> ser::SerializeTupleStruct for &mut Serializer<'_, W>
where
    W: Write,
{
    type Ok = ();
    type Error = AsonError;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<(), AsonError>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<(), AsonError> {
        unreachable!()
    }
}

impl<W> ser::SerializeTupleVariant for &mut Serializer<'_, W>
where
    W: Write,
{
    type Ok = ();
    type Error = AsonError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), AsonError>
    where
        T: ?Sized + Serialize,
    {
        if self.is_first_element {
            self.is_first_element = false;
        } else {
            self.print_space()?;
        }

        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), AsonError> {
        self.print_closing_parenthesis()
    }
}

impl<W> ser::SerializeMap for &mut Serializer<'_, W>
where
    W: Write,
{
    type Ok = ();
    type Error = AsonError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), AsonError>
    where
        T: ?Sized + Serialize,
    {
        if self.is_first_element {
            self.is_first_element = false;
        } else {
            self.print_newline()?;
        }

        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), AsonError>
    where
        T: ?Sized + Serialize,
    {
        self.print_str(": ")?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), AsonError> {
        self.print_closing_bracket()
    }
}

impl<W> ser::SerializeStruct for &mut Serializer<'_, W>
where
    W: Write,
{
    type Ok = ();
    type Error = AsonError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), AsonError>
    where
        T: ?Sized + Serialize,
    {
        if self.is_first_element {
            self.is_first_element = false;
        } else {
            self.print_newline()?;
        }

        self.print_str(key)?;
        self.print_str(": ")?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), AsonError> {
        self.print_closing_brace()
    }
}

impl<W> ser::SerializeStructVariant for &mut Serializer<'_, W>
where
    W: Write,
{
    type Ok = ();
    type Error = AsonError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), AsonError>
    where
        T: ?Sized + Serialize,
    {
        if self.is_first_element {
            self.is_first_element = false;
        } else {
            self.print_newline()?;
        }

        self.print_str(key)?;
        self.print_str(": ")?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), AsonError> {
        self.print_closing_brace()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;
    use serde::Serialize;
    use serde_bytes::ByteBuf;

    use crate::ser::{list_to_writer, ser_to_string};

    #[test]
    fn test_primitive_values() {
        // bool
        {
            let v0: bool = false;
            assert_eq!(ser_to_string(&v0).unwrap(), r#"false"#);

            let v1: bool = true;
            assert_eq!(ser_to_string(&v1).unwrap(), r#"true"#);
        }

        // signed integers
        {
            let v0: i8 = 11;
            assert_eq!(ser_to_string(&v0).unwrap(), r#"11_i8"#);

            let v1: i16 = 13;
            assert_eq!(ser_to_string(&v1).unwrap(), r#"13_i16"#);

            let v2: i32 = 17;
            assert_eq!(ser_to_string(&v2).unwrap(), r#"17"#);

            let v3: i64 = 19;
            assert_eq!(ser_to_string(&v3).unwrap(), r#"19_i64"#);
        }

        // unsigned integers
        {
            let v0: u8 = 11;
            assert_eq!(ser_to_string(&v0).unwrap(), r#"11_u8"#);

            let v1: u16 = 13;
            assert_eq!(ser_to_string(&v1).unwrap(), r#"13_u16"#);

            let v2: u32 = 17;
            assert_eq!(ser_to_string(&v2).unwrap(), r#"17_u32"#);

            let v3: u64 = 19;
            assert_eq!(ser_to_string(&v3).unwrap(), r#"19_u64"#);
        }

        // floating-point f32
        {
            let v0: f32 = 123_f32;
            assert_eq!(ser_to_string(&v0).unwrap(), r#"123_f32"#);

            let v1: f32 = -4.56_f32;
            assert_eq!(ser_to_string(&v1).unwrap(), r#"-4.56_f32"#);

            let v2: f32 = std::f32::consts::PI;
            assert_eq!(ser_to_string(&v2).unwrap(), r#"3.1415927_f32"#);

            let v3: f32 = 0f32;
            assert_eq!(ser_to_string(&v3).unwrap(), r#"0_f32"#);

            let v4: f32 = -0f32;
            assert_eq!(ser_to_string(&v4).unwrap(), r#"-0_f32"#);

            assert_eq!(ser_to_string(&f32::NAN).unwrap(), r#"NaN_f32"#);
            assert_eq!(ser_to_string(&f32::INFINITY).unwrap(), r#"Inf_f32"#);
            assert_eq!(ser_to_string(&f32::NEG_INFINITY).unwrap(), r#"-Inf_f32"#);
        }

        // floating-point f64
        {
            let v0: f64 = 123_f64;
            assert_eq!(ser_to_string(&v0).unwrap(), r#"123.0"#);

            let v1: f64 = -4.56_f64;
            assert_eq!(ser_to_string(&v1).unwrap(), r#"-4.56"#);

            let v2: f64 = std::f64::consts::E;
            assert_eq!(ser_to_string(&v2).unwrap(), r#"2.718281828459045"#);

            let v3: f64 = 0f64;
            assert_eq!(ser_to_string(&v3).unwrap(), r#"0.0"#);

            let v4: f64 = -0f64;
            assert_eq!(ser_to_string(&v4).unwrap(), r#"-0.0"#);

            assert_eq!(ser_to_string(&f64::NAN).unwrap(), r#"NaN"#);
            assert_eq!(ser_to_string(&f64::INFINITY).unwrap(), r#"Inf"#);
            assert_eq!(ser_to_string(&f64::NEG_INFINITY).unwrap(), r#"-Inf"#);
        }

        // char
        {
            assert_eq!(ser_to_string(&'a').unwrap(), r#"'a'"#);
            assert_eq!(ser_to_string(&'文').unwrap(), r#"'文'"#);
            assert_eq!(ser_to_string(&'🍒').unwrap(), r#"'🍒'"#);

            // escaped characters
            assert_eq!(ser_to_string(&'\\').unwrap(), r#"'\\'"#);
            assert_eq!(ser_to_string(&'\'').unwrap(), r#"'\''"#);

            // double quote does not necessary to be escaped
            assert_eq!(ser_to_string(&'"').unwrap(), r#"'"'"#);
            assert_eq!(ser_to_string(&'\"').unwrap(), r#"'"'"#);

            assert_eq!(ser_to_string(&'\t').unwrap(), r#"'\t'"#);
            assert_eq!(ser_to_string(&'\r').unwrap(), r#"'\r'"#);
            assert_eq!(ser_to_string(&'\n').unwrap(), r#"'\n'"#);
            assert_eq!(ser_to_string(&'\0').unwrap(), r#"'\0'"#);

            assert_eq!(ser_to_string(&'萱').unwrap(), r#"'萱'"#);
        }

        // string
        {
            assert_eq!(ser_to_string(&"abc文字🍒").unwrap(), r#""abc文字🍒""#);
            assert_eq!(
                ser_to_string(&"abc\"\'\\\t\0xyz").unwrap(),
                r#""abc\"'\\\t\0xyz""#
            );
            assert_eq!(
                ser_to_string(&"hello\r\nworld").unwrap(),
                "\"hello\r\nworld\""
            );
        }
    }

    #[test]
    fn test_hexadecimal_byte_data() {
        let v0 = vec![11u8, 13, 17, 19];
        let v0b = ByteBuf::from(v0);
        assert_eq!(ser_to_string(&v0b).unwrap(), r#"h"0b 0d 11 13""#);

        let v1 = b"abc";
        let v1b = ByteBuf::from(v1);
        assert_eq!(ser_to_string(&v1b).unwrap(), r#"h"61 62 63""#);
    }

    #[test]
    fn test_option() {
        let v0: Option<i32> = None;
        assert_eq!(ser_to_string(&v0).unwrap(), r#"Option::None"#);

        let v1: Option<i32> = Some(123);
        assert_eq!(ser_to_string(&v1).unwrap(), r#"Option::Some(123)"#);
    }

    #[test]
    fn test_list() {
        assert_eq!(
            ser_to_string(&vec![11, 13, 17, 19]).unwrap(),
            r#"[
    11
    13
    17
    19
]"#
        );

        assert_eq!(
            ser_to_string(&"abc".as_bytes()).unwrap(),
            r#"[
    97_u8
    98_u8
    99_u8
]"#
        );

        assert_eq!(
            ser_to_string(&vec!["foo", "bar", "2024"]).unwrap(),
            r#"[
    "foo"
    "bar"
    "2024"
]"#
        );

        // nested seq

        assert_eq!(
            ser_to_string(&vec![vec![11, 13], vec![17, 19], vec![23, 29]]).unwrap(),
            r#"[
    [
        11
        13
    ]
    [
        17
        19
    ]
    [
        23
        29
    ]
]"#
        );
    }

    #[test]
    fn test_tuple() {
        assert_eq!(
            ser_to_string(&(11, 13, 17, 19)).unwrap(),
            r#"(11 13 17 19)"#
        );

        // a fixed-length array is treated as tuple
        assert_eq!(ser_to_string(b"abc").unwrap(), r#"(97_u8 98_u8 99_u8)"#);

        assert_eq!(
            ser_to_string(&("foo", "bar", "2024")).unwrap(),
            r#"("foo" "bar" "2024")"#
        );

        // nested tuple
        assert_eq!(
            ser_to_string(&((11, 13), (17, 19), (23, 29))).unwrap(),
            r#"((11 13) (17 19) (23 29))"#
        );
    }

    #[test]
    fn test_object() {
        #[derive(Serialize)]
        struct Object {
            id: i32,
            name: String,
            checked: bool,
        }

        let v0 = Object {
            id: 123,
            name: "foo".to_owned(),
            checked: true,
        };

        let expected0 = r#"{
    id: 123
    name: "foo"
    checked: true
}"#;
        assert_eq!(ser_to_string(&v0).unwrap(), expected0);

        // nested object
        #[derive(Serialize)]
        struct Address {
            code: i32,
            city: String,
        }

        #[derive(Serialize)]
        struct NestedObject {
            id: i32,
            name: String,
            address: Box<Address>,
        }

        let v1 = NestedObject {
            id: 456,
            name: "bar".to_owned(),
            address: Box::new(Address {
                code: 518000,
                city: "sz".to_owned(),
            }),
        };

        let expected1 = r#"{
    id: 456
    name: "bar"
    address: {
        code: 518000
        city: "sz"
    }
}"#;

        assert_eq!(ser_to_string(&v1).unwrap(), expected1);
    }

    #[test]
    fn test_named_list() {
        let mut m0 = HashMap::<String, String>::new();
        m0.insert("red".to_owned(), "0xff0000".to_owned());
        m0.insert("green".to_owned(), "0x00ff00".to_owned());
        m0.insert("blue".to_owned(), "0x0000ff".to_owned());

        // the order of the key-value pairs in the output string is not guaranteed,
        // because the `HashMap` does not guarantee the order of the key-value pairs.
        // so we just check if the output string contains the expected key-value pairs.
        let s0 = ser_to_string(&m0).unwrap();
        assert!(s0.starts_with('['));
        assert!(s0.ends_with(']'));
        assert!(s0.contains(r#""red": "0xff0000""#));
        assert!(s0.contains(r#""green": "0x00ff00""#));
        assert!(s0.contains(r#""blue": "0x0000ff""#));

        let mut m1 = HashMap::<i32, Option<String>>::new();
        m1.insert(223, Some("hello".to_owned()));
        m1.insert(227, None);
        m1.insert(229, Some("world".to_owned()));

        // the order of the key-value pairs in the output string is not guaranteed,
        // because the `HashMap` does not guarantee the order of the key-value pairs.
        // so we just check if the output string contains the expected key-value pairs.
        let s1 = ser_to_string(&m1).unwrap();
        assert!(s1.starts_with('['));
        assert!(s1.ends_with(']'));
        assert!(s1.contains(r#"223: Option::Some("hello")"#));
        assert!(s1.contains(r#"227: Option::None"#));
        assert!(s1.contains(r#"229: Option::Some("world")"#));
    }

    #[test]
    fn test_enum() {
        #[derive(Serialize)]
        enum Color {
            Red,
            Green,
            Blue,
        }

        let v0 = Color::Red;
        assert_eq!(ser_to_string(&v0).unwrap(), r#"Color::Red"#);

        let v1 = Color::Green;
        assert_eq!(ser_to_string(&v1).unwrap(), r#"Color::Green"#);

        let v2 = Color::Blue;
        assert_eq!(ser_to_string(&v2).unwrap(), r#"Color::Blue"#);
    }

    #[test]
    fn test_variant_with_primitive_value() {
        #[derive(Serialize)]
        enum Color {
            Red,

            #[allow(dead_code)]
            Green,
            Blue,
            Grey(u8),
        }

        {
            let v0 = Color::Red;
            assert_eq!(ser_to_string(&v0).unwrap(), r#"Color::Red"#);

            let v1 = Color::Grey(11);
            assert_eq!(ser_to_string(&v1).unwrap(), r#"Color::Grey(11_u8)"#);
        }

        // nested
        #[derive(Serialize)]
        enum Apperance {
            Transparent,
            Color(Color),
        }

        {
            let v0 = Apperance::Transparent;
            assert_eq!(ser_to_string(&v0).unwrap(), r#"Apperance::Transparent"#);

            let v1 = Apperance::Color(Color::Blue);
            assert_eq!(
                ser_to_string(&v1).unwrap(),
                r#"Apperance::Color(Color::Blue)"#
            );

            let v2 = Apperance::Color(Color::Grey(13));
            assert_eq!(
                ser_to_string(&v2).unwrap(),
                r#"Apperance::Color(Color::Grey(13_u8))"#
            );
        }
    }

    #[test]
    fn test_variant_with_list_value() {
        #[derive(Serialize)]
        enum Item {
            Empty,
            List(Vec<i32>),
        }

        assert_eq!(
            ser_to_string(&vec![Item::Empty, Item::List(vec![11, 13])]).unwrap(),
            r#"[
    Item::Empty
    Item::List([
        11
        13
    ])
]"#
        );
    }

    #[test]
    fn test_variant_with_object_value() {
        #[derive(Serialize)]
        struct Object {
            id: i32,
            name: String,
        }

        #[derive(Serialize)]
        enum Item {
            Empty,
            Object(Object),
        }

        assert_eq!(
            ser_to_string(&vec![
                Item::Empty,
                Item::Object(Object {
                    id: 11,
                    name: "foo".to_owned()
                })
            ])
            .unwrap(),
            r#"[
    Item::Empty
    Item::Object({
        id: 11
        name: "foo"
    })
]"#
        );
    }

    #[test]
    fn test_tuple_like_variant() {
        #[allow(clippy::upper_case_acronyms)]
        #[derive(Serialize)]
        enum Color {
            Grey(u8),
            RGB(u8, u8, u8),
        }

        assert_eq!(
            ser_to_string(&Color::Grey(127)).unwrap(),
            r#"Color::Grey(127_u8)"#
        );

        assert_eq!(
            ser_to_string(&Color::RGB(255, 127, 63)).unwrap(),
            r#"Color::RGB(255_u8 127_u8 63_u8)"#
        );
    }

    #[test]
    fn test_object_like_variant() {
        #[derive(Serialize)]
        enum Shape {
            Circle(i32),
            Rect { width: i32, height: i32 },
        }

        assert_eq!(
            ser_to_string(&Shape::Circle(11)).unwrap(),
            r#"Shape::Circle(11)"#
        );

        assert_eq!(
            ser_to_string(&Shape::Rect {
                width: 200,
                height: 100
            })
            .unwrap(),
            r#"Shape::Rect{
    width: 200
    height: 100
}"#
        );
    }

    #[test]
    fn test_list_with_tuple_element() {
        assert_eq!(
            ser_to_string(&vec![(1, "foo"), (2, "bar")]).unwrap(),
            r#"[
    (1 "foo")
    (2 "bar")
]"#
        );

        assert_eq!(
            ser_to_string(&(vec![11, 13], vec!["foo", "bar"])).unwrap(),
            r#"([
    11
    13
] [
    "foo"
    "bar"
])"#
        );
    }

    #[test]
    fn test_list_with_object_elements() {
        #[derive(Serialize)]
        struct Object {
            id: i32,
            name: String,
        }

        assert_eq!(
            ser_to_string(&vec![
                Object {
                    id: 11,
                    name: "foo".to_owned()
                },
                Object {
                    id: 13,
                    name: "bar".to_owned()
                }
            ])
            .unwrap(),
            r#"[
    {
        id: 11
        name: "foo"
    }
    {
        id: 13
        name: "bar"
    }
]"#
        );
    }

    #[test]
    fn test_object_with_list_field() {
        #[derive(Serialize)]
        struct ObjectList {
            id: i32,
            items: Vec<i32>,
        }

        assert_eq!(
            ser_to_string(&ObjectList {
                id: 456,
                items: vec![11, 13, 17, 19]
            })
            .unwrap(),
            r#"{
    id: 456
    items: [
        11
        13
        17
        19
    ]
}"#
        );
    }

    #[test]
    fn test_tuple_with_object_elements() {
        #[derive(Serialize)]
        struct Object {
            id: i32,
            name: String,
        }

        assert_eq!(
            ser_to_string(&(
                123,
                Object {
                    id: 11,
                    name: "foo".to_owned()
                }
            ))
            .unwrap(),
            r#"(123 {
    id: 11
    name: "foo"
})"#
        );
    }

    #[test]
    fn test_object_with_tuple_field() {
        #[derive(Serialize)]
        struct ObjectDetail {
            id: i32,
            address: (i32, String),
        }

        assert_eq!(
            ser_to_string(&ObjectDetail {
                id: 456,
                address: (11, "sz".to_owned())
            })
            .unwrap(),
            r#"{
    id: 456
    address: (11 "sz")
}"#
        );
    }

    #[test]
    fn test_serialize_stream_list() {
        let mut buf: Vec<u8> = vec![];
        let mut ser = list_to_writer(&mut buf);

        ser.start_list().unwrap();
        ser.serialize_element(&11).unwrap();
        ser.serialize_element(&13).unwrap();
        ser.end_list().unwrap();

        let s = String::from_utf8(buf).unwrap();
        assert_eq!(
            s,
            r#"[
    11
    13
]"#
        );

        #[derive(Serialize, Debug, PartialEq)]
        struct Object {
            id: i32,
            name: String,
        }

        let mut buf: Vec<u8> = vec![];
        let mut ser = list_to_writer(&mut buf);

        ser.start_list().unwrap();
        ser.serialize_element(&Object {
            id: 11,
            name: "foo".to_owned(),
        })
        .unwrap();

        ser.serialize_element(&Object {
            id: 13,
            name: "bar".to_owned(),
        })
        .unwrap();

        ser.end_list().unwrap();

        let s = String::from_utf8(buf).unwrap();
        assert_eq!(
            s,
            r#"[
    {
        id: 11
        name: "foo"
    }
    {
        id: 13
        name: "bar"
    }
]"#
        )
    }
}
