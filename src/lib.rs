// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

mod char_with_position;
mod error_printer;
mod lexer;
mod normalizer;
mod peekable_iter;
mod position;
mod range;
mod token;
mod utf8_char_stream;

mod parser;
mod printer;

pub mod ast;
pub mod error;

pub use parser::parse_from_reader;
pub use parser::parse_from_str;
pub use printer::print_to_string;
pub use printer::print_to_writer;

// pub use serde::de::from_reader;
// pub use serde::de::from_str;
// pub use serde::ser::to_string;
// pub use serde::ser::to_writer;

// pub use serde::serde_date::Date;
