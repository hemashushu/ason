// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Position {
    pub index: usize,  // The absolute character index in the source.
    pub line: usize,   // The line number in the source (starting from 0).
    pub column: usize, // The column number within the current line (starting from 0).
}

impl Position {
    /// Creates a new Position with the given character index, line, and column.
    pub fn new(index: usize, line: usize, column: usize) -> Self {
        Self {
            index,
            line,
            column,
        }
    }
}
