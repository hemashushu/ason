// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use crate::position::Position;

/// An inclusive range between two character positions in the source text.
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Range {
    /// The first character's position.
    pub start: Position,
    /// The last character's position, included in the range.
    pub end_inclusive: Position,
}

impl Range {
    /// Creates a range from its inclusive start and end positions.
    ///
    /// The positions are copied without checking their order.
    pub fn new(start: &Position, end_inclusive: &Position) -> Self {
        Self {
            start: *start,
            end_inclusive: *end_inclusive,
        }
    }

    /// Creates a range containing a single character position.
    pub fn from_single_position(pos: &Position) -> Self {
        Self {
            start: *pos,
            end_inclusive: *pos,
        }
    }

    /// Creates a same-line range from a starting position and character count.
    ///
    /// `length` must be greater than zero. The end index and column are
    /// advanced by `length - 1`; line breaks are not accounted for.
    ///
    /// # Panics
    ///
    /// With arithmetic overflow checks enabled, panics if `length` is zero or
    /// calculating the end index or column overflows.
    pub fn from_position_and_length(pos: &Position, length: usize) -> Self {
        let inc = length - 1;
        Self {
            start: *pos,
            end_inclusive: Position::new(pos.index + inc, pos.line, pos.column + inc),
        }
    }

    /// Creates a same-line range from a zero-based position and character count.
    ///
    /// `length` must be greater than zero. The end index and column are
    /// advanced by `length - 1`; line breaks are not accounted for.
    ///
    /// # Panics
    ///
    /// With arithmetic overflow checks enabled, panics if `length` is zero or
    /// calculating the end index or column overflows.
    pub fn from_detail(index: usize, line: usize, column: usize, length: usize) -> Self {
        let inc = length - 1;
        let start = Position::new(index, line, column);
        let end_inclusive = Position::new(index + inc, line, column + inc);
        Self {
            start,
            end_inclusive,
        }
    }

    /// Creates a range from `left.start` through `right.end_inclusive`.
    ///
    /// The inputs are not checked for ordering, adjacency, or overlap.
    pub fn merge(left: &Self, right: &Self) -> Self {
        Self {
            start: left.start,
            end_inclusive: right.end_inclusive,
        }
    }
}
