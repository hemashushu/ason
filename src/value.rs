// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use chrono::{DateTime, FixedOffset};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bool(bool),
    Char(char),
    String(String),
    DateTime(DateTime<FixedOffset>),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Tuple(Vec<Value>),
    Object(Vec<(String, Value)>),
    Enumeration {
        name: String,
        variant: String,
        payload: VariantValue,
    },
    Option(Option<Box<Value>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariantValue {
    Unit,
    Newtype(Box<Value>),
    Tuple(Vec<Value>),
    Struct(Vec<(String, Value)>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum PayloadKind {
    Unit,
    Newtype,
    Tuple,
    Struct,
}

impl VariantValue {
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
