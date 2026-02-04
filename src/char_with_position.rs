// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use crate::position::Position;

#[derive(Debug, PartialEq)]
/// Represents a character along with its position in the source text.
pub struct CharWithPosition {
    /// The character from the source text.
    pub character: char,

    /// The position of the character in the source text.
    pub position: Position,
}

impl CharWithPosition {
    pub fn new(character: char, position: Position) -> Self {
        Self {
            character,
            position,
        }
    }
}

/// An iterator that yields each character from the upstream iterator along with its position in the source text.
pub struct CharsWithPositionIter<'a> {
    /// The underlying iterator that provides characters.
    upstream: &'a mut dyn Iterator<Item = char>,

    /// Tracks the current position in the source text.
    current_position: Position,
}

impl<'a> CharsWithPositionIter<'a> {
    pub fn new(upstream: &'a mut dyn Iterator<Item = char>) -> Self {
        Self {
            upstream,
            current_position: Position::new(0, 0, 0),
        }
    }
}

impl Iterator for CharsWithPositionIter<'_> {
    type Item = CharWithPosition;

    fn next(&mut self) -> Option<Self::Item> {
        match self.upstream.next() {
            Some(c) => {
                // Save the current position to associate with the character being returned.
                let last_position = self.current_position;

                // Advance the absolute character index.
                self.current_position.index += 1;

                // If the character is a newline, move to the next line and reset the column.
                if c == '\n' {
                    self.current_position.line += 1;
                    self.current_position.column = 0;
                } else {
                    // Otherwise, move to the next column in the current line.
                    self.current_position.column += 1;
                }

                Some(CharWithPosition::new(c, last_position))
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        char_with_position::{CharWithPosition, CharsWithPositionIter},
        position::Position,
    };

    #[test]
    fn test_chars_with_position_iter() {
        {
            let mut chars = "a\nmn\nxyz".chars();
            let mut char_position_iter = CharsWithPositionIter::new(&mut chars);

            assert_eq!(
                char_position_iter.next(),
                Some(CharWithPosition::new('a', Position::new(0, 0, 0)))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(CharWithPosition::new('\n', Position::new(1, 0, 1)))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(CharWithPosition::new('m', Position::new(2, 1, 0)))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(CharWithPosition::new('n', Position::new(3, 1, 1)))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(CharWithPosition::new('\n', Position::new(4, 1, 2)))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(CharWithPosition::new('x', Position::new(5, 2, 0)))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(CharWithPosition::new('y', Position::new(6, 2, 1)))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(CharWithPosition::new('z', Position::new(7, 2, 2)))
            );

            assert!(char_position_iter.next().is_none());
        }

        {
            let mut chars = "\n\r\n\n".chars();
            let mut char_position_iter = CharsWithPositionIter::new(&mut chars);

            assert_eq!(
                char_position_iter.next(),
                Some(CharWithPosition::new('\n', Position::new(0, 0, 0)))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(CharWithPosition::new('\r', Position::new(1, 1, 0)))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(CharWithPosition::new('\n', Position::new(2, 1, 1)))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(CharWithPosition::new('\n', Position::new(3, 2, 0)))
            );

            assert!(char_position_iter.next().is_none());
        }
    }
}
