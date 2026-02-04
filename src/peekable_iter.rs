// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

pub const ROUND_QUEUE_LENGTH: usize = 8;

/// `PeekableIter` extends the functionality of `std::iter::Peekable` by allowing
/// peeking at elements at any specified offset, not just the next one.
pub struct PeekableIter<'a, T>
where
    T: PartialEq,
{
    upstream: &'a mut dyn Iterator<Item = T>,
    buffer: RoundQueue<T>,
    buffer_size: usize,
}

/// A fixed-size circular queue used for buffering elements in PeekableIter.
struct RoundQueue<T>
where
    T: PartialEq,
{
    size: usize,
    position_read: usize,
    position_write: usize,
    data: [Option<T>; ROUND_QUEUE_LENGTH],
}

impl<T> RoundQueue<T>
where
    T: PartialEq,
{
    /// Creates a new RoundQueue with the given size.
    /// Panics if the size is greater than or equal to MAX_LOOKAHEAD_LENGTH.
    pub fn new(size: usize) -> Self {
        assert!(size < ROUND_QUEUE_LENGTH);

        Self {
            size,
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
        if self.position_write == self.size {
            self.position_write = 0;
        }
    }

    /// Removes and returns the value at the current read position.
    /// Advances the read position, wrapping around if necessary.
    pub fn dequeue(&mut self) -> Option<T> {
        let value = self.data[self.position_read].take();

        self.position_read += 1;
        if self.position_read == self.size {
            self.position_read = 0;
        }

        value
    }

    /// Returns a reference to the value at the given offset from the current read position,
    /// or None if the slot is empty. The offset must be less than the queue size.
    pub fn peek(&self, offset: usize) -> Option<&T> {
        assert!(offset < self.size);

        let mut position = self.position_read + offset;
        if position >= self.size {
            position -= self.size;
        }

        self.data[position].as_ref()
    }
}

impl<'a, T> PeekableIter<'a, T>
where
    T: PartialEq,
{
    /// Creates a new PeekableIter with the specified buffer size.
    /// The buffer is pre-filled with elements from the upstream iterator.
    ///
    /// `buffer_size` The size of the buffer to use for peeking. For example,
    /// if `buffer_size` is 2, you can peek with offsets 0 and 1.
    pub fn new(upstream: &'a mut dyn Iterator<Item = T>, buffer_size: usize) -> Self {
        let mut buffer = RoundQueue::new(buffer_size);

        // Pre-fill the buffer with the first `buffer_size` elements from the upstream iterator.
        for _ in 0..buffer_size {
            let value = upstream.next();
            buffer.enqueue(value);
        }

        Self {
            upstream,
            buffer,
            buffer_size,
        }
    }

    /// Returns a reference to the element at the specified offset in the buffer,
    /// or None if that position is empty. The offset must be less than the buffer size.
    pub fn peek(&self, offset: usize) -> Option<&T> {
        assert!(offset < self.buffer_size);
        self.buffer.peek(offset)
    }
}

impl<T> Iterator for PeekableIter<'_, T>
where
    T: PartialEq,
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

#[cfg(test)]
mod tests {
    use crate::peekable_iter::PeekableIter;

    #[test]
    fn test_peekable_iter() {
        let s = "0123";
        let mut chars = s.chars();
        let mut iter = PeekableIter::new(&mut chars, 3);

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
        let mut chars = s.chars();
        let mut iter1 = PeekableIter::new(&mut chars, 3);
        let mut iter2 = PeekableIter::new(&mut iter1, 3);

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
