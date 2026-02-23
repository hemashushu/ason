// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

mod char_with_position;
mod lexer;
mod normalizer;
mod parser;
mod peekable_iterator;
mod position;
mod range;
mod token;
mod token_stream_reader;
mod utf8_char_iterator;
// mod printer;

pub mod ast;
pub mod error;
pub mod error_printer;

pub use parser::parse_from_reader;
pub use parser::parse_from_string;
pub use token_stream_reader::stream_from_reader;
pub use token_stream_reader::stream_from_string;

// pub use parser::parse_from_str;
// pub use printer::print_to_string;
// pub use printer::print_to_writer;
