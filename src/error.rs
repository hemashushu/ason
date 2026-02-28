// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::fmt::{self, Display};

use crate::{position::Position, range::Range};

#[derive(Debug, PartialEq, Clone)]
pub enum AsonError {
    Message(String),
    UnexpectedEndOfDocument(String),
    MessageWithPosition(String, Position),
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
                    range.end_included.line + 1,
                    range.end_included.column + 1
                )?;
                write!(f, "{}", detail)
            }
        }
    }
}

impl std::error::Error for AsonError {}
