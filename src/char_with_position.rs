// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use crate::error::AsonError;
use crate::position::Position;

/// A Unicode scalar value and its zero-based position in the source text.
#[derive(Debug, PartialEq)]
pub struct CharWithPosition {
    /// The character from the source text.
    pub character: char,

    /// The position of the character in the source text.
    pub position: Position,
}

impl CharWithPosition {
    /// Associates `character` with its source `position`.
    pub fn new(character: char, position: Position) -> Self {
        Self {
            character,
            position,
        }
    }
}

/// Associates each upstream character with its zero-based source position.
///
/// Indices and columns count characters rather than bytes. A line feed
/// advances the line and resets the column; upstream errors are forwarded.
pub struct CharsWithPositionIterator<T>
where
    T: Iterator<Item = Result<char, AsonError>>,
{
    /// The underlying iterator that provides characters.
    upstream: T,

    /// Tracks the current position in the source text.
    current_position: Position,
}

impl<T> CharsWithPositionIterator<T>
where
    T: Iterator<Item = Result<char, AsonError>>,
{
    /// Wraps `upstream`, starting at character index, line, and column zero.
    pub fn new(upstream: T) -> Self {
        Self {
            upstream,
            current_position: Position::new(0, 0, 0),
        }
    }
}

impl<T> Iterator for CharsWithPositionIterator<T>
where
    T: Iterator<Item = Result<char, AsonError>>,
{
    type Item = Result<CharWithPosition, AsonError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.upstream.next() {
            Some(Ok(c)) => {
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

                Some(Ok(CharWithPosition::new(c, last_position)))
            }
            Some(Err(error)) => Some(Err(error)),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        char_with_position::{CharWithPosition, CharsWithPositionIterator},
        error::AsonError,
        position::Position,
    };

    #[test]
    fn test_chars_with_position_iter() {
        {
            let chars = "a\nmn\nxyz".chars().map(Ok::<char, AsonError>);
            let mut char_position_iter = CharsWithPositionIterator::new(chars);

            assert_eq!(
                char_position_iter.next(),
                Some(Ok(CharWithPosition::new('a', Position::new(0, 0, 0))))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(Ok(CharWithPosition::new('\n', Position::new(1, 0, 1))))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(Ok(CharWithPosition::new('m', Position::new(2, 1, 0))))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(Ok(CharWithPosition::new('n', Position::new(3, 1, 1))))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(Ok(CharWithPosition::new('\n', Position::new(4, 1, 2))))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(Ok(CharWithPosition::new('x', Position::new(5, 2, 0))))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(Ok(CharWithPosition::new('y', Position::new(6, 2, 1))))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(Ok(CharWithPosition::new('z', Position::new(7, 2, 2))))
            );

            assert!(char_position_iter.next().is_none());
        }

        {
            let chars = "\n\r\n\n".chars().map(Ok::<char, AsonError>);
            let mut char_position_iter = CharsWithPositionIterator::new(chars);

            assert_eq!(
                char_position_iter.next(),
                Some(Ok(CharWithPosition::new('\n', Position::new(0, 0, 0))))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(Ok(CharWithPosition::new('\r', Position::new(1, 1, 0))))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(Ok(CharWithPosition::new('\n', Position::new(2, 1, 1))))
            );

            assert_eq!(
                char_position_iter.next(),
                Some(Ok(CharWithPosition::new('\n', Position::new(3, 2, 0))))
            );

            assert!(char_position_iter.next().is_none());
        }
    }
}
