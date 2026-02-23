// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::io::{BufReader, Read};

pub struct UTF8CharIterator<'a, R>
where
    R: Read,
{
    bufreader: BufReader<&'a mut R>,
}

impl<'a, R> UTF8CharIterator<'a, R>
where
    R: Read,
{
    pub fn new(reader: &'a mut R) -> Self {
        Self {
            bufreader: BufReader::new(reader),
        }
    }
}

impl<R> UTF8CharIterator<'_, R>
where
    R: Read,
{
    /// Read a UTF-8 character from the stream.
    /// Ignores invalid UTF-8 sequences and I/O errors.
    #[inline]
    fn read_char(&mut self) -> Option<char> {
        // 1 byte:  0_bbb_aaaa
        // 2 bytes: 110_ccc_bb, 10_bb_aaaa
        // 3 bytes: 1110_dddd, 10_cccc_bb, 10_bb_aaaa
        // 4 bytes: 11110_f_ee, 10_ee_dddd, 10_cccc_bb, 10_bb_aaaa
        // ref:
        // https://en.wikipedia.org/wiki/UTF-8

        let first_byte = self.try_read_byte()?;
        match first_byte.leading_ones() {
            0 => {
                // 0_bbb_aaaa
                let char = unsafe { char::from_u32_unchecked(first_byte as u32) };
                Some(char)
            }
            2 => {
                // 110_ccc_bb, 10_bb_aaaa
                let second_byte = self.try_read_byte()?;
                let mut code: u32 = 0;
                code |= ((first_byte & 0b1_1111) as u32) << 6;
                code |= (second_byte & 0b11_1111) as u32;
                let char = unsafe { char::from_u32_unchecked(code) };
                Some(char)
            }
            3 => {
                // 1110_dddd, 10_cccc_bb, 10_bb_aaaa
                let two_bytes = self.consume_two_bytes()?;
                let mut code: u32 = 0;
                code |= ((first_byte & 0b1111) as u32) << 12;
                code |= ((two_bytes[0] & 0b11_1111) as u32) << 6;
                code |= (two_bytes[1] & 0b11_1111) as u32;
                let char = unsafe { char::from_u32_unchecked(code) };
                Some(char)
            }
            4 => {
                // 11110_f_ee, 10_ee_dddd, 10_cccc_bb, 10_bb_aaaa
                let three_bytes = self.consume_three_bytes()?;

                let mut code: u32 = 0;
                code |= ((first_byte & 0b111) as u32) << 18;
                code |= ((three_bytes[0] & 0b11_1111) as u32) << 12;
                code |= ((three_bytes[1] & 0b11_1111) as u32) << 6;
                code |= (three_bytes[2] & 0b11_1111) as u32;
                let char = unsafe { char::from_u32_unchecked(code) };
                Some(char)
            }
            _ => {
                // A byte between 0x80 and 0xBF is not a valid starting byte for UTF-8.
                //
                // P.S. 0x80 to 0xFF are not standard ASCII,
                // but often used in extended ASCII encodings like ISO-8859-1 (Latin-1), Windows-1252, etc.
                //
                // ```rust
                // panic!(
                //     "Invalid UTF-8 sequence, character code: 0x{:x}, at position: {}.",
                //     first_byte, position
                // );
                // ```

                // Ignore invalid UTF-8 sequences.
                Some(first_byte as char)
            }
        }
    }

    #[inline]
    fn try_read_byte(&mut self) -> Option<u8> {
        let mut buf = [0_u8; 1];
        let len = self.bufreader.read(&mut buf).unwrap_or(0);
        if len == 0 { None } else { Some(buf[0]) }
    }

    #[inline]
    fn consume_two_bytes(&mut self) -> Option<[u8; 2]> {
        let mut buf = [0_u8; 2];
        let len = self.bufreader.read(&mut buf).unwrap_or(0);
        if len < 2 {
            None
            // Err(std::io::Error::new(
            //     ErrorKind::InvalidData,
            //     "Incomplete UTF-8 character steam.",
            // ))
        } else {
            Some(buf)
        }
    }

    #[inline]
    fn consume_three_bytes(&mut self) -> Option<[u8; 3]> {
        let mut buf = [0_u8; 3];
        let len = self.bufreader.read(&mut buf).unwrap_or(0);

        if len < 3 {
            None
            // Err(std::io::Error::new(
            //     ErrorKind::InvalidData,
            //     "Incomplete UTF-8 character steam.",
            // ))
        } else {
            Some(buf)
        }
    }
}

impl<R> Iterator for UTF8CharIterator<'_, R>
where
    R: Read,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_char()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::utf8_char_iterator::UTF8CharIterator;

    #[test]
    fn test_utf8_char_iterator() {
        {
            let mut bytes = b"abc" as &[u8];
            let mut charstream = UTF8CharIterator::new(&mut bytes);

            assert_eq!(charstream.next(), Some('a'));
            assert_eq!(charstream.next(), Some('b'));
            assert_eq!(charstream.next(), Some('c'));
            assert_eq!(charstream.next(), None);
        }

        {
            let data = "a文b😋c".bytes().collect::<Vec<u8>>();
            let mut bytes = &data[..];
            let mut charstream = UTF8CharIterator::new(&mut bytes);

            assert_eq!(charstream.next(), Some('a'));
            assert_eq!(charstream.next(), Some('文'));
            assert_eq!(charstream.next(), Some('b'));
            assert_eq!(charstream.next(), Some('😋'));
            assert_eq!(charstream.next(), Some('c'));
            assert_eq!(charstream.next(), None);
        }
    }
}
