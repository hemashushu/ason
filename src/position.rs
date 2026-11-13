// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

/// A zero-based location in ASON source text.
///
/// The index and column count Unicode scalar values, not UTF-8 bytes.
/// A line feed advances the line and resets the column.
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Position {
    /// The absolute character index in the source, starting at zero.
    pub index: usize,
    /// The line number, starting at zero.
    pub line: usize,
    /// The character column within the line, starting at zero.
    pub column: usize,
}

impl Position {
    /// Creates a position from zero-based character index, line, and column.
    pub fn new(index: usize, line: usize, column: usize) -> Self {
        Self {
            index,
            line,
            column,
        }
    }
}
