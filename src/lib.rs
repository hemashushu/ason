// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

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
pub mod value;

pub use de::{ListStreamReader, ResultStreamReader};
pub use error::AsonError;
pub use ser::{ListStreamWriter, ResultStreamWriter};
pub use value::{Value, VariantValue};

pub use de::{
    de_from_reader, de_from_str, de_value_from_reader, de_value_from_str, list_stream_reader,
    result_stream_reader,
};

pub use ser::{
    list_stream_writer, result_stream_writer, ser_to_string, ser_to_writer, ser_value_to_string,
    ser_value_to_writer,
};
