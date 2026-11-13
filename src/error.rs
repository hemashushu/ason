// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::fmt::{self, Display};

use crate::{position::Position, range::Range};

/// An error encountered while reading, parsing, or writing ASON.
///
/// Errors may contain a source position or an inclusive source range.
/// Positions count Unicode scalar values and use zero-based lines and columns;
/// the [`Display`] representation reports lines and columns starting at one.
/// Use [`AsonError::with_source`] to include an annotated source snippet.
#[derive(Debug, PartialEq, Clone)]
pub enum AsonError {
    /// An error message without a source location.
    Message(String),
    /// The input ended before the current construct was complete.
    UnexpectedEndOfDocument(String),
    /// An error message and the position of the relevant character.
    MessageWithPosition(String, Position),
    /// An error message and the inclusive range of the relevant source text.
    MessageWithRange(String, Range),
}

impl Display for AsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AsonError::Message(msg) => f.write_str(msg),
            AsonError::UnexpectedEndOfDocument(detail) => {
                writeln!(f, "Unexpected to reach the end of document.")?;
                write!(f, "{}", detail)
            }
            AsonError::MessageWithPosition(detail, position) => {
                writeln!(
                    f,
                    "Error at line: {} column: {}",
                    position.line + 1,
                    position.column + 1
                )?;
                write!(f, "{}", detail)
            }
            AsonError::MessageWithRange(detail, range) => {
                writeln!(
                    f,
                    "Error from line: {} column: {}, to line: {} column: {}",
                    range.start.line + 1,
                    range.start.column + 1,
                    range.end_inclusive.line + 1,
                    range.end_inclusive.column + 1
                )?;
                write!(f, "{}", detail)
            }
        }
    }
}

impl std::error::Error for AsonError {}

impl serde::de::Error for AsonError {
    fn custom<T: Display>(msg: T) -> Self {
        AsonError::Message(msg.to_string())
    }
}

impl serde::ser::Error for AsonError {
    fn custom<T: Display>(msg: T) -> Self {
        AsonError::Message(msg.to_string())
    }
}
