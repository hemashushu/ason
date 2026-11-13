// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

//! Serialization and deserialization for the ASON text format.
//!
//! Use [`de_from_str`] or [`de_from_reader`] to deserialize into a known Serde
//! type, and [`ser_to_string`] or [`ser_to_writer`] to serialize it. For data
//! whose structure is not known in advance, the `de_value_*` and `ser_value_*`
//! functions work with an owned [`Value`] tree instead.
//!
//! The streaming APIs process root Lists, or Results whose `Ok` payload is a
//! List, one element at a time. [`syntax::tokenize`] provides lossless lexical
//! spans when the original source text must be retained.
//!
//! ```
//! let values: Vec<i32> = ason::de_from_str("[1 2 3]")?;
//! assert_eq!(values, vec![1, 2, 3]);
//! # Ok::<(), ason::AsonError>(())
//! ```

mod char_with_position;
mod de;
mod error;
mod error_printer;
mod lexer;
mod normalizer;
mod peekable_iterator;
mod position;
mod range;
mod ser;
mod token;
mod utf8_char_iterator;

pub mod array;
pub mod date_time;
pub mod syntax;
mod value;

pub use error::AsonError;
pub use value::{Value, VariantValue};

pub use de::{
    ListStreamReader, ResultStreamReader, de_from_reader, de_from_str, de_value_from_reader,
    de_value_from_str, list_stream_reader, result_stream_reader,
};

pub use ser::{
    ListStreamWriter, ResultStreamWriter, list_stream_writer, result_stream_writer, ser_to_string,
    ser_to_writer, ser_value_to_string, ser_value_to_writer,
};
