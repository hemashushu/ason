// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use chrono::{DateTime, FixedOffset};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Number {
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
}

#[derive(Debug, PartialEq)]
pub struct KeyValuePair {
    pub key: String,
    pub value: Box<AsonNode>,
}

#[derive(Debug, PartialEq)]
pub struct NamedListEntry {
    pub name: Box<AsonNode>,
    pub value: Box<AsonNode>,
}

#[derive(Debug, PartialEq)]
pub struct Variant {
    // Variant type name.
    // e.g., the "Option" of "Option::None"
    pub type_name: String,

    // Variant member name.
    // e.g., the "None" of "Option::None"
    pub member_name: String,

    pub value: VariantValue,
}

#[derive(Debug, PartialEq)]
pub enum VariantValue {
    Empty,                     // No value variant
    Value(Box<AsonNode>),      // Single value variant
    Tuple(Vec<AsonNode>),      // tuple-like variant
    Object(Vec<KeyValuePair>), // object-like variant
}

#[derive(Debug, PartialEq)]
pub enum AsonNode {
    Number(Number),
    Boolean(bool),
    Char(char),
    String(String),
    DateTime(DateTime<FixedOffset>),

    Variant(Variant),
    ByteData(Vec<u8>),
    List(Vec<AsonNode>),
    NamedList(Vec<NamedListEntry>),
    Tuple(Vec<AsonNode>),
    Object(Vec<KeyValuePair>),
}

impl KeyValuePair {
    pub fn new(key: &str, value: AsonNode) -> Self {
        Self {
            key: key.to_owned(),
            value: Box::new(value),
        }
    }
}

impl Variant {
    pub fn new(type_name: &str, member_name: &str) -> Self {
        Self {
            type_name: type_name.to_owned(),
            member_name: member_name.to_owned(),
            value: VariantValue::Empty,
        }
    }

    pub fn with_value(type_name: &str, member_name: &str, value: AsonNode) -> Self {
        Self {
            type_name: type_name.to_owned(),
            member_name: member_name.to_owned(),
            value: VariantValue::Value(Box::new(value)),
        }
    }

    pub fn with_tuple_like(type_name: &str, member_name: &str, values: Vec<AsonNode>) -> Self {
        Self {
            type_name: type_name.to_owned(),
            member_name: member_name.to_owned(),
            value: VariantValue::Tuple(values),
        }
    }

    pub fn with_object_like(
        type_name: &str,
        member_name: &str,
        key_value_pairs: Vec<KeyValuePair>,
    ) -> Self {
        Self {
            type_name: type_name.to_owned(),
            member_name: member_name.to_owned(),
            value: VariantValue::Object(key_value_pairs),
        }
    }
}
