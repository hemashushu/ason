// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use crate::position::Position;

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Range {
    pub start: Position,
    pub end_included: Position,
}

impl Range {
    pub fn new(start: &Position, end_included: &Position) -> Self {
        Self {
            start: *start,
            end_included: *end_included,
        }
    }

    /// Creates a Range that represents a single position (i.e., start and end are the same).
    pub fn from_single_position(pos: &Position) -> Self {
        Self {
            start: *pos,
            end_included: *pos,
        }
    }

    /// Creates a Range from a starting position and a length,
    /// where the end position is calculated based on the length and
    /// assumes that the range is on the same line (i.e., only the column is incremented).
    pub fn from_position_and_length(pos: &Position, length: usize) -> Self {
        let inc = length - 1;
        Self {
            start: *pos,
            end_included: Position::new(pos.index + inc, pos.line, pos.column + inc),
        }
    }

    /// Creates a Range from detailed parameters, including the
    /// starting index, line, column, and length.
    /// The end position is calculated based on the starting position and length,
    /// assuming that the range is on the same line.
    pub fn from_detail(index: usize, line: usize, column: usize, length: usize) -> Self {
        let inc = length - 1;
        let start = Position::new(index, line, column);
        let end_included = Position::new(index + inc, line, column + inc);
        Self {
            start,
            end_included,
        }
    }

    /// Merges two Ranges into a single Range that spans from the start of the left Range
    /// to the end of the right Range. This is useful for combining adjacent or overlapping ranges
    pub fn merge(left: &Self, right: &Self) -> Self {
        Self {
            start: left.start,
            end_included: right.end_included,
        }
    }
}
