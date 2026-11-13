// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::io::{self, BufReader, Read};

use crate::error::AsonError;

/// Decodes a buffered byte reader into UTF-8 characters.
///
/// Short reads are accepted and interrupted reads are retried. Invalid UTF-8,
/// incomplete sequences, and I/O failures are yielded as an error once,
/// after which the iterator is exhausted.
pub(super) struct UTF8CharIterator<T>
where
    T: Read,
{
    reader: BufReader<T>,
    finished: bool,
}

impl<T> UTF8CharIterator<T>
where
    T: Read,
{
    /// Wraps `reader` in a [`BufReader`] and begins decoding at its current position.
    ///
    /// An already buffered reader is accepted but does not require special
    /// handling; this constructor still adds its own buffer.
    pub(super) fn new(reader: T) -> Self {
        Self {
            reader: BufReader::new(reader),
            finished: false,
        }
    }
}

impl<T> UTF8CharIterator<T>
where
    T: Read,
{
    /// Read and validate one UTF-8 character, tolerating short reads.
    #[inline]
    fn read_char(&mut self) -> Result<Option<char>, AsonError> {
        let Some(first_byte) = self.read_byte()? else {
            return Ok(None);
        };

        let width = match first_byte {
            0x00..=0x7f => 1,
            0xc2..=0xdf => 2,
            0xe0..=0xef => 3,
            0xf0..=0xf4 => 4,
            _ => return Err(AsonError::Message("Invalid UTF-8 leading byte.".into())),
        };

        let mut bytes = [0; 4];
        bytes[0] = first_byte;
        for byte in bytes.iter_mut().take(width).skip(1) {
            *byte = self
                .read_byte()?
                .ok_or_else(|| AsonError::Message("Incomplete UTF-8 sequence at EOF.".into()))?;
        }

        let text = std::str::from_utf8(&bytes[..width])
            .map_err(|error| AsonError::Message(format!("Invalid UTF-8: {error}")))?;
        Ok(text.chars().next())
    }

    fn read_byte(&mut self) -> Result<Option<u8>, AsonError> {
        let mut byte = [0];
        loop {
            match self.reader.read(&mut byte) {
                Ok(0) => return Ok(None),
                Ok(_) => return Ok(Some(byte[0])),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(AsonError::Message(format!("I/O error: {error}"))),
            }
        }
    }
}

impl<T> Iterator for UTF8CharIterator<T>
where
    T: Read,
{
    type Item = Result<char, AsonError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        match self.read_char() {
            Ok(Some(character)) => Some(Ok(character)),
            Ok(None) => {
                self.finished = true;
                None
            }
            Err(error) => {
                self.finished = true;
                Some(Err(error))
            }
        }
    }
}

impl<T: Read> std::iter::FusedIterator for UTF8CharIterator<T> {}

#[cfg(test)]
mod tests {
    use std::io::{self, Cursor, Read};

    use pretty_assertions::assert_eq;

    use crate::error::AsonError;
    use crate::utf8_char_iterator::UTF8CharIterator;

    struct FailingReader {
        bytes: &'static [u8],
    }

    impl Read for FailingReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if let Some((&byte, remaining)) = self.bytes.split_first() {
                buffer[0] = byte;
                self.bytes = remaining;
                Ok(1)
            } else {
                Err(io::Error::other("test read failure"))
            }
        }
    }

    struct ShortInterruptedReader {
        bytes: &'static [u8],
        interrupted: bool,
    }

    impl Read for ShortInterruptedReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::ErrorKind::Interrupted.into());
            }
            if let Some((&byte, remaining)) = self.bytes.split_first() {
                buffer[0] = byte;
                self.bytes = remaining;
                Ok(1)
            } else {
                Ok(0)
            }
        }
    }

    #[test]
    fn test_utf8_char_iterator() {
        {
            let bytes = b"abc" as &[u8];
            let mut charstream = UTF8CharIterator::new(bytes);

            assert_eq!(charstream.next(), Some(Ok('a')));
            assert_eq!(charstream.next(), Some(Ok('b')));
            assert_eq!(charstream.next(), Some(Ok('c')));
            assert_eq!(charstream.next(), None);
        }

        {
            let data = "a文b😋c".bytes().collect::<Vec<u8>>();
            let bytes = Cursor::new(data);
            let mut charstream = UTF8CharIterator::new(bytes);

            assert_eq!(charstream.next(), Some(Ok('a')));
            assert_eq!(charstream.next(), Some(Ok('文')));
            assert_eq!(charstream.next(), Some(Ok('b')));
            assert_eq!(charstream.next(), Some(Ok('😋')));
            assert_eq!(charstream.next(), Some(Ok('c')));
            assert_eq!(charstream.next(), None);
        }
    }

    #[test]
    fn test_invalid_and_incomplete_utf8() {
        for bytes in [&[0xff][..], &[0xc0, 0x80][..], &[0xe2, 0x82][..]] {
            let mut chars = UTF8CharIterator::new(bytes);
            assert!(matches!(chars.next(), Some(Err(AsonError::Message(_)))));
            assert_eq!(chars.next(), None);
        }
    }

    #[test]
    fn test_io_error_is_not_eof() {
        let mut chars = UTF8CharIterator::new(FailingReader { bytes: b"a" });
        assert_eq!(chars.next(), Some(Ok('a')));
        assert!(matches!(
            chars.next(),
            Some(Err(AsonError::Message(message))) if message.contains("test read failure")
        ));
        assert_eq!(chars.next(), None);
    }

    #[test]
    fn test_short_reads_and_interrupted_reads() {
        let text = "文😋";
        let reader = ShortInterruptedReader {
            bytes: text.as_bytes(),
            interrupted: false,
        };
        let decoded = UTF8CharIterator::new(reader)
            .collect::<Result<String, _>>()
            .unwrap();
        assert_eq!(decoded, text);
    }
}
