// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

//! Owned dynamic representations of ASON values and enumeration payloads.
//!
//! Use the crate's `de_value_*` functions to parse these values and the
//! `ser_value_*` functions to write them without defining a Serde data type.

use chrono::{DateTime, FixedOffset};

/// An owned ASON value whose structure can be inspected at runtime.
///
/// Numeric variants retain their ASON type; Lists, Maps, Tuples, Objects, and
/// enumerations have distinct representations. Parsed Object fields and Map
/// entries retain their source order. Comments, whitespace, and original
/// literal spelling are not retained.
///
/// A parsed `Option` uses [`Value::Option`]; other enumerations, including
/// `Result`, use [`Value::Enumeration`]. Because both an empty List and an
/// empty Map are written as `[]`, parsing that text produces an empty List.
/// Manually constructed values can contain shapes that the serializer rejects,
/// such as an empty Tuple. Floating-point equality follows Rust's usual
/// semantics, including `NaN != NaN`.
///
/// This type does not implement Serde's `Serialize` or `Deserialize`. Use
/// [`crate::de_value_from_str`], [`crate::de_value_from_reader`],
/// [`crate::ser_value_to_string`], or [`crate::ser_value_to_writer`].
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A signed 8-bit integer.
    I8(i8),
    /// A signed 16-bit integer.
    I16(i16),
    /// A signed 32-bit integer, the default ASON integer type.
    I32(i32),
    /// A signed 64-bit integer.
    I64(i64),
    /// An unsigned 8-bit integer.
    U8(u8),
    /// An unsigned 16-bit integer.
    U16(u16),
    /// An unsigned 32-bit integer.
    U32(u32),
    /// An unsigned 64-bit integer.
    U64(u64),
    /// A 32-bit floating-point number.
    F32(f32),
    /// A 64-bit floating-point number, the default ASON floating-point type.
    F64(f64),
    /// A Boolean value.
    Bool(bool),
    /// A Unicode scalar value.
    Char(char),
    /// An owned string.
    String(String),
    /// A date and time with a fixed UTC offset.
    DateTime(DateTime<FixedOffset>),
    /// Hexadecimal byte data.
    Bytes(Vec<u8>),
    /// An ordered List of values.
    List(Vec<Value>),
    /// An ordered Map of key-value pairs.
    Map(Vec<(Value, Value)>),
    /// An ordered, nonempty Tuple of values.
    ///
    /// An empty vector can be constructed but cannot be serialized as a Tuple.
    Tuple(Vec<Value>),
    /// An ordered Object of named fields.
    ///
    /// Parsing rejects duplicate field names.
    Object(Vec<(String, Value)>),
    /// An enumeration with its type name, variant name, and payload.
    Enumeration {
        /// The enumeration's type name, such as `Result`.
        name: String,
        /// The variant name, such as `Ok`.
        variant: String,
        /// The variant's unit, newtype, tuple, or struct payload.
        payload: VariantValue,
    },
    /// An `Option::None` or an `Option::Some` containing an owned value.
    Option(Option<Box<Value>>),
}

/// The payload carried by an ASON enumeration variant.
///
/// Parsing distinguishes a single parenthesized value from multiple values:
/// the former becomes [`VariantValue::Newtype`], the latter
/// [`VariantValue::Tuple`].
#[derive(Debug, Clone, PartialEq)]
pub enum VariantValue {
    /// A variant without a payload, such as `Color::Red`.
    Unit,
    /// A variant with one value, such as `Result::Ok(42)`.
    Newtype(Box<Value>),
    /// A variant with multiple positional values.
    ///
    /// An empty vector cannot be serialized. A one-element vector is written
    /// as a single-value payload and is parsed back as `Newtype`.
    Tuple(Vec<Value>),
    /// A variant with an ordered collection of named fields.
    Struct(Vec<(String, Value)>),
}

/// The payload shape used to validate predefined enumerations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum PayloadKind {
    /// No payload.
    Unit,
    /// One value.
    Newtype,
    /// Positional values.
    Tuple,
    /// Named fields.
    Struct,
}

impl VariantValue {
    /// Returns the shape of this variant's payload.
    pub(crate) fn kind(&self) -> PayloadKind {
        match self {
            VariantValue::Unit => PayloadKind::Unit,
            VariantValue::Newtype(_) => PayloadKind::Newtype,
            VariantValue::Tuple(_) => PayloadKind::Tuple,
            VariantValue::Struct(_) => PayloadKind::Struct,
        }
    }
}

/// Validates the predefined Enumerations `Option` and `Result`.
///
/// `None` must have no payload; `Some`, `Ok` and `Err` must carry exactly one value.
/// Other type names are not checked.
pub(crate) fn validate_predefined_enumeration(
    name: &str,
    variant: &str,
    kind: PayloadKind,
) -> Result<(), String> {
    let expected = match (name, variant) {
        ("Option", "None") => PayloadKind::Unit,
        ("Option", "Some") | ("Result", "Ok" | "Err") => PayloadKind::Newtype,
        ("Option", _) => return Err("Expect \"Option::None\" or \"Option::Some(...)\".".to_owned()),
        ("Result", _) => {
            return Err("Expect \"Result::Ok(...)\" or \"Result::Err(...)\".".to_owned());
        }
        _ => return Ok(()),
    };

    if kind == expected {
        Ok(())
    } else if expected == PayloadKind::Unit {
        Err(format!("\"{name}::{variant}\" must not have a value."))
    } else {
        Err(format!(
            "\"{name}::{variant}\" must have exactly one value."
        ))
    }
}
