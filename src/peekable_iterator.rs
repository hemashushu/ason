// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

/// The fixed number of elements buffered for lookahead.
pub const ROUND_QUEUE_LENGTH: usize = 8;

/// An iterator with fixed-size lookahead beyond the next element.
///
/// Up to [`ROUND_QUEUE_LENGTH`] positions can be inspected without consuming
/// them. Constructing the iterator eagerly reads from the upstream iterator.
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
    /// Creates an iterator and pre-fills its fixed-size lookahead buffer.
    ///
    /// This calls `upstream.next()` [`ROUND_QUEUE_LENGTH`] times, so
    /// construction can block when the upstream source is live.
    pub fn new(mut upstream: U) -> Self {
        let mut buffer = RoundQueue::new();

        // Pre-fill the buffer with the first `ROUND_QUEUE_LENGTH` elements from the upstream iterator.
        for _ in 0..ROUND_QUEUE_LENGTH {
            let value = upstream.next();
            buffer.enqueue(value);
        }

        Self { upstream, buffer }
    }

    /// Borrows the buffered element at `offset`, or returns `None` for an empty slot.
    ///
    /// Offset zero refers to the next element without consuming it.
    ///
    /// # Panics
    ///
    /// Panics if `offset` is at least [`ROUND_QUEUE_LENGTH`].
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
    /// Creates an empty fixed-size circular queue.
    pub fn new() -> Self {
        Self {
            position_read: 0,
            position_write: 0,
            data: [const { None }; ROUND_QUEUE_LENGTH],
        }
    }

    /// Stores `value` at the write position and advances it with wraparound.
    ///
    /// An occupied slot is overwritten; this queue does not check capacity.
    pub fn enqueue(&mut self, value: Option<T>) {
        self.data[self.position_write] = value;

        self.position_write += 1;
        if self.position_write == ROUND_QUEUE_LENGTH {
            self.position_write = 0;
        }
    }

    /// Takes the current read slot and advances the read position with wraparound.
    ///
    /// Returns `None` if the slot is empty.
    pub fn dequeue(&mut self) -> Option<T> {
        let value = self.data[self.position_read].take();

        self.position_read += 1;
        if self.position_read == ROUND_QUEUE_LENGTH {
            self.position_read = 0;
        }

        value
    }

    /// Borrows the slot at `offset` from the read position, if it is occupied.
    ///
    /// # Panics
    ///
    /// Panics if `offset` is at least [`ROUND_QUEUE_LENGTH`].
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
