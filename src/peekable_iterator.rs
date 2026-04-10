// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

pub const ROUND_QUEUE_LENGTH: usize = 8;

/// `PeekableIterator` extends `std::iter::Peekable` by allowing
/// peeking at elements at any specified offset, not just the next one.
pub struct PeekableIterator<T, U>
where
    T: PartialEq,
    U: Iterator<Item = T>,
{
    upstream: U,
    buffer: RoundQueue<T>,
}

impl<T, U> PeekableIterator<T, U>
where
    T: PartialEq,
    U: Iterator<Item = T>,
{
    /// Creates a new `PeekableIterator` with the specified buffer size.
    /// The buffer is pre-filled with elements from the upstream iterator.
    pub fn new(mut upstream: U) -> Self {
        let mut buffer = RoundQueue::new();

        // Pre-fill the buffer with the first `ROUND_QUEUE_LENGTH` elements from the upstream iterator.
        for _ in 0..ROUND_QUEUE_LENGTH {
            let value = upstream.next();
            buffer.enqueue(value);
        }

        Self { upstream, buffer }
    }

    /// Returns a reference to the element at the specified offset in the buffer,
    /// or None if that position is empty. The offset must be less than the buffer size.
    pub fn peek(&self, offset: usize) -> Option<&T> {
        assert!(offset < ROUND_QUEUE_LENGTH);
        self.buffer.peek(offset)
    }
}

impl<T, U> Iterator for PeekableIterator<T, U>
where
    T: PartialEq,
    U: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // Remove and return the next value from the buffer.
        // The buffer is pre-filled during initialization, so we always dequeue first,
        // then fetch the next value from the upstream iterator and enqueue it.
        let value = self.buffer.dequeue();

        let next_value = self.upstream.next();
        self.buffer.enqueue(next_value);

        value
    }
}

/// A fixed-size circular queue used for buffering elements in `PeekableIterator`.
struct RoundQueue<T>
where
    T: PartialEq,
{
    position_read: usize,
    position_write: usize,
    data: [Option<T>; ROUND_QUEUE_LENGTH],
}

impl<T> RoundQueue<T>
where
    T: PartialEq,
{
    /// Creates a new RoundQueue.
    pub fn new() -> Self {
        Self {
            position_read: 0,
            position_write: 0,
            data: [const { None }; ROUND_QUEUE_LENGTH],
        }
    }

    /// Adds a value to the queue at the current write position.
    /// Advances the write position, wrapping around if necessary.
    pub fn enqueue(&mut self, value: Option<T>) {
        self.data[self.position_write] = value;

        self.position_write += 1;
        if self.position_write == ROUND_QUEUE_LENGTH {
            self.position_write = 0;
        }
    }

    /// Removes and returns the value at the current read position.
    /// Advances the read position, wrapping around if necessary.
    pub fn dequeue(&mut self) -> Option<T> {
        let value = self.data[self.position_read].take();

        self.position_read += 1;
        if self.position_read == ROUND_QUEUE_LENGTH {
            self.position_read = 0;
        }

        value
    }

    /// Returns a reference to the value at the given offset from the current read position,
    /// or None if the slot is empty. The offset must be less than the queue size.
    pub fn peek(&self, offset: usize) -> Option<&T> {
        assert!(offset < ROUND_QUEUE_LENGTH);

        let mut position = self.position_read + offset;
        if position >= ROUND_QUEUE_LENGTH {
            position -= ROUND_QUEUE_LENGTH;
        }

        self.data[position].as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::peekable_iterator::PeekableIterator;

    #[test]
    fn test_peekable_iter() {
        let s = "0123";
        let chars = s.chars();
        let mut iter = PeekableIterator::new(chars);

        // Initial state: buffer contains '0', '1', '2'
        assert_eq!(Some(&'0'), iter.peek(0));
        assert_eq!(Some(&'1'), iter.peek(1));
        assert_eq!(Some(&'2'), iter.peek(2));

        // Consume '0'
        assert_eq!(Some('0'), iter.next());
        assert_eq!(Some(&'1'), iter.peek(0));
        assert_eq!(Some(&'2'), iter.peek(1));
        assert_eq!(Some(&'3'), iter.peek(2));

        // Consume '1'
        assert_eq!(Some('1'), iter.next());
        assert_eq!(Some(&'2'), iter.peek(0));
        assert_eq!(Some(&'3'), iter.peek(1));
        assert_eq!(None, iter.peek(2));

        // Consume '2'
        assert_eq!(Some('2'), iter.next());
        assert_eq!(Some(&'3'), iter.peek(0));
        assert_eq!(None, iter.peek(1));
        assert_eq!(None, iter.peek(2));

        // Consume '3'
        assert_eq!(Some('3'), iter.next());
        assert_eq!(None, iter.peek(0));
        assert_eq!(None, iter.peek(1));
        assert_eq!(None, iter.peek(2));

        // Iterator is now empty
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.peek(0));
        assert_eq!(None, iter.peek(1));
        assert_eq!(None, iter.peek(2));
    }

    #[test]
    fn test_nested_peekable_iter() {
        let s = "0123";
        let chars = s.chars();
        let iter1 = PeekableIterator::new(chars);
        let mut iter2 = PeekableIterator::new(iter1);

        // Initial state: buffer contains '0', '1', '2'
        assert_eq!(Some(&'0'), iter2.peek(0));
        assert_eq!(Some(&'1'), iter2.peek(1));
        assert_eq!(Some(&'2'), iter2.peek(2));

        // Consume '0'
        assert_eq!(Some('0'), iter2.next());
        assert_eq!(Some(&'1'), iter2.peek(0));

        // Consume '1'
        assert_eq!(Some('1'), iter2.next());
        assert_eq!(Some(&'2'), iter2.peek(0));

        // Consume '2'
        assert_eq!(Some('2'), iter2.next());
        assert_eq!(Some(&'3'), iter2.peek(0));

        // Consume '3'
        assert_eq!(Some('3'), iter2.next());
        assert_eq!(None, iter2.peek(0));

        // Iterator is now empty
        assert_eq!(None, iter2.next());
        assert_eq!(None, iter2.peek(0));
    }
}
