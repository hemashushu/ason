// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::fmt::Display;

use chrono::DateTime;

use crate::{
    char_with_position::CharWithPosition,
    error::AsonError,
    peekable_iter::PeekableIter,
    position::Position,
    range::Range,
    token::{NumberToken, Token, TokenWithRange},
};

pub const PEEK_BUFFER_LENGTH_LEX: usize = 3;

pub struct Lexer<'a> {
    upstream: &'a mut PeekableIter<'a, CharWithPosition>,

    // The position of the last consumed character by `next_char()`.
    last_position: Position,

    // Stack of positions.
    // It is used to store the positions of characters when consuming them in sequence,
    // and later used to create the `Range` of tokens.
    position_stack: Vec<Position>,
}

impl<'a> Lexer<'a> {
    pub fn new(upstream: &'a mut PeekableIter<'a, CharWithPosition>) -> Self {
        Self {
            upstream,
            last_position: Position::default(),
            position_stack: vec![],
        }
    }

    fn next_char(&mut self) -> Option<char> {
        match self.upstream.next() {
            Some(CharWithPosition {
                character,
                position,
            }) => {
                self.last_position = position;
                Some(character)
            }
            None => None,
        }
    }

    fn peek_char(&self, offset: usize) -> Option<&char> {
        match self.upstream.peek(offset) {
            Some(CharWithPosition { character, .. }) => Some(character),
            None => None,
        }
    }

    fn peek_position(&self, offset: usize) -> Option<&Position> {
        match self.upstream.peek(offset) {
            Some(CharWithPosition { position, .. }) => Some(position),
            None => None,
        }
    }

    fn peek_char_and_equals(&self, offset: usize, expected_char: char) -> bool {
        matches!(
            self.upstream.peek(offset),
            Some(CharWithPosition { character, .. }) if character == &expected_char)
    }

    /// Saves the last position to the stack.
    ///
    /// Where the last position is identical to position of
    /// the character consumed by `next_char()`.
    fn push_last_position_into_stack(&mut self) {
        self.position_stack.push(self.last_position);
    }

    /// Saves the current position to the stack.
    ///
    /// Where the current position is identical to the value returned by
    /// `self.peek_position(0)`.
    fn push_peek_position_into_stack(&mut self) {
        let position = *self.peek_position(0).unwrap();
        self.position_stack.push(position);
    }

    /// Pops a position from the stack.
    ///
    /// It is usually used after `push_last_position_into_stack()` or
    /// `push_peek_position_into_stack()` to form a `Range` for a token.
    fn pop_position_from_stack(&mut self) -> Position {
        self.position_stack.pop().unwrap()
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<TokenWithRange, AsonError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lex()
    }
}

impl Lexer<'_> {
    fn lex(&mut self) -> Option<Result<TokenWithRange, AsonError>> {
        // ```diagram
        // c....
        // ^____ current char, not EOF, validated
        // ```

        let result = loop {
            let current_char = match self.peek_char(0) {
                Some(c) => *c,
                None => return None, // EOF
            };

            match current_char {
                '/' if self.peek_char_and_equals(1, '/') => {
                    // line comment
                    self.lex_line_comment()
                }
                '/' if self.peek_char_and_equals(1, '*') => {
                    // block comment
                    if let Err(e) = self.lex_block_comment() {
                        break Err(e);
                    }
                }
                ',' => {
                    self.next_char(); // Consume ','
                }
                ' ' | '\t' => {
                    self.next_char(); // Consume space or tab
                }
                '\r' if self.peek_char_and_equals(1, '\n') => {
                    // Windows style new line `\r\n`
                    self.next_char(); // Consume '\r'
                    self.next_char(); // Consume '\n'
                }
                '\n' => {
                    self.next_char(); // Consume '\n'
                }
                ':' => {
                    self.next_char(); // Consume ':'

                    // Don't confuse with variant separator "::"
                    break Ok(TokenWithRange::new(
                        Token::Colon,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                '{' => {
                    self.next_char(); // Consume '{'

                    break Ok(TokenWithRange::new(
                        Token::OpeningBrace,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                '}' => {
                    self.next_char(); // Consume '}'

                    break Ok(TokenWithRange::new(
                        Token::ClosingBrace,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                '[' => {
                    self.next_char(); // Consume '['

                    break Ok(TokenWithRange::new(
                        Token::OpeningBracket,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                ']' => {
                    self.next_char(); // Consume ']'

                    break Ok(TokenWithRange::new(
                        Token::ClosingBracket,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                '(' => {
                    self.next_char(); // Consume '('

                    break Ok(TokenWithRange::new(
                        Token::OpeningParenthesis,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                ')' => {
                    self.next_char(); // Consume ')'

                    break Ok(TokenWithRange::new(
                        Token::ClosingParenthesis,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                '+' => {
                    self.next_char(); // Consume '+'

                    break Ok(TokenWithRange::new(
                        Token::_Plus,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                '-' => {
                    self.next_char(); // Consume '-'

                    break Ok(TokenWithRange::new(
                        Token::_Minus,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                '0' if matches!(self.peek_char(1), Some('x' | 'X')) => {
                    // hexadecimal number
                    break self.lex_hexadecimal_number();
                }
                '0' if matches!(self.peek_char(1), Some('b' | 'B')) => {
                    // binary number
                    break self.lex_binary_number();
                }
                '0' if matches!(self.peek_char(1), Some('o' | 'O')) => {
                    // octal number
                    break self.lex_octal_number();
                }
                '0' if matches!(self.peek_char(1), Some('.')) => {
                    // decimal number
                    break self.lex_decimal_number();
                }
                '0'..='9' => {
                    // decimal number
                    break self.lex_decimal_number();
                }
                'h' if self.peek_char_and_equals(1, '"') => {
                    // hex byte data element
                    break self.lex_hexadecimal_byte_data();
                }
                'd' if self.peek_char_and_equals(1, '"') => {
                    // date
                    break self.lex_datetime();
                }
                'r' if self.peek_char_and_equals(1, '"') => {
                    // raw string
                    break self.lex_raw_string();
                }
                'r' if self.peek_char_and_equals(1, '#') && self.peek_char_and_equals(2, '"') => {
                    // raw string with hash symbol
                    break self.lex_raw_string_with_hash_symbol();
                }
                '"' => {
                    // string
                    if self.peek_char_and_equals(1, '"') && self.peek_char_and_equals(2, '"') {
                        // auto-trimmed string
                        break self.lex_auto_trimmed_string();
                    } else {
                        // normal string
                        break self.lex_string();
                    }
                }
                '\'' => {
                    // char
                    break self.lex_char();
                }
                'a'..='z' | 'A'..='Z' | '_' | '\u{a0}'..='\u{d7ff}' | '\u{e000}'..='\u{10ffff}' => {
                    // identifier
                    break self.lex_identifier();
                }
                current_char => {
                    break Err(AsonError::MessageWithPosition(
                        format!("Unexpected char '{}'.", current_char),
                        *self.peek_position(0).unwrap(),
                    ));
                }
            }
        };

        Some(result)
    }

    fn lex_identifier(&mut self) -> Result<TokenWithRange, AsonError> {
        // ```diagram
        // key_nameT  //
        // ^       ^__// to here
        // |__________// current char, validated
        //
        // T = terminator chars || EOF
        // ```

        let mut identifier_buffer = String::new();

        // A flag to indicate whether '::' is found.
        // '::' is used to separate type name and member name in variant token.
        let mut found_double_colon = false;

        self.push_peek_position_into_stack();

        while let Some(current_char) = self.peek_char(0) {
            match current_char {
                '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' => {
                    identifier_buffer.push(*current_char);
                    self.next_char(); // consume char
                }
                ':' if self.peek_char_and_equals(1, ':') => {
                    found_double_colon = true;
                    identifier_buffer.push_str("::");
                    self.next_char(); // consume 1st ":"
                    self.next_char(); // consume 2nd ":"
                }
                '\u{a0}'..='\u{d7ff}' | '\u{e000}'..='\u{10ffff}' => {
                    // A char is a ‘Unicode scalar value’, which is any ‘Unicode code point’ other than a surrogate code point.
                    // This has a fixed numerical definition: code points are in the range 0 to 0x10FFFF,
                    // inclusive. Surrogate code points, used by UTF-16, are in the range 0xD800 to 0xDFFF.
                    //
                    // check out:
                    // https://doc.rust-lang.org/std/primitive.char.html
                    //
                    // CJK chars: '\u{4e00}'..='\u{9fff}'
                    // for complete CJK chars, check out Unicode standard
                    // Ch. 18.1 Han CJK Unified Ideographs
                    //
                    // summary:
                    // Block Position Comment
                    // CJK Unified Ideographs 4E00–9FFF Common
                    // CJK Unified Ideographs Extension A 3400–4DBF Rare
                    // CJK Unified Ideographs Extension B 20000–2A6DF Rare, historic
                    // CJK Unified Ideographs Extension C 2A700–2B73F Rare, historic
                    // CJK Unified Ideographs Extension D 2B740–2B81F Uncommon, some in current use
                    // CJK Unified Ideographs Extension E 2B820–2CEAF Rare, historic
                    // CJK Unified Ideographs Extension F 2CEB0–2EBEF Rare, historic
                    // CJK Unified Ideographs Extension G 30000–3134F Rare, historic
                    // CJK Unified Ideographs Extension H 31350–323AF Rare, historic
                    // CJK Compatibility Ideographs F900–FAFF Duplicates, unifiable variants, corporate characters
                    // CJK Compatibility Ideographs Supplement 2F800–2FA1F Unifiable variants
                    //
                    // https://www.unicode.org/versions/Unicode15.0.0/ch18.pdf
                    // https://en.wikipedia.org/wiki/CJK_Unified_Ideographs
                    // https://www.unicode.org/versions/Unicode15.0.0/
                    //
                    // see also
                    // https://www.unicode.org/reports/tr31/tr31-37.html

                    identifier_buffer.push(*current_char);
                    self.next_char(); // consume char
                }
                ' ' | '\t' | '\r' | '\n' | ',' | ':' | '{' | '}' | '[' | ']' | '(' | ')' | '/' => {
                    // | '\'' | '"' => {
                    // terminator chars
                    break;
                }
                _ => {
                    return Err(AsonError::MessageWithPosition(
                        format!("Invalid char '{}' for identifier.", current_char),
                        *self.peek_position(0).unwrap(),
                    ));
                }
            }
        }

        let identifier_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        let token = if found_double_colon {
            let (type_name, member_name) = identifier_buffer.split_once("::").unwrap();
            Token::Variant(type_name.to_owned(), member_name.to_owned())
        } else {
            match identifier_buffer.as_str() {
                "true" => Token::Boolean(true),
                "false" => Token::Boolean(false),
                "NaN" | "NaN_f64" => Token::Number(NumberToken::F64(f64::NAN)), // the default floating-point type is f64
                "NaN_f32" => Token::Number(NumberToken::F32(f32::NAN)),
                "Inf" | "Inf_f64" => Token::Number(NumberToken::F64(f64::INFINITY)), // the default floating-point type is f64
                "Inf_f32" => Token::Number(NumberToken::F32(f32::INFINITY)),
                _ => Token::Identifier(identifier_buffer),
            }
        };

        Ok(TokenWithRange::new(token, identifier_range))
    }

    fn lex_decimal_number(&mut self) -> Result<TokenWithRange, AsonError> {
        // ```diagram
        // 123456T  //
        // ^     ^__// to here
        // |________// current char, validated
        //
        // T = terminator chars || EOF
        // ```

        let mut number_buffer = String::new();

        // Number type suffix, e.g.,
        // "123_i16", "456_u32", "3.14_f64"
        let mut number_type_opt: Option<NumberType> = None;

        // A flag to indicate whether '.' is found.
        // The presence of '.' indicates a floating-point number.
        let mut found_point = false;

        // A flag to indicate whether 'e' is found.
        // The presence of 'e' indicates a floating-point number.
        let mut found_e = false;

        // Decimal number samples:
        //
        // 123
        // 3.14
        // 2.99e8
        // 2.99e+8
        // 6.672e-34

        self.push_peek_position_into_stack();

        while let Some(current_char) = self.peek_char(0) {
            match current_char {
                '0'..='9' => {
                    // valid digits for decimal number
                    number_buffer.push(*current_char);
                    self.next_char(); // consume digit
                }
                '_' => {
                    self.next_char(); // consume '_'
                }
                '.' if !found_point => {
                    found_point = true;

                    // 3.14
                    // 271.828
                    number_buffer.push(*current_char);
                    self.next_char(); // consume '.'
                }
                'e' | 'E' if !found_e => {
                    found_e = true;

                    // 123e45
                    // 123e+45
                    // 123e-45
                    if self.peek_char_and_equals(1, '-') {
                        number_buffer.push_str("e-");
                        self.next_char(); // consume 'e'
                        self.next_char(); // consume '-'
                    } else if self.peek_char_and_equals(1, '+') {
                        number_buffer.push_str("e+");
                        self.next_char(); // consume 'e'
                        self.next_char(); // consume '+'
                    } else {
                        number_buffer.push(*current_char);
                        self.next_char(); // consume 'e'
                    }
                }
                'i' | 'u' | 'f'
                    if number_type_opt.is_none()
                        && matches!(self.peek_char(1), Some('0'..='9')) =>
                {
                    let number_type = self.lex_number_type_suffix()?;
                    number_type_opt.replace(number_type);
                    break;
                }
                ' ' | '\t' | '\r' | '\n' | ',' | ':' | '{' | '}' | '[' | ']' | '(' | ')' | '/' => {
                    // | '\'' | '"' => {
                    // terminator chars
                    break;
                }
                _ => {
                    return Err(AsonError::MessageWithPosition(
                        format!("Invalid char '{}' for decimal number.", current_char),
                        *self.peek_position(0).unwrap(),
                    ));
                }
            }
        }

        // check syntax

        // Tailing '.' is not allowed
        if number_buffer.ends_with('.') {
            return Err(AsonError::MessageWithRange(
                "Decimal number can not ends with \".\".".to_owned(),
                Range::new(&self.pop_position_from_stack(), &self.last_position),
            ));
        }

        // Tailing 'e' is not allowed
        if number_buffer.ends_with('e') {
            return Err(AsonError::MessageWithRange(
                "Decimal number can not ends with \"e\".".to_owned(),
                Range::new(&self.pop_position_from_stack(), &self.last_position),
            ));
        }

        // Leading zeros are not allowed for decimal integers
        if !found_point && !found_e && number_buffer.len() > 1 && number_buffer.starts_with('0') {
            return Err(AsonError::MessageWithRange(
                "Leading zeros are not allowed for decimal integers.".to_owned(),
                Range::new(&self.pop_position_from_stack(), &self.last_position),
            ));
        }

        let number_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        let number_token: NumberToken = if let Some(number_type) = number_type_opt {
            // Numbers with explicit type.
            match number_type {
                NumberType::F32 => {
                    let v = number_buffer.parse::<f32>().map_err(|_| {
                        AsonError::MessageWithRange(
                            format!(
                                "Can not convert \"{}\" to f32 floating-point number.",
                                number_buffer
                            ),
                            number_range,
                        )
                    })?;

                    // overflow when parsing from string
                    if v.is_infinite() {
                        return Err(AsonError::MessageWithRange(
                            format!(
                                "F32 floating point number \"{}\" is overflow.",
                                number_buffer
                            ),
                            number_range,
                        ));
                    }

                    NumberToken::F32(v)
                }
                NumberType::F64 => {
                    let v = number_buffer.parse::<f64>().map_err(|_| {
                        AsonError::MessageWithRange(
                            format!(
                                "Can not convert \"{}\" to f64 floating-point number.",
                                number_buffer
                            ),
                            number_range,
                        )
                    })?;

                    // overflow when parsing from string
                    if v.is_infinite() {
                        return Err(AsonError::MessageWithRange(
                            format!(
                                "F64 floating point number \"{}\" is overflow.",
                                number_buffer
                            ),
                            number_range,
                        ));
                    }

                    NumberToken::F64(v)
                }
                _ => convert_integer_number_string_with_data_type(
                    &number_buffer,
                    10,
                    number_type,
                    number_range,
                )?,
            }
        } else if found_point || found_e {
            // Numbers without explicit type, and it is a floating-point number.
            // The default floating-point number type is `f64`.

            let v = number_buffer.parse::<f64>().map_err(|_| {
                AsonError::MessageWithRange(
                    format!(
                        "Can not convert \"{}\" to f64 floating-point number.",
                        number_buffer
                    ),
                    number_range,
                )
            })?;

            // overflow when parsing from string
            if v.is_infinite() {
                return Err(AsonError::MessageWithRange(
                    format!(
                        "F64 floating point number \"{}\" is overflow.",
                        number_buffer
                    ),
                    number_range,
                ));
            }

            NumberToken::F64(v)
        } else {
            // Numbers without explicit type, and it is an integer number.
            // The default integer number type is `i32`.

            let v = number_buffer.parse::<u32>().map_err(|_| {
                AsonError::MessageWithRange(
                    format!(
                        "Can not convert \"{}\" to i32 integer number.",
                        number_buffer,
                    ),
                    number_range,
                )
            })?;

            NumberToken::I32(v)
        };

        Ok(TokenWithRange::new(
            Token::Number(number_token),
            number_range,
        ))
    }

    fn lex_hexadecimal_number(&mut self) -> Result<TokenWithRange, AsonError> {
        // ```diagram
        // 0xaabbT  //
        // ^^    ^__// to here
        // ||_______// validated
        // |________// current char, validated
        //
        // T = terminator chars || EOF
        // ```

        self.push_peek_position_into_stack();

        self.next_char(); // consume '0'
        self.next_char(); // consume 'x'

        let mut number_buffer = String::new();

        // Number type suffix, e.g.,
        // "123_i16", "456_u32", "3.14_f64"
        let mut number_type_opt: Option<NumberType> = None;

        // A flag to indicate whether '.' is found.
        // The presence of '.' indicates a floating-point number.
        let mut found_point = false;

        // A flag to indicate whether 'p' is found.
        // The presence of 'p' indicates a hexadecimal floating-point number.
        let mut found_p: bool = false;

        while let Some(current_char) = self.peek_char(0) {
            match current_char {
                'f' if number_type_opt.is_none()
                    && found_p
                    && matches!(self.peek_char(1), Some('0'..='9')) =>
                {
                    // 'f' is allowed only in the hexadecimal floating point literal, that is,
                    // the character 'p' should be detected before 'f'.
                    let number_type = self.lex_number_type_suffix()?;
                    number_type_opt.replace(number_type);
                    break;
                }
                '0'..='9' | 'a'..='f' | 'A'..='F' => {
                    // valid digits for hex number
                    number_buffer.push(*current_char);
                    self.next_char(); // consume digit
                }
                '_' => {
                    self.next_char(); // consume '_'
                }
                '.' if !found_point && !found_p => {
                    found_point = true;

                    // 0x1.9
                    // 0x12.bc
                    number_buffer.push(*current_char);
                    self.next_char(); // consume '.'
                }
                'p' | 'P' if !found_p => {
                    found_p = true;

                    // 0x0.123p45
                    // 0x0.123p+45
                    // 0x0.123p-45
                    if self.peek_char_and_equals(1, '-') {
                        number_buffer.push_str("p-");
                        self.next_char(); // consume 'p'
                        self.next_char(); // consume '-'
                    } else if self.peek_char_and_equals(1, '+') {
                        number_buffer.push_str("p+");
                        self.next_char(); // consume 'p'
                        self.next_char(); // consume '+'
                    } else {
                        number_buffer.push(*current_char);
                        self.next_char(); // consume 'p'
                    }
                }
                'i' | 'u'
                    if number_type_opt.is_none()
                        && !found_point
                        && !found_p
                        && matches!(self.peek_char(1), Some('0'..='9')) =>
                {
                    // Only 'i' and 'u' are allowed for hexadecimal integer numbers,
                    // 'f' is allowed only in hexadecimal floating-point numbers.
                    let number_type = self.lex_number_type_suffix()?;
                    number_type_opt.replace(number_type);

                    break;
                }
                ' ' | '\t' | '\r' | '\n' | ',' | ':' | '{' | '}' | '[' | ']' | '(' | ')' | '/' => {
                    // | '\'' | '"' => {
                    // terminator chars
                    break;
                }
                _ => {
                    return Err(AsonError::MessageWithPosition(
                        format!("Invalid char '{}' for hexadecimal number.", current_char),
                        *self.peek_position(0).unwrap(),
                    ));
                }
            }
        }

        // Check syntax

        // Empty hexadecimal number is not allowed, e.g., "0x"
        if number_buffer.is_empty() {
            return Err(AsonError::MessageWithRange(
                "Empty hexadecimal number".to_owned(),
                Range::new(&self.pop_position_from_stack(), &self.last_position),
            ));
        }

        // Tailing '.' is not allowed in hexadecimal floating-point number
        if number_buffer.ends_with('.') {
            return Err(AsonError::MessageWithRange(
                format!(
                    "Hexadecimal floating point number \"{}\" is missing the exponent.",
                    number_buffer
                ),
                Range::new(&self.pop_position_from_stack(), &self.last_position),
            ));
        }

        // Tailing 'p' is not allowed in hexadecimal floating-point number
        if number_buffer.ends_with('p') {
            return Err(AsonError::MessageWithRange(
                format!(
                    "Hexadecimal floating point number \"{}\" is missing the exponent.",
                    number_buffer
                ),
                Range::new(&self.pop_position_from_stack(), &self.last_position),
            ));
        }

        // Found '.' but no 'p' is not allowed in hexadecimal floating-point number
        if found_point && !found_p {
            return Err(AsonError::MessageWithRange(
                format!(
                    "Hexadecimal floating point number \"{}\" is missing the exponent.",
                    number_buffer
                ),
                Range::new(&self.pop_position_from_stack(), &self.last_position),
            ));
        }

        let number_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        let number_token = if found_p {
            // It is hexadecimal floating-point number.
            // The default type for floating-point is `f64`.
            let mut type_f64 = true;

            if let Some(number_type) = number_type_opt {
                match number_type {
                    NumberType::F32 => {
                        type_f64 = false;
                    }
                    NumberType::F64 => {
                        type_f64 = true;
                    }
                    _ => {
                        return Err(AsonError::MessageWithRange(
                            format!(
                                "Invalid type \"{}\" for hexadecimal floating-point numbers, only type \"f32\" and \"f64\" are allowed.",
                                number_type
                            ),
                            number_range,
                        ));
                    }
                }
            };

            number_buffer.insert_str(0, "0x");

            if type_f64 {
                let v = hexfloat2::parse::<f64>(&number_buffer).map_err(|_| {
                    // there is no detail message provided by `hexfloat2::parse`.
                    AsonError::MessageWithRange(
                        format!(
                            "Can not convert \"{}\" to f64 floating-point number.",
                            number_buffer
                        ),
                        number_range,
                    )
                })?;

                NumberToken::F64(v)
            } else {
                let v = hexfloat2::parse::<f32>(&number_buffer).map_err(|_| {
                    // there is no detail message provided by `hexfloat2::parse`.
                    AsonError::MessageWithRange(
                        format!(
                            "Can not convert \"{}\" to f32 floating-point number.",
                            number_buffer
                        ),
                        number_range,
                    )
                })?;

                NumberToken::F32(v)
            }
        } else if let Some(number_type) = number_type_opt {
            convert_integer_number_string_with_data_type(
                &number_buffer,
                16,
                number_type,
                number_range,
            )?
        } else {
            // It is hexadecimal integer number without explicit type.
            // The default type for integer is `i32`.
            let v = u32::from_str_radix(&number_buffer, 16).map_err(|_| {
                AsonError::MessageWithRange(
                    format!(
                        "Can not convert \"0x{}\" to i32 integer number.",
                        number_buffer
                    ),
                    number_range,
                )
            })?;

            NumberToken::I32(v)
        };

        Ok(TokenWithRange::new(
            Token::Number(number_token),
            number_range,
        ))
    }

    fn lex_binary_number(&mut self) -> Result<TokenWithRange, AsonError> {
        // ```diagram
        // 0b1010T  //
        // ^^    ^__// to here
        // ||_______// validated
        // |________// current char, validated
        //
        // T = terminator chars || EOF
        // ```

        self.push_peek_position_into_stack();

        self.next_char(); // consume '0'
        self.next_char(); // consume 'b'

        let mut number_buffer = String::new();
        let mut number_type_opt: Option<NumberType> = None;

        while let Some(current_char) = self.peek_char(0) {
            match current_char {
                '0' | '1' => {
                    // valid digits for binary number
                    number_buffer.push(*current_char);
                    self.next_char(); // consume digit
                }
                '_' => {
                    self.next_char(); // consume '_'
                }
                // binary form only supports integer numbers, does not support floating-point numbers
                'i' | 'u'
                    if number_type_opt.is_none()
                        && matches!(self.peek_char(1), Some('0'..='9')) =>
                {
                    let number_type = self.lex_number_type_suffix()?;
                    number_type_opt.replace(number_type);
                    break;
                }
                ' ' | '\t' | '\r' | '\n' | ',' | ':' | '{' | '}' | '[' | ']' | '(' | ')' | '/' => {
                    // | '\'' | '"' => {
                    // terminator chars
                    break;
                }
                _ => {
                    return Err(AsonError::MessageWithPosition(
                        format!("Invalid char '{}' for binary number.", current_char),
                        *self.peek_position(0).unwrap(),
                    ));
                }
            }
        }

        // Check syntax

        // Empty binary number is not allowed, e.g., "0b"
        if number_buffer.is_empty() {
            return Err(AsonError::MessageWithRange(
                "Empty binary number.".to_owned(),
                Range::new(&self.pop_position_from_stack(), &self.last_position),
            ));
        }

        let number_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        let number_token = if let Some(number_type) = number_type_opt {
            convert_integer_number_string_with_data_type(
                &number_buffer,
                2,
                number_type,
                number_range,
            )?
        } else {
            // It is binary integer number without explicit type.
            // The default type for integer is `i32`.

            let v = u32::from_str_radix(&number_buffer, 2).map_err(|_| {
                AsonError::MessageWithRange(
                    format!(
                        "Can not convert \"0b{}\" to i32 integer number.",
                        number_buffer
                    ),
                    number_range,
                )
            })?;

            NumberToken::I32(v)
        };

        Ok(TokenWithRange::new(
            Token::Number(number_token),
            number_range,
        ))
    }

    fn lex_octal_number(&mut self) -> Result<TokenWithRange, AsonError> {
        // ```diagram
        // 0o1267T  //
        // ^^    ^__// to here
        // ||_______// validated
        // |________// current char, validated
        //
        // T = terminator chars || EOF
        // ```

        self.push_peek_position_into_stack();

        self.next_char(); // consume '0'
        self.next_char(); // consume 'o'

        let mut number_buffer = String::new();
        let mut number_type_opt: Option<NumberType> = None;

        while let Some(current_char) = self.peek_char(0) {
            match current_char {
                '0'..='7' => {
                    // valid digits for binary number
                    number_buffer.push(*current_char);
                    self.next_char(); // consume digit
                }
                '_' => {
                    self.next_char(); // consume '_'
                }
                // octal form only supports integer numbers, does not support floating-point numbers
                'i' | 'u'
                    if number_type_opt.is_none()
                        && matches!(self.peek_char(1), Some('0'..='9')) =>
                {
                    let number_type = self.lex_number_type_suffix()?;
                    number_type_opt.replace(number_type);
                    break;
                }
                ' ' | '\t' | '\r' | '\n' | ',' | ':' | '{' | '}' | '[' | ']' | '(' | ')' | '/' => {
                    // | '\'' | '"' => {
                    // terminator chars
                    break;
                }
                _ => {
                    return Err(AsonError::MessageWithPosition(
                        format!("Invalid char '{}' for octal number.", current_char),
                        *self.peek_position(0).unwrap(),
                    ));
                }
            }
        }

        // Check syntax

        // Empty octal number is not allowed, e.g., "0o"
        if number_buffer.is_empty() {
            return Err(AsonError::MessageWithRange(
                "Empty octal number.".to_owned(),
                Range::new(&self.pop_position_from_stack(), &self.last_position),
            ));
        }

        let number_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        let number_token = if let Some(number_type) = number_type_opt {
            convert_integer_number_string_with_data_type(
                &number_buffer,
                8,
                number_type,
                number_range,
            )?
        } else {
            // It is octal integer number without explicit type.
            // The default type for integer is `i32`.

            let v = u32::from_str_radix(&number_buffer, 8).map_err(|_| {
                AsonError::MessageWithRange(
                    format!(
                        "Can not convert \"0o{}\" to i32 integer number.",
                        number_buffer
                    ),
                    number_range,
                )
            })?;

            NumberToken::I32(v)
        };

        Ok(TokenWithRange::new(
            Token::Number(number_token),
            number_range,
        ))
    }

    fn lex_number_type_suffix(&mut self) -> Result<NumberType, AsonError> {
        // ```diagram
        // iddT  //
        // ^^ ^__// to here
        // ||____// d = 0..9, validated
        // |_____// current char, validated
        //
        // i = i/u/f
        // d = 0..=9
        // T = terminator chars || EOF
        // ```

        self.push_peek_position_into_stack();

        let first_char = self.next_char().unwrap(); // consume char 'i/u/f'

        let mut type_buffer = String::new();
        type_buffer.push(first_char);

        while let Some(current_char) = self.peek_char(0) {
            match current_char {
                '0'..='9' => {
                    // valid char for type name
                    type_buffer.push(*current_char);

                    // consume digit
                    self.next_char();
                }
                _ => {
                    break;
                }
            }
        }

        let type_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        let number_type = convert_str_to_number_type(&type_buffer)
            .map_err(|msg| AsonError::MessageWithRange(msg, type_range))?;

        Ok(number_type)
    }

    fn lex_char(&mut self) -> Result<TokenWithRange, AsonError> {
        // ```diagram
        // 'a'?  //
        // ^  ^__// to here
        // |_____// current char, validated
        // ```

        self.push_peek_position_into_stack();

        self.next_char(); // consume `'`

        let character = match self.next_char() {
            Some(current_char) => {
                match current_char {
                    '\\' => {
                        // escape chars
                        match self.next_char() {
                            Some(escape_type) => {
                                match escape_type {
                                    '\\' => '\\',
                                    '\'' => '\'',
                                    '"' => {
                                        // double quote does not necessary to be escaped for char
                                        // however, it is still supported for consistency between chars and strings.
                                        '"'
                                    }
                                    't' => {
                                        // horizontal tabulation
                                        '\t'
                                    }
                                    'r' => {
                                        // carriage return (CR, ascii 13)
                                        '\r'
                                    }
                                    'n' => {
                                        // new line character (line feed, LF, ascii 10)
                                        '\n'
                                    }
                                    '0' => {
                                        // null char
                                        '\0'
                                    }
                                    'u' => {
                                        if self.peek_char_and_equals(0, '{') {
                                            // unicode code point, e.g. '\u{2d}', '\u{6587}'
                                            self.unescape_unicode_code_point()?
                                        } else {
                                            return Err(AsonError::MessageWithPosition(
                                                "Missing the brace for unicode escape sequence."
                                                    .to_owned(),
                                                self.last_position,
                                            ));
                                        }
                                    }
                                    _ => {
                                        return Err(AsonError::MessageWithPosition(
                                            format!("Unsupported escape char '{}'.", escape_type),
                                            self.last_position,
                                        ));
                                    }
                                }
                            }
                            None => {
                                // `\` + EOF
                                return Err(AsonError::UnexpectedEndOfDocument(
                                    "Incomplete escape character sequence.".to_owned(),
                                ));
                            }
                        }
                    }
                    '\'' => {
                        // `''`
                        return Err(AsonError::MessageWithRange(
                            "Empty char.".to_owned(),
                            Range::new(&self.pop_position_from_stack(), &self.last_position),
                        ));
                    }
                    _ => {
                        // ordinary char
                        current_char
                    }
                }
            }
            None => {
                // `'EOF`
                return Err(AsonError::UnexpectedEndOfDocument(
                    "Incomplete character.".to_owned(),
                ));
            }
        };

        // consume the right single quote
        match self.next_char() {
            Some('\'') => {
                // Ok
            }
            Some(_) => {
                // `'a?`
                return Err(AsonError::MessageWithPosition(
                    "Expected a quote for char".to_owned(),
                    self.last_position,
                ));
            }
            None => {
                // `'aEOF`
                return Err(AsonError::UnexpectedEndOfDocument(
                    "Incomplete character.".to_owned(),
                ));
            }
        }

        let character_range = Range::new(&self.pop_position_from_stack(), &self.last_position);
        Ok(TokenWithRange::new(Token::Char(character), character_range))
    }

    fn unescape_unicode_code_point(&mut self) -> Result<char, AsonError> {
        // ```diagram
        // \u{6587}?  //
        //   ^     ^__// to here
        //   |________// current char, validated
        // ```

        self.push_peek_position_into_stack();

        self.next_char(); // comsume char '{'

        let mut codepoint_buffer = String::new();

        loop {
            match self.next_char() {
                Some(current_char) => match current_char {
                    '}' => break,
                    '0'..='9' | 'a'..='f' | 'A'..='F' => codepoint_buffer.push(current_char),
                    _ => {
                        return Err(AsonError::MessageWithPosition(
                            format!(
                                "Invalid character '{}' for unicode escape sequence.",
                                current_char
                            ),
                            self.last_position,
                        ));
                    }
                },
                None => {
                    // EOF
                    return Err(AsonError::UnexpectedEndOfDocument(
                        "Incomplete unicode escape sequence.".to_owned(),
                    ));
                }
            }

            if codepoint_buffer.len() > 6 {
                break;
            }
        }

        let codepoint_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        if codepoint_buffer.len() > 6 {
            return Err(AsonError::MessageWithRange(
                "Unicode point code exceeds six digits.".to_owned(),
                codepoint_range,
            ));
        }

        if codepoint_buffer.is_empty() {
            return Err(AsonError::MessageWithRange(
                "Empty unicode code point.".to_owned(),
                codepoint_range,
            ));
        }

        let codepoint = u32::from_str_radix(&codepoint_buffer, 16).unwrap();

        if let Some(ch) = char::from_u32(codepoint) {
            // valid code point:
            // 0 to 0x10FFFF, inclusive
            //
            // ref:
            // https://doc.rust-lang.org/std/primitive.char.html
            Ok(ch)
        } else {
            Err(AsonError::MessageWithRange(
                "Invalid unicode code point.".to_owned(),
                codepoint_range,
            ))
        }
    }

    fn lex_string(&mut self) -> Result<TokenWithRange, AsonError> {
        // ```diagram
        // "abc"?  //
        // ^    ^__// to here
        // |_______// current char, validated
        // ```

        self.push_peek_position_into_stack();

        self.next_char(); // consume '"'

        let mut string_buffer = String::new();

        loop {
            match self.next_char() {
                Some(current_char) => {
                    match current_char {
                        '\\' => {
                            // save the start position of the escape sequence (i.e. the "\" char)
                            self.push_last_position_into_stack();

                            // escape chars
                            match self.next_char() {
                                Some(escape_type) => {
                                    match escape_type {
                                        '\\' => {
                                            string_buffer.push('\\');
                                        }
                                        '\'' => {
                                            // single quote does not necessary to be escaped for string
                                            // however, it is still supported for consistency between chars and strings.
                                            string_buffer.push('\'');
                                        }
                                        '"' => {
                                            string_buffer.push('"');
                                        }
                                        't' => {
                                            // horizontal tabulation
                                            string_buffer.push('\t');
                                        }
                                        'r' => {
                                            // carriage return (CR, ascii 13)
                                            string_buffer.push('\r');
                                        }
                                        'n' => {
                                            // new line character (line feed, LF, ascii 10)
                                            string_buffer.push('\n');
                                        }
                                        '0' => {
                                            // null char
                                            string_buffer.push('\0');
                                        }
                                        'u' => {
                                            if self.peek_char_and_equals(0, '{') {
                                                // unicode code point, e.g. '\u{2d}', '\u{6587}'
                                                let ch = self.unescape_unicode_code_point()?;
                                                string_buffer.push(ch);
                                            } else {
                                                return Err(AsonError::MessageWithPosition(
                                                    "Missing the brace for unicode escape sequence.".to_owned(),

                                                        self.last_position
                                                ));
                                            }
                                        }
                                        '\r' if self.peek_char_and_equals(0, '\n') => {
                                            // concatenate string
                                            self.next_char(); // consume '\n'
                                            self.consume_all_leading_whitespaces()?;
                                        }
                                        '\n' => {
                                            // concatenate string
                                            self.consume_all_leading_whitespaces()?;
                                        }
                                        _ => {
                                            return Err(AsonError::MessageWithPosition(
                                                format!(
                                                    "Unsupported escape char '{}'.",
                                                    escape_type
                                                ),
                                                self.last_position,
                                            ));
                                        }
                                    }
                                }
                                None => {
                                    // `\` + EOF
                                    return Err(AsonError::UnexpectedEndOfDocument(
                                        "Incomplete character escape sequence.".to_owned(),
                                    ));
                                }
                            }

                            // discard the saved position of the escape sequence
                            self.pop_position_from_stack();
                        }
                        '"' => {
                            // encounter the closing double quote, which
                            // means the end of the string literal.
                            break;
                        }
                        _ => {
                            // ordinary char
                            string_buffer.push(current_char);
                        }
                    }
                }
                None => {
                    // Incomplete string literal (`"...EOF`).
                    return Err(AsonError::UnexpectedEndOfDocument(
                        "Incomplete string.".to_owned(),
                    ));
                }
            }
        }

        let string_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        Ok(TokenWithRange::new(
            Token::String(string_buffer),
            string_range,
        ))
    }

    fn consume_all_leading_whitespaces(&mut self) -> Result<(), AsonError> {
        // ```diagram
        // \nssssS  //
        //   ^   ^__// to here ('s' = whitespace, 'S' = not whitespace)
        //   |______// current char, UNVALIDATED
        // ```

        loop {
            match self.peek_char(0) {
                Some(current_char) => {
                    match current_char {
                        ' ' | '\t' => {
                            self.next_char(); // consume ' ' or '\t'
                        }
                        _ => {
                            break;
                        }
                    }
                }
                None => {
                    // EOF
                    return Err(AsonError::UnexpectedEndOfDocument(
                        "Incomplete string.".to_owned(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn lex_raw_string(&mut self) -> Result<TokenWithRange, AsonError> {
        // ```diagram
        // r"abc"?  //
        // ^^    ^__// to here
        // ||_______// validated
        // |________// current char, validated
        // ```

        self.push_peek_position_into_stack();

        self.next_char(); // consume char 'r'
        self.next_char(); // consume the '"'

        let mut string_buffer = String::new();

        loop {
            match self.next_char() {
                Some(current_char) => {
                    match current_char {
                        '"' => {
                            // encounter the closing double quote, which
                            // means the end of the string literal.
                            break;
                        }
                        _ => {
                            // ordinary char
                            string_buffer.push(current_char);
                        }
                    }
                }
                None => {
                    // `r"...EOF`
                    return Err(AsonError::UnexpectedEndOfDocument(
                        "Incomplete string.".to_owned(),
                    ));
                }
            }
        }

        let string_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        Ok(TokenWithRange::new(
            Token::String(string_buffer),
            string_range,
        ))
    }

    fn lex_raw_string_with_hash_symbol(&mut self) -> Result<TokenWithRange, AsonError> {
        // ```diagram
        // r#"abc"#?  //
        // ^^^     ^__// to here
        // |||________// validated
        // ||_________// validated
        // |__________// current char, validated
        //
        // hash symbol = '#', i.e. the pound sign
        // ```

        self.push_peek_position_into_stack();

        self.next_char(); // consume 'r'
        self.next_char(); // consume '#'
        self.next_char(); // consume '"'

        let mut string_buffer = String::new();

        loop {
            match self.next_char() {
                Some(current_char) => {
                    match current_char {
                        '"' if self.peek_char_and_equals(0, '#') => {
                            // it is the end of the string
                            self.next_char(); // consume '#'
                            break;
                        }
                        _ => {
                            // ordinary char
                            string_buffer.push(current_char);
                        }
                    }
                }
                None => {
                    // `r#"...EOF`
                    return Err(AsonError::UnexpectedEndOfDocument(
                        "Incomplete string.".to_owned(),
                    ));
                }
            }
        }

        let string_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        Ok(TokenWithRange::new(
            Token::String(string_buffer),
            string_range,
        ))
    }

    fn lex_auto_trimmed_string(&mut self) -> Result<TokenWithRange, AsonError> {
        // ```diagram
        // """\n                    //
        // ^^^  auto-trimmed string //
        // |||  ...\n               //
        // |||  """?                //
        // |||     ^________________// to here ('?' = any chars or EOF)
        // |||______________________// validated
        // ||_______________________// validated
        // |________________________// current char, validated
        //
        // note:
        // - the '\n' of the first line is necessary.
        // - the closed `"""` must be started with a new line.
        // ```

        self.push_peek_position_into_stack();

        self.next_char(); // consume the 1st '"'
        self.next_char(); // consume the 2nd '"'
        self.next_char(); // consume the 3rd '"'

        if self.peek_char_and_equals(0, '\n') {
            self.next_char(); // consume '\n'
        } else if self.peek_char_and_equals(0, '\r') && self.peek_char_and_equals(1, '\n') {
            self.next_char(); // consume '\r'
            self.next_char(); // consume '\n'
        } else {
            return Err(AsonError::MessageWithPosition(
                "The content of auto-trimmed string should start on a new line.".to_owned(),
                self.last_position,
            ));
        }

        let mut lines = vec![];
        let mut current_line = vec![];

        loop {
            match self.next_char() {
                Some(current_char) => {
                    match current_char {
                        '\n' => {
                            current_line.push('\n');
                            lines.push(current_line);

                            current_line = vec![];
                        }
                        '\r' if self.peek_char_and_equals(0, '\n') => {
                            self.next_char(); // consume '\n'

                            current_line.push('\r');
                            current_line.push('\n');
                            lines.push(current_line);

                            current_line = vec![];
                        }
                        '"' if current_line.iter().all(|&c| c == ' ' || c == '\t')
                            && self.peek_char_and_equals(0, '"')
                            && self.peek_char_and_equals(1, '"') =>
                        {
                            // it is the end of string
                            self.next_char(); // consume '"'
                            self.next_char(); // consume '"'
                            break;
                        }
                        _ => {
                            // ordinary char
                            current_line.push(current_char);
                        }
                    }
                }
                None => {
                    // `"""\n...EOF`
                    return Err(AsonError::UnexpectedEndOfDocument(
                        "Incomplete string.".to_owned(),
                    ));
                }
            }
        }

        let range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        if lines.is_empty() {
            return Ok(TokenWithRange::new(Token::String(String::new()), range));
        }

        // Calculate leading spaces of each line.
        //
        // The empty lines would be excluded
        let numbers: Vec<usize> = lines
            .iter()
            .filter(|line| {
                let is_empty = (line.len() == 1 && line[0] == '\n')
                    || (line.len() == 2 && line[0] == '\r' && line[1] == '\n');
                !is_empty
            })
            .map(|line| {
                let mut count = 0;
                while count < line.len() {
                    if !(line[count] == ' ' || line[count] == '\t') {
                        break;
                    }
                    count += 1;
                }
                count
            })
            .collect();

        let min_number = *numbers.iter().min().unwrap_or(&0);

        // Trim leading spaces for each line
        lines
            .iter_mut()
            .filter(|line| {
                let is_empty = (line.len() == 1 && line[0] == '\n')
                    || (line.len() == 2 && line[0] == '\r' && line[1] == '\n');
                !is_empty
            })
            .for_each(|line| {
                line.drain(0..min_number);
            });

        // trim the ending '\n' or "\r\n"
        let last_index = lines.len() - 1;
        let last_line = &mut lines[last_index];

        if matches!(last_line.last(), Some('\n')) {
            last_line.pop();
        }

        if matches!(last_line.last(), Some('\r')) {
            last_line.pop();
        }

        let content = lines
            .iter()
            .map(|line| line.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("");

        Ok(TokenWithRange::new(Token::String(content), range))
    }

    fn lex_datetime(&mut self) -> Result<TokenWithRange, AsonError> {
        // ```diagram
        // d"2024-03-16T16:30:50+08:00"?  //
        // ^^                          ^__// to here
        // ||_____________________________// validated
        // |______________________________// current char, validated
        // ```

        self.push_peek_position_into_stack();

        self.next_char(); // consume the char 'd'
        self.next_char(); // consume left quote

        let mut date_buffer = String::new();

        loop {
            match self.next_char() {
                Some(current_char) => {
                    match current_char {
                        '"' => {
                            // it is the end of the date time string
                            break;
                        }
                        '0'..='9' | '-' | ':' | ' ' | 't' | 'T' | 'z' | 'Z' | '+' => {
                            // valid chars
                            date_buffer.push(current_char);
                        }
                        _ => {
                            return Err(AsonError::MessageWithPosition(
                                format!("Invalid char '{}' for datetime.", current_char),
                                self.last_position,
                            ));
                        }
                    }
                }
                None => {
                    // d"...EOF
                    return Err(AsonError::UnexpectedEndOfDocument(
                        "Incomplete date time.".to_owned(),
                    ));
                }
            }
        }

        let date_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        let len = date_buffer.len();

        if len == 10 {
            // YYYY-MM-DD
            date_buffer.push_str("T00:00:00Z");
        } else if len == 19 {
            // YYYY-MM-DD HH:mm:ss
            date_buffer.push('Z');
        } else if len == 20 || len == 25 {
            // ref3339
            // YYYY-MM-DDTHH:mm:ssZ
            // YYYY-MM-DDTHH:mm:ss+08:00
        } else {
            return Err(AsonError::MessageWithRange(
                format!(
                    "Invalid date time string: {}, the required format is: \"YYYY-MM-DD HH:mm:ss\"",
                    date_buffer
                ),
                date_range,
            ));
        }

        let rfc3339 = DateTime::parse_from_rfc3339(&date_buffer).map_err(|_| {
            AsonError::MessageWithRange(
                format!(
                    "Can not convert the string \"{}\" to datetime.",
                    date_buffer
                ),
                date_range,
            )
        })?;

        Ok(TokenWithRange::new(Token::Date(rfc3339), date_range))
    }

    fn lex_hexadecimal_byte_data(&mut self) -> Result<TokenWithRange, AsonError> {
        // ```diagram
        // h"00 11 aa bb"?  //
        // ^^            ^__// to here
        // ||_______________// validated
        // |________________// current char, validated
        // ```

        let consume_zero_or_more_whitespaces = |iter: &mut Lexer| -> Result<usize, AsonError> {
            // exit when encounting non-whitespaces or EOF
            let mut amount: usize = 0;

            while let Some(' ' | '\t' | '\r' | '\n') = iter.peek_char(0) {
                amount += 1;
                iter.next_char();
            }

            Ok(amount)
        };

        let consume_one_or_more_whitespaces = |iter: &mut Lexer| -> Result<usize, AsonError> {
            let mut amount: usize = 0;

            loop {
                match iter.peek_char(0) {
                    Some(current_char) => {
                        match current_char {
                            ' ' | '\t' | '\r' | '\n' => {
                                // consume whitespace
                                iter.next_char();
                                amount += 1;
                            }
                            _ => {
                                if amount > 0 {
                                    break;
                                } else {
                                    return Err(AsonError::MessageWithPosition(
                                        "Expect a whitespace between the hexadecimal byte data digits."
                                            .to_owned(),
                                        iter.last_position
                                    ));
                                }
                            }
                        }
                    }
                    None => {
                        // h"...EOF
                        return Err(AsonError::UnexpectedEndOfDocument(
                            "Incomplete hexadecimal byte data.".to_owned(),
                        ));
                    }
                }
            }

            Ok(amount)
        };

        self.push_peek_position_into_stack();

        self.next_char(); // consume char 'h'
        self.next_char(); // consume quote '"'

        let mut bytes: Vec<u8> = Vec::new();
        let mut hex_number_with_two_digits: [char; 2] = ['0', '0'];

        consume_zero_or_more_whitespaces(self)?;

        // Collect hexadecimal byte data
        loop {
            if self.peek_char_and_equals(0, '"') {
                break;
            }

            // read two hex digits
            for digit in &mut hex_number_with_two_digits {
                match self.next_char() {
                    Some(current_char) => match current_char {
                        'a'..='f' | 'A'..='F' | '0'..='9' => {
                            *digit = current_char;
                        }
                        _ => {
                            return Err(AsonError::MessageWithPosition(
                                format!(
                                    "Invalid digit '{}' for hexadecimal byte data.",
                                    current_char
                                ),
                                self.last_position,
                            ));
                        }
                    },
                    None => {
                        return Err(AsonError::UnexpectedEndOfDocument(
                            "Incomplete hexadecimal byte data.".to_owned(),
                        ));
                    }
                }
            }

            let byte_string = String::from_iter(hex_number_with_two_digits);
            let byte_value = u8::from_str_radix(&byte_string, 16).unwrap();
            bytes.push(byte_value);

            if self.peek_char_and_equals(0, '"') {
                break;
            }

            // consume at lease one whitespace
            consume_one_or_more_whitespaces(self)?;
        }

        self.next_char(); // consume '"'

        let bytes_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        Ok(TokenWithRange::new(
            Token::HexadecimalByteData(bytes),
            bytes_range,
        ))
    }

    fn lex_line_comment(&mut self) {
        // ```diagram
        // //.....?[\r]\n
        // ^^     ^__// to here ('?' = any char or EOF)
        // ||________// validated
        // |_________// current char, validated
        //
        // ```

        // note that the trailing '\n' or '\r\n' does not belong to line comment

        self.next_char(); // consume the 1st '/'
        self.next_char(); // consume the 2nd '/'

        while let Some(current_char) = self.peek_char(0) {
            // ignore all chars until encountering '\n' or '\r\n'.
            // do not consume '\n' or '\r\n' since they do not belong to the line comment token.
            match current_char {
                '\n' => {
                    break;
                }
                '\r' if self.peek_char_and_equals(0, '\n') => {
                    break;
                }
                _ => {
                    // comment_buffer.push(*current_char);

                    self.next_char(); // consume char
                }
            }
        }
    }

    fn lex_block_comment(&mut self) -> Result<(), AsonError> {
        // ```diagram
        // /*...*/?  //
        // ^^     ^__// to here
        // ||________// validated
        // |_________// current char, validated
        // ```

        self.next_char(); // consume '/'
        self.next_char(); // consume '*'

        let mut block_comment_depth = 1;

        loop {
            match self.next_char() {
                Some(current_char) => {
                    match current_char {
                        '/' if self.peek_char_and_equals(0, '*') => {
                            // nested block comment
                            self.next_char(); // consume '*'

                            // increase depth
                            block_comment_depth += 1;
                        }
                        '*' if self.peek_char_and_equals(0, '/') => {
                            self.next_char(); // consume '/'

                            // decrease depth
                            block_comment_depth -= 1;

                            // check pairs
                            if block_comment_depth == 0 {
                                break;
                            }
                        }
                        _ => {
                            // ignore all chars except "/*" and "*/"
                            // note that line comments within block comments are ignored also.
                        }
                    }
                }
                None => {
                    let msg = if block_comment_depth > 1 {
                        "Incomplete nested block comment.".to_owned()
                    } else {
                        "Incomplete block comment.".to_owned()
                    };

                    return Err(AsonError::UnexpectedEndOfDocument(msg));
                }
            }
        }

        Ok(())
    }
}

enum NumberType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

impl Display for NumberType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumberType::I8 => write!(f, "i8"),
            NumberType::I16 => write!(f, "i16"),
            NumberType::I32 => write!(f, "i32"),
            NumberType::I64 => write!(f, "i64"),
            NumberType::U8 => write!(f, "u8"),
            NumberType::U16 => write!(f, "u16"),
            NumberType::U32 => write!(f, "u32"),
            NumberType::U64 => write!(f, "u64"),
            NumberType::F32 => write!(f, "f32"),
            NumberType::F64 => write!(f, "f64"),
        }
    }
}

fn convert_integer_number_string_with_data_type(
    number_string: &str,
    radix: u32,
    number_type: NumberType,
    number_range: Range,
) -> Result<NumberToken, AsonError> {
    let prefix_name = match radix {
        2 => "0b",
        8 => "0o",
        16 => "0x",
        _ => "",
    };

    let token = match number_type {
        NumberType::I8 => {
            let v = u8::from_str_radix(number_string, radix).map_err(|_| {
                AsonError::MessageWithRange(
                    format!(
                        "Can not convert \"{}{}\" to i8 integer number.",
                        prefix_name, number_string,
                    ),
                    number_range,
                )
            })?;

            NumberToken::I8(v)
        }
        NumberType::U8 => {
            let v = u8::from_str_radix(number_string, radix).map_err(|_| {
                AsonError::MessageWithRange(
                    format!(
                        "Can not convert \"{}{}\" to u8 integer number.",
                        prefix_name, number_string,
                    ),
                    number_range,
                )
            })?;

            NumberToken::U8(v)
        }
        NumberType::I16 => {
            let v = u16::from_str_radix(number_string, radix).map_err(|_| {
                AsonError::MessageWithRange(
                    format!(
                        "Can not convert \"{}{}\" to i16 integer number.",
                        prefix_name, number_string,
                    ),
                    number_range,
                )
            })?;

            NumberToken::I16(v)
        }
        NumberType::U16 => {
            let v = u16::from_str_radix(number_string, radix).map_err(|_| {
                AsonError::MessageWithRange(
                    format!(
                        "Can not convert \"{}{}\" to u16 integer number.",
                        prefix_name, number_string,
                    ),
                    number_range,
                )
            })?;

            NumberToken::U16(v)
        }
        NumberType::I32 => {
            let v = u32::from_str_radix(number_string, radix).map_err(|_| {
                AsonError::MessageWithRange(
                    format!(
                        "Can not convert \"{}{}\" to i32 integer number.",
                        prefix_name, number_string
                    ),
                    number_range,
                )
            })?;

            NumberToken::I32(v)
        }
        NumberType::U32 => {
            let v = u32::from_str_radix(number_string, radix).map_err(|_| {
                AsonError::MessageWithRange(
                    format!(
                        "Can not convert \"{}{}\" to u32 integer number.",
                        prefix_name, number_string
                    ),
                    number_range,
                )
            })?;

            NumberToken::U32(v)
        }
        NumberType::I64 => {
            let v = u64::from_str_radix(number_string, radix).map_err(|_| {
                AsonError::MessageWithRange(
                    format!(
                        "Can not convert \"{}{}\" to i64 integer number.",
                        prefix_name, number_string
                    ),
                    number_range,
                )
            })?;

            NumberToken::I64(v)
        }
        NumberType::U64 => {
            let v = u64::from_str_radix(number_string, radix).map_err(|_| {
                AsonError::MessageWithRange(
                    format!(
                        "Can not convert \"{}{}\" to u64 integer number.",
                        prefix_name, number_string
                    ),
                    number_range,
                )
            })?;

            NumberToken::U64(v)
        }
        NumberType::F32 | NumberType::F64 => {
            unreachable!()
        }
    };

    Ok(token)
}

fn convert_str_to_number_type(s: &str) -> Result<NumberType, String> {
    let t = match s {
        "i8" => NumberType::I8,
        "i16" => NumberType::I16,
        "i32" => NumberType::I32,
        "i64" => NumberType::I64,
        "u8" => NumberType::U8,
        "u16" => NumberType::U16,
        "u32" => NumberType::U32,
        "u64" => NumberType::U64,
        "f32" => NumberType::F32,
        "f64" => NumberType::F64,
        _ => {
            return Err(format!("Invalid number type \"{}\".", s));
        }
    };

    Ok(t)
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use pretty_assertions::assert_eq;

    use crate::{
        char_with_position::CharsWithPositionIter,
        error::AsonError,
        lexer::{NumberToken, TokenWithRange},
        peekable_iter::PeekableIter,
        position::Position,
        range::Range,
    };

    use super::{Lexer, PEEK_BUFFER_LENGTH_LEX, Token};

    impl Token {
        pub fn new_variant(type_name: &str, member_name: &str) -> Self {
            Token::Variant(type_name.to_owned(), member_name.to_owned())
        }

        pub fn new_identifier(s: &str) -> Self {
            Token::Identifier(s.to_owned())
        }

        pub fn new_string(s: &str) -> Self {
            Token::String(s.to_owned())
        }
    }

    /// Helper function to lex tokens from a string
    fn lex_from_str(s: &str) -> Result<Vec<TokenWithRange>, AsonError> {
        let mut chars = s.chars();
        let mut char_position_iter = CharsWithPositionIter::new(&mut chars);
        let mut peekable_char_position_iter =
            PeekableIter::new(&mut char_position_iter, PEEK_BUFFER_LENGTH_LEX);
        let lexer = Lexer::new(&mut peekable_char_position_iter);

        // Collect tokens
        //
        // do not use `iter.collect::<Vec<_>>()` to collect tokens,
        // because the `Lexer` throws exceptions via `next() -> Option<Result<...>>`.
        //
        // if we use `collect()`, once an error occurs,
        // the iterator wouldn't stop immediately, instead, it would continue to iterate until the end,
        let mut token_with_ranges = vec![];
        for result in lexer {
            match result {
                Ok(twr) => token_with_ranges.push(twr),
                Err(e) => return Err(e),
            }
        }

        Ok(token_with_ranges)
    }

    /// Helper function to lex tokens from a string, without location info
    fn lex_from_str_without_location(s: &str) -> Result<Vec<Token>, AsonError> {
        let tokens = lex_from_str(s)?
            .into_iter()
            .map(|e| e.token)
            .collect::<Vec<Token>>();
        Ok(tokens)
    }

    #[test]
    fn test_lex_whitespaces() {
        assert_eq!(lex_from_str_without_location("  ").unwrap(), vec![]);

        assert_eq!(
            lex_from_str_without_location("()").unwrap(),
            vec![Token::OpeningParenthesis, Token::ClosingParenthesis]
        );

        assert_eq!(
            lex_from_str_without_location("(  )").unwrap(),
            vec![Token::OpeningParenthesis, Token::ClosingParenthesis]
        );

        assert_eq!(
            lex_from_str_without_location("( , )").unwrap(),
            vec![Token::OpeningParenthesis, Token::ClosingParenthesis]
        );

        assert_eq!(
            lex_from_str_without_location("( , , ,, )").unwrap(),
            vec![Token::OpeningParenthesis, Token::ClosingParenthesis]
        );

        assert_eq!(
            lex_from_str_without_location("(\t\r\n\n\n)").unwrap(),
            vec![Token::OpeningParenthesis, Token::ClosingParenthesis,]
        );

        assert_eq!(
            lex_from_str_without_location("(\n,\n)").unwrap(),
            vec![Token::OpeningParenthesis, Token::ClosingParenthesis]
        );

        // Testing the ranges

        assert_eq!(lex_from_str("  ").unwrap(), vec![]);

        assert_eq!(
            lex_from_str("()").unwrap(),
            vec![
                TokenWithRange::new(Token::OpeningParenthesis, Range::from_detail(0, 0, 0, 1)),
                TokenWithRange::new(Token::ClosingParenthesis, Range::from_detail(1, 0, 1, 1)),
            ]
        );

        assert_eq!(
            lex_from_str("(  )").unwrap(),
            vec![
                TokenWithRange::new(Token::OpeningParenthesis, Range::from_detail(0, 0, 0, 1)),
                TokenWithRange::new(Token::ClosingParenthesis, Range::from_detail(3, 0, 3, 1)),
            ]
        );

        // "(\t\r\n\n\n)"
        //  _--____--__-
        //  0  2   4 5 6    // index
        //  0  0   1 2 3    // line
        //  0  2   0 0 1    // column
        //  1  2   1 1 1    // length

        assert_eq!(
            lex_from_str("(\t\r\n\n\n)").unwrap(),
            vec![
                TokenWithRange::new(Token::OpeningParenthesis, Range::from_detail(0, 0, 0, 1)),
                TokenWithRange::new(Token::ClosingParenthesis, Range::from_detail(6, 3, 0, 1)),
            ]
        );
    }

    #[test]
    fn test_lex_punctuations() {
        assert_eq!(
            lex_from_str_without_location(",:{}[]()+-").unwrap(),
            vec![
                Token::Colon,
                Token::OpeningBrace,
                Token::ClosingBrace,
                Token::OpeningBracket,
                Token::ClosingBracket,
                Token::OpeningParenthesis,
                Token::ClosingParenthesis,
                Token::_Plus,
                Token::_Minus
            ]
        );
    }

    #[test]
    fn test_lex_identifier() {
        assert_eq!(
            lex_from_str_without_location("name").unwrap(),
            vec![Token::new_identifier("name")]
        );

        assert_eq!(
            lex_from_str_without_location("(name)").unwrap(),
            vec![
                Token::OpeningParenthesis,
                Token::new_identifier("name"),
                Token::ClosingParenthesis,
            ]
        );

        assert_eq!(
            lex_from_str_without_location("( a )").unwrap(),
            vec![
                Token::OpeningParenthesis,
                Token::new_identifier("a"),
                Token::ClosingParenthesis,
            ]
        );

        assert_eq!(
            lex_from_str_without_location("a__b__c").unwrap(),
            vec![Token::new_identifier("a__b__c")]
        );

        assert_eq!(
            lex_from_str_without_location("foo bar").unwrap(),
            vec![Token::new_identifier("foo"), Token::new_identifier("bar")]
        );

        assert_eq!(
            lex_from_str_without_location("αβγ 文字 🍞🥛").unwrap(),
            vec![
                Token::new_identifier("αβγ"),
                Token::new_identifier("文字"),
                Token::new_identifier("🍞🥛"),
            ]
        );

        // Testing the ranges

        assert_eq!(
            lex_from_str("hello ASON").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::new_identifier("hello"),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 5)
                ),
                TokenWithRange::new(
                    Token::new_identifier("ASON"),
                    Range::from_position_and_length(&Position::new(6, 0, 6), 4)
                )
            ]
        );

        // err: invalid char
        assert!(matches!(
            lex_from_str("abc&xyz"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 3,
                    line: 0,
                    column: 3
                }
            ))
        ));
    }

    #[test]
    fn test_lex_keyword() {
        assert_eq!(
            lex_from_str_without_location("true").unwrap(),
            vec![Token::Boolean(true)]
        );

        assert_eq!(
            lex_from_str_without_location("false").unwrap(),
            vec![Token::Boolean(false)]
        );

        assert_eq!(
            lex_from_str_without_location("true false").unwrap(),
            vec![Token::Boolean(true), Token::Boolean(false)]
        );

        assert_eq!(
            lex_from_str_without_location("Inf Inf_f32 Inf_f64").unwrap(),
            vec![
                Token::Number(NumberToken::F64(f64::INFINITY)),
                Token::Number(NumberToken::F32(f32::INFINITY)),
                Token::Number(NumberToken::F64(f64::INFINITY)),
            ]
        );

        let nans = lex_from_str_without_location("NaN NaN_f32 NaN_f64").unwrap();
        assert!(matches!(nans[0], Token::Number(NumberToken::F64(v)) if v.is_nan()));
        assert!(matches!(nans[1], Token::Number(NumberToken::F32(v)) if v.is_nan()));
        assert!(matches!(nans[2], Token::Number(NumberToken::F64(v)) if v.is_nan()));

        assert_eq!(
            lex_from_str_without_location("Inf_i32").unwrap(),
            vec![Token::new_identifier("Inf_i32")]
        );

        assert_eq!(
            lex_from_str_without_location("NaN_i32").unwrap(),
            vec![Token::new_identifier("NaN_i32")]
        );

        // Testing the ranges

        // "[\n    true\n    false\n]"
        //  01 234567890 1234567890 1   // index
        //  00 111111111 2222222222 3   // line
        //  01 012345678 0123456789 0   // column
        //  11     4   1     5    1 1   // length

        assert_eq!(
            lex_from_str("[\n    true\n    false\n]").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::OpeningBracket,
                    Range::from_position_and_length(&Position::new(0, 0, 0), 1)
                ),
                TokenWithRange::new(
                    Token::Boolean(true),
                    Range::from_position_and_length(&Position::new(6, 1, 4), 4)
                ),
                TokenWithRange::new(
                    Token::Boolean(false),
                    Range::from_position_and_length(&Position::new(15, 2, 4), 5)
                ),
                TokenWithRange::new(
                    Token::ClosingBracket,
                    Range::from_position_and_length(&Position::new(21, 3, 0), 1)
                ),
            ]
        );
    }

    #[test]
    fn test_lex_decimal_number() {
        assert_eq!(
            lex_from_str_without_location("(211)").unwrap(),
            vec![
                Token::OpeningParenthesis,
                Token::Number(NumberToken::I32(211)),
                Token::ClosingParenthesis,
            ]
        );

        assert_eq!(
            lex_from_str_without_location("211").unwrap(),
            vec![Token::Number(NumberToken::I32(211))]
        );

        assert_eq!(
            lex_from_str_without_location("-2017").unwrap(),
            vec![Token::_Minus, Token::Number(NumberToken::I32(2017))]
        );

        assert_eq!(
            lex_from_str_without_location("+2024").unwrap(),
            vec![Token::_Plus, Token::Number(NumberToken::I32(2024))]
        );

        assert_eq!(
            lex_from_str_without_location("223_211").unwrap(),
            vec![Token::Number(NumberToken::I32(223_211))]
        );

        assert_eq!(
            lex_from_str_without_location("223 211").unwrap(),
            vec![
                Token::Number(NumberToken::I32(223)),
                Token::Number(NumberToken::I32(211)),
            ]
        );

        // Testing the ranges

        assert_eq!(
            lex_from_str("223 211").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(223)),
                    Range::from_position_and_length(&Position::new(0, 0, 0,), 3)
                ),
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(211)),
                    Range::from_position_and_length(&Position::new(4, 0, 4,), 3)
                ),
            ]
        );

        // err: invalid char for decimal number
        assert!(matches!(
            lex_from_str("12x34"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 2,
                    line: 0,
                    column: 2,
                }
            ))
        ));

        // err: number width overflow
        assert!(matches!(
            lex_from_str("4_294_967_296"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 12,
                        line: 0,
                        column: 12,
                    }
                }
            ))
        ));

        // err: leading 0 in decimal number
        assert!(matches!(
            lex_from_str("0123"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 3,
                        line: 0,
                        column: 3,
                    }
                }
            ))
        ));
    }

    #[allow(clippy::approx_constant)]
    #[test]
    fn test_lex_decimal_floating_point_number() {
        assert_eq!(
            lex_from_str_without_location("3.14").unwrap(),
            vec![Token::Number(NumberToken::F64(3.14))]
        );

        assert_eq!(
            lex_from_str_without_location("+1.414").unwrap(),
            vec![Token::_Plus, Token::Number(NumberToken::F64(1.414))]
        );

        assert_eq!(
            lex_from_str_without_location("-2.718").unwrap(),
            vec![Token::_Minus, Token::Number(NumberToken::F64(2.718))]
        );

        assert_eq!(
            lex_from_str_without_location("2.998e8").unwrap(),
            vec![Token::Number(NumberToken::F64(2.998e8))]
        );

        assert_eq!(
            lex_from_str_without_location("2.998e+8").unwrap(),
            vec![Token::Number(NumberToken::F64(2.998e+8))]
        );

        assert_eq!(
            lex_from_str_without_location("6.626e-34").unwrap(),
            vec![Token::Number(NumberToken::F64(6.626e-34))]
        );

        // err: incomplete floating point number since ends with '.'
        assert!(matches!(
            lex_from_str("123."),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 3,
                        line: 0,
                        column: 3,
                    }
                }
            ))
        ));

        // err: incomplete floating point number since ends with 'e'
        assert!(matches!(
            lex_from_str("123e"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 3,
                        line: 0,
                        column: 3,
                    }
                }
            ))
        ));

        // err: multiple '.' (point)
        assert!(matches!(
            lex_from_str("1.23.456"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 4,
                    line: 0,
                    column: 4,
                }
            ))
        ));

        // err: multiple 'e' (exponent)
        assert!(matches!(
            lex_from_str("1e23e456"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 4,
                    line: 0,
                    column: 4,
                }
            ))
        ));

        // err: starts with dot
        assert!(matches!(
            lex_from_str(".123"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 0,
                    line: 0,
                    column: 0,
                }
            ))
        ));
    }

    #[test]
    fn test_lex_decimal_number_with_explicit_type() {
        // general
        {
            assert_eq!(
                lex_from_str_without_location("11i8").unwrap(),
                vec![Token::Number(NumberToken::I8(11))]
            );

            assert_eq!(
                lex_from_str_without_location("11_i8").unwrap(),
                vec![Token::Number(NumberToken::I8(11))]
            );

            assert_eq!(
                lex_from_str_without_location("11__i8").unwrap(),
                vec![Token::Number(NumberToken::I8(11))]
            );

            // Testing the ranges

            // "101_i16 103_u32"
            //  012345678901234  // index
            assert_eq!(
                lex_from_str("101_i16 103_u32").unwrap(),
                vec![
                    TokenWithRange::new(
                        Token::Number(NumberToken::I16(101)),
                        Range::from_position_and_length(&Position::new(0, 0, 0), 7)
                    ),
                    TokenWithRange::new(
                        Token::Number(NumberToken::U32(103)),
                        Range::from_position_and_length(&Position::new(8, 0, 8), 7)
                    ),
                ]
            );
        }

        // i8/u8
        {
            assert_eq!(
                lex_from_str_without_location("127_i8").unwrap(),
                vec![Token::Number(NumberToken::I8(127))]
            );

            assert_eq!(
                lex_from_str_without_location("255_u8").unwrap(),
                vec![Token::Number(NumberToken::U8(255))]
            );

            // err: number width overflow
            assert!(matches!(
                lex_from_str("256_u8"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 5,
                            line: 0,
                            column: 5
                        }
                    }
                ))
            ));
        }

        // i16/u16
        {
            assert_eq!(
                lex_from_str_without_location("32767_i16").unwrap(),
                vec![Token::Number(NumberToken::I16(32767))]
            );

            assert_eq!(
                lex_from_str_without_location("65535_u16").unwrap(),
                vec![Token::Number(NumberToken::U16(65535))]
            );

            // err: number width overflow
            assert!(matches!(
                lex_from_str("65536_u16"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 8,
                            line: 0,
                            column: 8
                        }
                    }
                ))
            ));
        }

        // i32/u32
        {
            assert_eq!(
                lex_from_str_without_location("2_147_483_647_i32").unwrap(),
                vec![Token::Number(NumberToken::I32(2_147_483_647i32 as u32))]
            );

            assert_eq!(
                lex_from_str_without_location("4_294_967_295_u32").unwrap(),
                vec![Token::Number(NumberToken::U32(u32::MAX))]
            );

            // err: number width overflow
            assert!(matches!(
                lex_from_str("4_294_967_296_u32"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 16,
                            line: 0,
                            column: 16
                        }
                    }
                ))
            ));
        }

        // i64/u64
        {
            assert_eq!(
                lex_from_str_without_location("9_223_372_036_854_775_807_i64").unwrap(),
                vec![Token::Number(NumberToken::I64(
                    9_223_372_036_854_775_807i64 as u64
                )),]
            );

            assert_eq!(
                lex_from_str_without_location("18_446_744_073_709_551_615_u64").unwrap(),
                vec![Token::Number(NumberToken::U64(u64::MAX))]
            );

            // err: number width overflow
            assert!(matches!(
                lex_from_str("18_446_744_073_709_551_616_u64"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 29,
                            line: 0,
                            column: 29
                        }
                    }
                ))
            ));
        }

        // f32
        {
            assert_eq!(
                lex_from_str_without_location("3.402_823_5e+38_f32").unwrap(),
                vec![Token::Number(NumberToken::F32(3.402_823_5e38f32))]
            );

            assert_eq!(
                lex_from_str_without_location("1.175_494_4e-38_f32").unwrap(),
                vec![Token::Number(NumberToken::F32(1.175_494_4e-38f32))]
            );

            // err: number width overflow
            assert!(matches!(
                lex_from_str("3.4e39_f32"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 9,
                            line: 0,
                            column: 9
                        }
                    }
                ))
            ));
        }

        // f64
        {
            assert_eq!(
                lex_from_str_without_location("1.797_693_134_862_315_7e+308_f64").unwrap(),
                vec![Token::Number(NumberToken::F64(
                    1.797_693_134_862_315_7e308_f64
                )),]
            );

            assert_eq!(
                lex_from_str_without_location("2.2250738585072014e-308_f64").unwrap(),
                vec![Token::Number(NumberToken::F64(2.2250738585072014e-308f64)),]
            );

            // err: number width overflow
            assert!(matches!(
                lex_from_str("1.8e309_f64"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 10,
                            line: 0,
                            column: 10
                        }
                    }
                ))
            ));
        }
    }

    #[test]
    fn test_lex_hexadecimal_number() {
        assert_eq!(
            lex_from_str_without_location("0xabcd").unwrap(),
            vec![Token::Number(NumberToken::I32(0xabcd))]
        );

        assert_eq!(
            lex_from_str_without_location("-0xaabb").unwrap(),
            vec![Token::_Minus, Token::Number(NumberToken::I32(0xaabb))]
        );

        assert_eq!(
            lex_from_str_without_location("+0xccdd").unwrap(),
            vec![Token::_Plus, Token::Number(NumberToken::I32(0xccdd))]
        );

        // Testing the ranges

        assert_eq!(
            lex_from_str("0xab 0xdef").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(0xab)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 4)
                ),
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(0xdef)),
                    Range::from_position_and_length(&Position::new(5, 0, 5,), 5)
                ),
            ]
        );

        // err: invalid char for hex number
        assert!(matches!(
            lex_from_str("0x1234xyz"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 6,
                    line: 0,
                    column: 6,
                }
            ))
        ));

        // err: number width overflow
        assert!(matches!(
            lex_from_str("0x1_0000_0000"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0
                    },
                    end_included: Position {
                        index: 12,
                        line: 0,
                        column: 12
                    }
                }
            ))
        ));

        // err: empty hex number
        assert!(matches!(
            lex_from_str("0x"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0
                    },
                    end_included: Position {
                        index: 1,
                        line: 0,
                        column: 1
                    }
                }
            ))
        ));
    }

    #[test]
    fn test_lex_hexadecimal_number_with_explicit_type() {
        // general
        {
            assert_eq!(
                lex_from_str_without_location("0x11i8").unwrap(),
                vec![Token::Number(NumberToken::I8(0x11))]
            );

            assert_eq!(
                lex_from_str_without_location("0x11_i8").unwrap(),
                vec![Token::Number(NumberToken::I8(0x11))]
            );

            assert_eq!(
                lex_from_str_without_location("0x11__i8").unwrap(),
                vec![Token::Number(NumberToken::I8(0x11))]
            );

            // Testing the ranges

            // "0x101_i16 0x103_u32"
            //  0123456789012345678  // index
            assert_eq!(
                lex_from_str("0x101_i16 0x103_u32").unwrap(),
                vec![
                    TokenWithRange::new(
                        Token::Number(NumberToken::I16(0x101)),
                        Range::from_position_and_length(&Position::new(0, 0, 0), 9)
                    ),
                    TokenWithRange::new(
                        Token::Number(NumberToken::U32(0x103)),
                        Range::from_position_and_length(&Position::new(10, 0, 10), 9)
                    ),
                ]
            );
        }

        // i8/u8
        {
            assert_eq!(
                lex_from_str_without_location("0x7f_i8").unwrap(),
                vec![Token::Number(NumberToken::I8(0x7f_i8 as u8))]
            );

            assert_eq!(
                lex_from_str_without_location("0xff_u8").unwrap(),
                vec![Token::Number(NumberToken::U8(0xff_u8))]
            );

            // err: number width overflow
            assert!(matches!(
                lex_from_str("0x1_ff_u8"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 8,
                            line: 0,
                            column: 8
                        }
                    }
                ))
            ));
        }

        // i16/u16
        {
            assert_eq!(
                lex_from_str_without_location("0x7fff_i16").unwrap(),
                vec![Token::Number(NumberToken::I16(0x7fff_i16 as u16))]
            );

            assert_eq!(
                lex_from_str_without_location("0xffff_u16").unwrap(),
                vec![Token::Number(NumberToken::U16(0xffff_u16))]
            );

            // err: number width overflow
            assert!(matches!(
                lex_from_str("0x1_ffff_u16"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 11,
                            line: 0,
                            column: 11
                        }
                    }
                ))
            ));
        }

        // i32/u32
        {
            assert_eq!(
                lex_from_str_without_location("0x7fff_ffff_i32").unwrap(),
                vec![Token::Number(NumberToken::I32(0x7fff_ffff_i32 as u32))]
            );

            assert_eq!(
                lex_from_str_without_location("0xffff_ffff_u32").unwrap(),
                vec![Token::Number(NumberToken::U32(0xffff_ffff_u32))]
            );

            // err: number width overflow
            assert!(matches!(
                lex_from_str("0x1_ffff_ffff_u32"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 16,
                            line: 0,
                            column: 16
                        }
                    }
                ))
            ));
        }

        // i64/u64
        {
            assert_eq!(
                lex_from_str_without_location("0x7fff_ffff_ffff_ffff_i64").unwrap(),
                vec![Token::Number(NumberToken::I64(
                    0x7fff_ffff_ffff_ffff_i64 as u64
                ))]
            );

            assert_eq!(
                lex_from_str_without_location("0xffff_ffff_ffff_ffff_u64").unwrap(),
                vec![Token::Number(NumberToken::U64(0xffff_ffff_ffff_ffff_u64))]
            );

            // err: number width overflow
            assert!(matches!(
                lex_from_str("0x1_ffff_ffff_ffff_ffff_u64"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 26,
                            line: 0,
                            column: 26
                        }
                    }
                ))
            ));
        }

        // integer with string "f32" suffix
        assert_eq!(
            lex_from_str_without_location("0xaa_f32").unwrap(),
            vec![Token::Number(NumberToken::I32(0xaaf32))]
        );

        // integer with string "f64" suffix
        assert_eq!(
            lex_from_str_without_location("0xbb_f64").unwrap(),
            vec![Token::Number(NumberToken::I32(0xbbf64))]
        );
    }

    #[test]
    fn test_lex_hexadecimal_floating_point_number() {
        // default type is f64
        assert_eq!(
            lex_from_str_without_location("0x1.4p3").unwrap(),
            vec![Token::Number(NumberToken::F64(10f64))]
        );

        // 3.1415927f32
        assert_eq!(
            lex_from_str_without_location("0x1.921fb6p1f32").unwrap(),
            vec![Token::Number(NumberToken::F32(std::f32::consts::PI))]
        );

        // 2.718281828459045f64
        assert_eq!(
            lex_from_str_without_location("0x1.5bf0a8b145769p+1_f64").unwrap(),
            vec![Token::Number(NumberToken::F64(std::f64::consts::E))]
        );

        // https://observablehq.com/@jrus/hexfloat
        assert_eq!(
            lex_from_str_without_location("0x1.62e42fefa39efp-1_f64").unwrap(),
            vec![Token::Number(NumberToken::F64(std::f64::consts::LN_2))]
        );

        // Testing the ranges

        assert_eq!(
            lex_from_str("0x1.4p3").unwrap(),
            vec![TokenWithRange::new(
                Token::Number(NumberToken::F64(10f64)),
                Range::from_position_and_length(&Position::new(0, 0, 0), 7)
            )]
        );

        // err: tails with '.'
        assert!(matches!(
            lex_from_str("0x1."),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0
                    },
                    end_included: Position {
                        index: 3,
                        line: 0,
                        column: 3
                    }
                }
            ))
        ));

        // err: tails with 'p'
        assert!(matches!(
            lex_from_str("0x1p"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0
                    },
                    end_included: Position {
                        index: 3,
                        line: 0,
                        column: 3
                    }
                }
            ))
        ));

        // err: missing the exponent
        assert!(matches!(
            lex_from_str("0x1.23"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0
                    },
                    end_included: Position {
                        index: 5,
                        line: 0,
                        column: 5
                    }
                }
            ))
        ));

        // err: multiple '.' (point)
        assert!(matches!(
            lex_from_str("0x1.2.3"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 5,
                    line: 0,
                    column: 5,
                }
            ))
        ));

        // err: multiple 'p' (exponent)
        assert!(matches!(
            lex_from_str("0x1.2p3p4"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 7,
                    line: 0,
                    column: 7,
                }
            ))
        ));

        // err: incorrect type (invalid dot '.' after 'p')
        assert!(matches!(
            lex_from_str("0x1.23p4.5"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 8,
                    line: 0,
                    column: 8,
                }
            ))
        ));

        // err: incorrect type (invalid char 'i' after 'p')
        assert!(matches!(
            lex_from_str("0x1.23p4_i32"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 9,
                    line: 0,
                    column: 9,
                }
            ))
        ));
    }

    #[test]
    fn test_lex_binary_number() {
        assert_eq!(
            lex_from_str_without_location("0b1100").unwrap(),
            vec![Token::Number(NumberToken::I32(0b1100))]
        );

        assert_eq!(
            lex_from_str_without_location("-0b1010").unwrap(),
            vec![Token::_Minus, Token::Number(NumberToken::I32(0b1010))]
        );

        assert_eq!(
            lex_from_str_without_location("+0b0101").unwrap(),
            vec![Token::_Plus, Token::Number(NumberToken::I32(0b0101))]
        );

        // Testing the ranges

        assert_eq!(
            lex_from_str("0b10 0b0101").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(0b10)),
                    Range::from_position_and_length(&Position::new(0, 0, 0,), 4)
                ),
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(0b0101)),
                    Range::from_position_and_length(&Position::new(5, 0, 5,), 6)
                ),
            ]
        );

        // err: does not support binary floating point
        assert!(matches!(
            lex_from_str("0b11.10"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 4,
                    line: 0,
                    column: 4,
                }
            ))
        ));

        // err: number width overflow
        assert!(matches!(
            lex_from_str("0b1_0000_0000_0000_0000_0000_0000_0000_0000"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0
                    },
                    end_included: Position {
                        index: 42,
                        line: 0,
                        column: 42
                    }
                }
            ))
        ));

        // err: invalid digit for binary number
        assert!(matches!(
            lex_from_str("0b1012"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 5,
                    line: 0,
                    column: 5,
                }
            ))
        ));

        // err: empty binary number
        assert!(matches!(
            lex_from_str("0b"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0
                    },
                    end_included: Position {
                        index: 1,
                        line: 0,
                        column: 1
                    }
                }
            ))
        ));
    }

    #[test]
    fn test_lex_binary_number_with_explicit_type() {
        // general
        {
            assert_eq!(
                lex_from_str_without_location("0b11i8").unwrap(),
                vec![Token::Number(NumberToken::I8(0b11))]
            );

            assert_eq!(
                lex_from_str_without_location("0b11_i8").unwrap(),
                vec![Token::Number(NumberToken::I8(0b11))]
            );

            assert_eq!(
                lex_from_str_without_location("0b11__i8").unwrap(),
                vec![Token::Number(NumberToken::I8(0b11))]
            );

            // Testing the ranges

            // "0b101_i16 0b1010_u32"
            //  01234567890123456789  // index
            assert_eq!(
                lex_from_str("0b101_i16 0b1010_u32").unwrap(),
                vec![
                    TokenWithRange::new(
                        Token::Number(NumberToken::I16(0b101)),
                        Range::from_position_and_length(&Position::new(0, 0, 0), 9)
                    ),
                    TokenWithRange::new(
                        Token::Number(NumberToken::U32(0b1010)),
                        Range::from_position_and_length(&Position::new(10, 0, 10), 10)
                    ),
                ]
            );
        }

        // i8/u8
        {
            assert_eq!(
                lex_from_str_without_location("0b0111_1111_i8").unwrap(),
                vec![Token::Number(NumberToken::I8(0x7f_i8 as u8))]
            );

            assert_eq!(
                lex_from_str_without_location("0b1111_1111_u8").unwrap(),
                vec![Token::Number(NumberToken::U8(0xff_u8))]
            );

            // err: number width overflow
            assert!(matches!(
                lex_from_str("0b1_1111_1111_u8"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 15,
                            line: 0,
                            column: 15
                        }
                    }
                ))
            ));
        }

        // i16/u16
        {
            assert_eq!(
                lex_from_str_without_location("0b0111_1111_1111_1111_i16").unwrap(),
                vec![Token::Number(NumberToken::I16(0x7fff_i16 as u16))]
            );

            assert_eq!(
                lex_from_str_without_location("0b1111_1111_1111_1111_u16").unwrap(),
                vec![Token::Number(NumberToken::U16(0xffff_u16))]
            );

            // err: number width overflow
            assert!(matches!(
                lex_from_str("0b1_1111_1111_1111_1111_u16"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 26,
                            line: 0,
                            column: 26
                        }
                    }
                ))
            ));
        }

        // i32/u32
        {
            assert_eq!(
                lex_from_str_without_location("0b0111_1111_1111_1111__1111_1111_1111_1111_i32")
                    .unwrap(),
                vec![Token::Number(NumberToken::I32(0x7fff_ffff_i32 as u32))]
            );

            assert_eq!(
                lex_from_str_without_location("0b1111_1111_1111_1111__1111_1111_1111_1111_u32")
                    .unwrap(),
                vec![Token::Number(NumberToken::U32(0xffff_ffff_u32))]
            );

            // err: number width overflow
            assert!(matches!(
                lex_from_str("0b1_1111_1111_1111_1111__1111_1111_1111_1111_u32"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 47,
                            line: 0,
                            column: 47
                        }
                    }
                ))
            ));
        }

        // i64/u64
        {
            assert_eq!(
                lex_from_str_without_location("0b0111_1111_1111_1111__1111_1111_1111_1111__1111_1111_1111_1111__1111_1111_1111_1111_i64").unwrap(),
                vec![Token::Number(NumberToken::I64(0x7fff_ffff_ffff_ffff_i64 as u64))]
            );

            assert_eq!(
                lex_from_str_without_location("0b1111_1111_1111_1111__1111_1111_1111_1111__1111_1111_1111_1111__1111_1111_1111_1111_u64").unwrap(),
                vec![Token::Number(NumberToken::U64(0xffff_ffff_ffff_ffff_u64))]
            );

            // err: number width overflow
            assert!(matches!(
                lex_from_str(
                    "0b1_1111_1111_1111_1111__1111_1111_1111_1111__1111_1111_1111_1111__1111_1111_1111_1111_u64"
                ),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 89,
                            line: 0,
                            column: 89
                        }
                    }
                ))
            ));
        }

        // err: does not support binary floating pointer number.
        // error type is: invalid char 'f' for binary number
        assert!(matches!(
            lex_from_str("0b11_f32"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 5,
                    line: 0,
                    column: 5,
                }
            ))
        ));
    }

    #[test]
    fn test_lex_octal_number() {
        assert_eq!(
            lex_from_str_without_location("0o123").unwrap(),
            vec![Token::Number(NumberToken::I32(0o123))]
        );

        assert_eq!(
            lex_from_str_without_location("-0o644").unwrap(),
            vec![Token::_Minus, Token::Number(NumberToken::I32(0o644))]
        );

        assert_eq!(
            lex_from_str_without_location("+0o777").unwrap(),
            vec![Token::_Plus, Token::Number(NumberToken::I32(0o777))]
        );

        // Testing the ranges

        assert_eq!(
            lex_from_str("0o11 0o0755").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(0o11)),
                    Range::from_position_and_length(&Position::new(0, 0, 0,), 4)
                ),
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(0o755)),
                    Range::from_position_and_length(&Position::new(5, 0, 5,), 6)
                ),
            ]
        );

        // err: does not support octal floating point
        assert!(matches!(
            lex_from_str("0o11.10"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 4,
                    line: 0,
                    column: 4,
                }
            ))
        ));

        // err: number width overflow
        // u32 max is 0o377_7777_7777 (10 groups of `0b111` and one group of `0b11`)
        assert!(matches!(
            lex_from_str("0o400_0000_0000"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0
                    },
                    end_included: Position {
                        index: 14,
                        line: 0,
                        column: 14
                    }
                }
            ))
        ));

        // err: invalid digit for binary number
        assert!(matches!(
            lex_from_str("0o1018"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 5,
                    line: 0,
                    column: 5,
                }
            ))
        ));

        // err: empty binary number
        assert!(matches!(
            lex_from_str("0o"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0
                    },
                    end_included: Position {
                        index: 1,
                        line: 0,
                        column: 1
                    }
                }
            ))
        ));
    }

    #[test]
    fn test_lex_octal_number_with_explicit_type() {
        // general
        {
            assert_eq!(
                lex_from_str_without_location("0o11i8").unwrap(),
                vec![Token::Number(NumberToken::I8(0o11))]
            );

            assert_eq!(
                lex_from_str_without_location("0o11_i8").unwrap(),
                vec![Token::Number(NumberToken::I8(0o11))]
            );

            assert_eq!(
                lex_from_str_without_location("0o11__i8").unwrap(),
                vec![Token::Number(NumberToken::I8(0o11))]
            );

            // Testing the ranges

            // "0o600_i16 0o0444_u32"
            //  01234567890123456789  // index
            assert_eq!(
                lex_from_str("0o600_i16 0o0444_u32").unwrap(),
                vec![
                    TokenWithRange::new(
                        Token::Number(NumberToken::I16(0o600)),
                        Range::from_position_and_length(&Position::new(0, 0, 0), 9)
                    ),
                    TokenWithRange::new(
                        Token::Number(NumberToken::U32(0o444)),
                        Range::from_position_and_length(&Position::new(10, 0, 10), 10)
                    ),
                ]
            );
        }

        // i8/u8
        {
            assert_eq!(
                lex_from_str_without_location("0o66_i8").unwrap(),
                vec![Token::Number(NumberToken::I8(0o66i8 as u8))]
            );

            assert_eq!(
                lex_from_str_without_location("0o77_u8").unwrap(),
                vec![Token::Number(NumberToken::U8(0o77_u8))]
            );

            // err: number width overflow
            // u8 max is 0o377 (2 groups of `0b111` and one group of `0b11`)
            assert!(matches!(
                lex_from_str("0o400_u8"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 7,
                            line: 0,
                            column: 7
                        }
                    }
                ))
            ));
        }

        // i16/u16
        {
            assert_eq!(
                lex_from_str_without_location("0o1234i16").unwrap(),
                vec![Token::Number(NumberToken::I16(0o1234i16 as u16))]
            );

            assert_eq!(
                lex_from_str_without_location("0o7777_u16").unwrap(),
                vec![Token::Number(NumberToken::U16(0o7777_u16))]
            );

            // err: number width overflow
            // u16 max is 0o177_777 (5 groups of `0b111` and one group of `0b1`)
            assert!(matches!(
                lex_from_str("0o200000_u16"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 11,
                            line: 0,
                            column: 11
                        }
                    }
                ))
            ));
        }

        // i32/u32
        {
            assert_eq!(
                lex_from_str_without_location("0o1234_567i32").unwrap(),
                vec![Token::Number(NumberToken::I32(0o1_234_567_i32 as u32))]
            );

            assert_eq!(
                lex_from_str_without_location("0o7777_777_u32").unwrap(),
                vec![Token::Number(NumberToken::U32(0o7_777_777_u32))]
            );

            // err: number width overflow
            // u32 max is 0o3_7777_7777 (10 groups of `0b111` and one group of `0b11`)
            assert!(matches!(
                lex_from_str("0o400_0000_0000_u32"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 18,
                            line: 0,
                            column: 18
                        }
                    }
                ))
            ));
        }

        // i64/u64
        {
            assert_eq!(
                lex_from_str_without_location("0o555_777i64").unwrap(),
                vec![Token::Number(NumberToken::I64(0o555_777i64 as u64))]
            );

            assert_eq!(
                lex_from_str_without_location("0o644_755_u64").unwrap(),
                vec![Token::Number(NumberToken::U64(0o644_755_u64))]
            );

            // err: number width overflow
            // u64 max is 0o17_7777_7777_7777_7777_7777 (21 groups of `0b111` and one group of `0b1`)
            assert!(matches!(
                lex_from_str("0o20_0000_0000_0000_0000_0000_u64"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0
                        },
                        end_included: Position {
                            index: 32,
                            line: 0,
                            column: 32
                        }
                    }
                ))
            ));
        }

        // err: does not support binary floating pointer number.
        // error type is: invalid char 'f' for binary number
        assert!(matches!(
            lex_from_str("0o77_f32"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 5,
                    line: 0,
                    column: 5,
                }
            ))
        ));
    }

    #[test]
    fn test_lex_char() {
        assert_eq!(
            lex_from_str_without_location("'a'").unwrap(),
            vec![Token::Char('a')]
        );

        assert_eq!(
            lex_from_str_without_location("('a')").unwrap(),
            vec![
                Token::OpeningParenthesis,
                Token::Char('a'),
                Token::ClosingParenthesis
            ]
        );

        assert_eq!(
            lex_from_str_without_location("'a' 'z'").unwrap(),
            vec![Token::Char('a'), Token::Char('z')]
        );

        // CJK
        assert_eq!(
            lex_from_str_without_location("'文'").unwrap(),
            vec![Token::Char('文')]
        );

        // emoji
        assert_eq!(
            lex_from_str_without_location("'😊'").unwrap(),
            vec![Token::Char('😊')]
        );

        // escape char `\\`
        assert_eq!(
            lex_from_str_without_location("'\\\\'").unwrap(),
            vec![Token::Char('\\')]
        );

        // escape char `\'`
        assert_eq!(
            lex_from_str_without_location("'\\\''").unwrap(),
            vec![Token::Char('\'')]
        );

        // escape char `"`
        assert_eq!(
            lex_from_str_without_location("'\\\"'").unwrap(),
            vec![Token::Char('"')]
        );

        // escape char `\t`
        assert_eq!(
            lex_from_str_without_location("'\\t'").unwrap(),
            vec![Token::Char('\t')]
        );

        // escape char `\r`
        assert_eq!(
            lex_from_str_without_location("'\\r'").unwrap(),
            vec![Token::Char('\r')]
        );

        // escape char `\n`
        assert_eq!(
            lex_from_str_without_location("'\\n'").unwrap(),
            vec![Token::Char('\n')]
        );

        // escape char `\0`
        assert_eq!(
            lex_from_str_without_location("'\\0'").unwrap(),
            vec![Token::Char('\0')]
        );

        // escape char, unicode
        assert_eq!(
            lex_from_str_without_location("'\\u{2d}'").unwrap(),
            vec![Token::Char('-')]
        );

        // escape char, unicode
        assert_eq!(
            lex_from_str_without_location("'\\u{6587}'").unwrap(),
            vec![Token::Char('文')]
        );

        // Testing the ranges

        assert_eq!(
            lex_from_str("'a' '文'").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::Char('a'),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 3)
                ),
                TokenWithRange::new(
                    Token::Char('文'),
                    Range::from_position_and_length(&Position::new(4, 0, 4), 3)
                )
            ]
        );

        assert_eq!(
            lex_from_str("'\\t'").unwrap(),
            vec![TokenWithRange::new(
                Token::Char('\t'),
                Range::from_position_and_length(&Position::new(0, 0, 0), 4)
            )]
        );

        assert_eq!(
            lex_from_str("'\\u{6587}'").unwrap(),
            vec![TokenWithRange::new(
                Token::Char('文'),
                Range::from_position_and_length(&Position::new(0, 0, 0), 10)
            )]
        );

        // err: empty char
        assert!(matches!(
            lex_from_str("''"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0
                    },
                    end_included: Position {
                        index: 1,
                        line: 0,
                        column: 1
                    }
                }
            ))
        ));

        // err: empty char, missing the char
        assert!(matches!(
            lex_from_str("'"),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: incomplete char, missing the right quote, encounter EOF
        assert!(matches!(
            lex_from_str("'a"),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: invalid char, expect the right quote, encounter another char
        assert!(matches!(
            lex_from_str("'ab"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 2,
                    line: 0,
                    column: 2,
                }
            ))
        ));

        // err: invalid char, expect the right quote, encounter another char
        assert!(matches!(
            lex_from_str("'ab'"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 2,
                    line: 0,
                    column: 2,
                }
            ))
        ));

        // err: unsupported escape char \v
        assert!(matches!(
            lex_from_str(r#"'\v'"#),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 2,
                    line: 0,
                    column: 2
                }
            ))
        ));

        // err: unsupported hex escape "\x.."
        assert!(matches!(
            lex_from_str(r#"'\x33'"#),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 2,
                    line: 0,
                    column: 2
                }
            ))
        ));

        // err: empty unicode escape string
        // "'\\u{}'"
        //  01 2345     // index
        assert!(matches!(
            lex_from_str("'\\u{}'"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 3,
                        line: 0,
                        column: 3
                    },
                    end_included: Position {
                        index: 4,
                        line: 0,
                        column: 4
                    }
                }
            ))
        ));

        // err: invalid unicode code point, digits too much
        // "'\\u{1000111}'"
        //  01 23456789012      // index
        assert!(matches!(
            lex_from_str("'\\u{1000111}'"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 3,
                        line: 0,
                        column: 3
                    },
                    end_included: Position {
                        index: 10,
                        line: 0,
                        column: 10
                    }
                }
            ))
        ));

        // err: invalid unicode code point, code point out of range
        // "'\\u{123456}'"
        //  01 2345678901   // index
        assert!(matches!(
            lex_from_str("'\\u{123456}'"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 3,
                        line: 0,
                        column: 3
                    },
                    end_included: Position {
                        index: 10,
                        line: 0,
                        column: 10
                    }
                }
            ))
        ));

        // err: invalid char in the unicode escape sequence
        assert!(matches!(
            lex_from_str("'\\u{12mn}''"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 6,
                    line: 0,
                    column: 6,
                }
            ))
        ));

        // err: missing the closing brace for unicode escape sequence
        assert!(matches!(
            lex_from_str("'\\u{1234'"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 8,
                    line: 0,
                    column: 8,
                }
            ))
        ));

        // err: incomplete unicode escape sequence, encounter EOF
        assert!(matches!(
            lex_from_str("'\\u{1234"),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: missing left brace for unicode escape sequence
        assert!(matches!(
            lex_from_str("'\\u1234}'"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 2,
                    line: 0,
                    column: 2,
                }
            ))
        ));
    }

    #[test]
    fn test_lex_string() {
        assert_eq!(
            lex_from_str_without_location(r#""abc""#).unwrap(),
            vec![Token::new_string("abc")]
        );

        assert_eq!(
            lex_from_str_without_location(r#"("abc")"#).unwrap(),
            vec![
                Token::OpeningParenthesis,
                Token::new_string("abc"),
                Token::ClosingParenthesis,
            ]
        );

        assert_eq!(
            lex_from_str_without_location(r#""abc" "xyz""#).unwrap(),
            vec![Token::new_string("abc"), Token::new_string("xyz")]
        );

        assert_eq!(
            lex_from_str_without_location("\"abc\"\n\n\"xyz\"").unwrap(),
            vec![Token::new_string("abc"), Token::new_string("xyz"),]
        );

        // unicode
        assert_eq!(
            lex_from_str_without_location(
                r#"
                "abc文字😊"
                "#
            )
            .unwrap(),
            vec![Token::new_string("abc文字😊"),]
        );

        // empty string
        assert_eq!(
            lex_from_str_without_location("\"\"").unwrap(),
            vec![Token::new_string("")]
        );

        // escape chars
        assert_eq!(
            lex_from_str_without_location(
                r#"
                "\\\'\"\t\r\n\0\u{2d}\u{6587}"
                "#
            )
            .unwrap(),
            vec![Token::new_string("\\\'\"\t\r\n\0-文"),]
        );

        // Testing the ranges
        // "abc" "文字😊"
        // 01234567 8 9 0

        assert_eq!(
            lex_from_str(r#""abc" "文字😊""#).unwrap(),
            vec![
                TokenWithRange::new(
                    Token::new_string("abc"),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 5)
                ),
                TokenWithRange::new(
                    Token::new_string("文字😊"),
                    Range::from_position_and_length(&Position::new(6, 0, 6), 5)
                ),
            ]
        );

        // err: incomplete string, missing the closed quote
        assert!(matches!(
            lex_from_str("\"abc"),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: incomplete string, missing the closed quote, ends with \n
        assert!(matches!(
            lex_from_str("\"abc\n"),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: incomplete string, missing the closed quote, ends with whitespaces/other chars
        assert!(matches!(
            lex_from_str("\"abc\n   "),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: unsupported escape char \v
        assert!(matches!(
            lex_from_str(r#""abc\vxyz""#),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 5,
                    line: 0,
                    column: 5,
                }
            ))
        ));

        // err: unsupported hex escape "\x.."
        assert!(matches!(
            lex_from_str(r#""abc\x33xyz""#),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 5,
                    line: 0,
                    column: 5,
                }
            ))
        ));

        // err: empty unicode escape string
        // "abc\u{}"
        // 012345678    // index
        assert!(matches!(
            lex_from_str(r#""abc\u{}xyz""#),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 6,
                        line: 0,
                        column: 6
                    },
                    end_included: Position {
                        index: 7,
                        line: 0,
                        column: 7
                    }
                }
            ))
        ));

        // err: invalid unicode code point, too much digits
        // "abc\u{1000111}xyz"
        // 0123456789023456789  // index
        assert!(matches!(
            lex_from_str(r#""abc\u{1000111}xyz""#),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 6,
                        line: 0,
                        column: 6
                    },
                    end_included: Position {
                        index: 13,
                        line: 0,
                        column: 13
                    }
                }
            ))
        ));

        // err: invalid unicode code point, code point out of range
        // "abc\u{123456}xyz"
        // 012345678901234567   // index
        assert!(matches!(
            lex_from_str(r#""abc\u{123456}xyz""#),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 6,
                        line: 0,
                        column: 6
                    },
                    end_included: Position {
                        index: 13,
                        line: 0,
                        column: 13
                    }
                }
            ))
        ));

        // err: invalid char in the unicode escape sequence
        assert!(matches!(
            lex_from_str(r#""abc\u{12mn}xyz""#),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 9,
                    line: 0,
                    column: 9,
                }
            ))
        ));

        // err: missing the closing brace for unicode escape sequence
        assert!(matches!(
            lex_from_str(r#""abc\u{1234""#),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 11,
                    line: 0,
                    column: 11,
                }
            ))
        ));

        // err: incomplete unicode escape sequence, encounter EOF
        assert!(matches!(
            lex_from_str(r#""abc\u{1234"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: missing left brace for unicode escape sequence
        assert!(matches!(
            lex_from_str(r#""abc\u1234}xyz""#),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 5,
                    line: 0,
                    column: 5,
                }
            ))
        ));
    }

    #[test]
    fn test_lex_multiple_line_string() {
        assert_eq!(
            lex_from_str_without_location("\"abc\n    \n\n    \n\"").unwrap(),
            vec![Token::new_string("abc\n    \n\n    \n")]
        );

        assert_eq!(
            lex_from_str_without_location("\"abc\ndef\n    uvw\r\n\t  \txyz\"").unwrap(),
            vec![Token::new_string("abc\ndef\n    uvw\r\n\t  \txyz")]
        );

        // Testing the ranges

        assert_eq!(
            lex_from_str("\"abc\n    xyz\n\" \"foo\nbar\"").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::new_string("abc\n    xyz\n"),
                    Range::new(&Position::new(0, 0, 0), &Position::new(13, 2, 0))
                ),
                TokenWithRange::new(
                    Token::new_string("foo\nbar"),
                    Range::new(&Position::new(15, 2, 2), &Position::new(23, 3, 3))
                )
            ]
        );

        // err: incomplete string, missing the closed quote, ends with \n
        assert!(matches!(
            lex_from_str("\"abc\n    \n\n    \n"),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: incomplete string, missing the closed quote, whitespaces/other chars
        assert!(matches!(
            lex_from_str("\"abc\n    \n\n    \n   "),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));
    }

    #[test]
    fn test_lex_concatenate_string() {
        // the tailing '\' should escapes the new-line chars
        assert_eq!(
            lex_from_str_without_location("\"abc\\\ndef\\\n    opq\\\r\n\t  \txyz\"").unwrap(),
            vec![Token::new_string("abcdefopqxyz")]
        );

        // the tailing '\' should escapes the new-line chars and trim the leading white-spaces
        assert_eq!(
            lex_from_str_without_location("\"\\\n  \t  \"").unwrap(),
            vec![Token::new_string("")]
        );

        // Testing the ranges

        assert_eq!(
            lex_from_str("\"abc\\\n\\\n    xyz\" \"\\\n\"").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::new_string("abcxyz"),
                    Range::new(&Position::new(0, 0, 0), &Position::new(15, 2, 7))
                ),
                TokenWithRange::new(
                    Token::new_string(""),
                    Range::new(&Position::new(17, 2, 9), &Position::new(20, 3, 0))
                )
            ]
        );

        // err: incomplete string, missing the right quote, ends with \n
        assert!(matches!(
            lex_from_str("\"abc\\\n"),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: incomplete string, missing the right quote, ends with whitespaces/other chars
        assert!(matches!(
            lex_from_str("\"abc\\\n    "),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));
    }

    #[test]
    fn test_lex_raw_string() {
        assert_eq!(
            lex_from_str_without_location(
                "r\"abc\ndef\n    uvw\r\n\t escape: \\r\\n\\t\\\\ unicode: \\u{1234} xyz\""
            )
            .unwrap(),
            vec![Token::new_string(
                "abc\ndef\n    uvw\r\n\t escape: \\r\\n\\t\\\\ unicode: \\u{1234} xyz"
            )]
        );

        // Testing the ranges

        assert_eq!(
            lex_from_str("r\"abc\n    xyz\" r\"foo\\nbar\"").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::new_string("abc\n    xyz"),
                    Range::new(&Position::new(0, 0, 0), &Position::new(13, 1, 7))
                ),
                TokenWithRange::new(
                    Token::new_string("foo\\nbar"),
                    Range::new(&Position::new(15, 1, 9), &Position::new(25, 1, 19))
                )
            ]
        );

        // err: incomplete string, missing the right quote
        assert!(matches!(
            lex_from_str("r\"abc    "),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: incomplete string, missing the right quote, ends with \n
        assert!(matches!(
            lex_from_str("r\"abc\n"),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: incomplete string, missing the right quote, ends with whitespaces/other chars
        assert!(matches!(
            lex_from_str("r\"abc\n   "),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));
    }

    #[test]
    fn test_lex_raw_string_with_hash_symbol() {
        assert_eq!(
            lex_from_str_without_location(
                "r#\"abc\ndef\n    uvw\r\n\t escape: \\r\\n\\t\\\\ unicode: \\u{1234} xyz quote: \"foo\"\"#"
            ).unwrap(),
            vec![Token::new_string(
                "abc\ndef\n    uvw\r\n\t escape: \\r\\n\\t\\\\ unicode: \\u{1234} xyz quote: \"foo\""
            )]
        );

        // Testing the ranges

        assert_eq!(
            lex_from_str("r#\"abc\n    xyz\"# r#\"foo\\nbar\"#").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::new_string("abc\n    xyz"),
                    Range::new(&Position::new(0, 0, 0), &Position::new(15, 1, 8))
                ),
                TokenWithRange::new(
                    Token::new_string("foo\\nbar"),
                    Range::new(&Position::new(17, 1, 10), &Position::new(29, 1, 22))
                )
            ]
        );

        // err: incomplete string, missing the closed hash
        assert!(matches!(
            lex_from_str("r#\"abc    \""),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: incomplete string, missing the closed quote, ends with \n
        assert!(matches!(
            lex_from_str("r#\"abc\n"),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: incomplete string, missing the closed quote, ends with whitespace/other chars
        assert!(matches!(
            lex_from_str("r#\"abc\nxyz"),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));
    }

    #[test]
    fn test_lex_auto_trimmed_string() {
        assert_eq!(
            lex_from_str_without_location(
                r#"
                """
                one
                  two
                    three
                end
                """
                "#
            )
            .unwrap(),
            vec![Token::new_string("one\n  two\n    three\nend"),]
        );

        assert_eq!(
            lex_from_str_without_location(
                r#"
                """
                one
              two
            three
                end
                """
                "#
            )
            .unwrap(),
            vec![Token::new_string("    one\n  two\nthree\n    end"),]
        );

        // contains empty lines
        assert_eq!(
            lex_from_str_without_location(
                r#"
                """
                    one\\\"\t\r\n\u{1234}

                    end
                """
                "#
            )
            .unwrap(),
            vec![Token::new_string("one\\\\\\\"\\t\\r\\n\\u{1234}\n\nend"),]
        );

        // including (""")
        assert_eq!(
            lex_from_str_without_location(
                r#"
                """
                    one"""
                    two
                """
                "#
            )
            .unwrap(),
            vec![Token::new_string("one\"\"\"\ntwo"),]
        );

        // inline
        assert_eq!(
            lex_from_str_without_location(
                r#"
                11 """
                    abc
                """ 13
                "#
            )
            .unwrap(),
            vec![
                Token::Number(NumberToken::I32(11)),
                Token::new_string("abc"),
                Token::Number(NumberToken::I32(13)),
            ]
        );

        // Testing the ranges

        assert_eq!(
            lex_from_str(
                r#"["""
    foo
    bar
""", """
    hello
    world
"""]"#
            )
            .unwrap(),
            vec![
                TokenWithRange::new(
                    Token::OpeningBracket,
                    Range::new(&Position::new(0, 0, 0), &Position::new(0, 0, 0))
                ),
                TokenWithRange::new(
                    Token::new_string("foo\nbar"),
                    Range::new(&Position::new(1, 0, 1), &Position::new(23, 3, 2))
                ),
                TokenWithRange::new(
                    Token::new_string("hello\nworld"),
                    Range::new(&Position::new(26, 3, 5), &Position::new(52, 6, 2))
                ),
                TokenWithRange::new(
                    Token::ClosingBracket,
                    Range::new(&Position::new(53, 6, 3), &Position::new(53, 6, 3))
                ),
            ]
        );

        // err: the content does not start on a new line
        assert!(matches!(
            lex_from_str(
                r#"
123 """hello
"""
"#
            ),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 7,
                    line: 1,
                    column: 6,
                }
            ))
        ));

        // err: missing the ending marker (the ending marker does not start on a new line)
        assert!(matches!(
            lex_from_str(
                r#"
"""
hello"""
"#
            ),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: missing the ending marker
        assert!(matches!(
            lex_from_str(
                r#"
"""
hello
world"#
            ),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: missing the ending marker, ends with \n
        assert!(matches!(
            lex_from_str(
                r#"
"""
hello
"#
            ),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));
    }

    #[test]
    fn test_lex_hexadecimal_byte_data() {
        assert_eq!(
            lex_from_str_without_location(
                r#"
                h""
                "#
            )
            .unwrap(),
            vec![Token::HexadecimalByteData(vec![]),]
        );

        assert_eq!(
            lex_from_str_without_location(
                r#"
                h"11"
                "#
            )
            .unwrap(),
            vec![Token::HexadecimalByteData(vec![0x11]),]
        );

        assert_eq!(
            lex_from_str_without_location(
                r#"
                h"11 13 17 19"
                "#
            )
            .unwrap(),
            vec![Token::HexadecimalByteData(vec![0x11, 0x13, 0x17, 0x19]),]
        );

        assert_eq!(
            lex_from_str_without_location(
                "
                h\"  11\t  13\r17\r\n  19\n  \"
                "
            )
            .unwrap(),
            vec![Token::HexadecimalByteData(vec![0x11, 0x13, 0x17, 0x19]),]
        );

        // Testing the ranges

        assert_eq!(
            lex_from_str("h\"11 13\"").unwrap(),
            vec![TokenWithRange::new(
                Token::HexadecimalByteData(vec![0x11, 0x13]),
                Range::from_position_and_length(&Position::new(0, 0, 0), 8)
            )]
        );

        assert_eq!(
            lex_from_str("h\"11\" h\"13\"").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::HexadecimalByteData(vec![0x11]),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 5)
                ),
                TokenWithRange::new(
                    Token::HexadecimalByteData(vec![0x13]),
                    Range::from_position_and_length(&Position::new(6, 0, 6), 5)
                )
            ]
        );

        // err: not enough digits
        assert!(matches!(
            lex_from_str("h\"11 1\""),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 6,
                    line: 0,
                    column: 6,
                }
            ))
        ));

        // err: too much digits, no whitespace between two bytes
        assert!(matches!(
            lex_from_str("h\"11 1317\""),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 6,
                    line: 0,
                    column: 6,
                }
            ))
        ));

        // err: invalid char for byte string
        assert!(matches!(
            lex_from_str("h\"11 1x\""),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 6,
                    line: 0,
                    column: 6,
                }
            ))
        ));

        // err: invalid separator
        assert!(matches!(
            lex_from_str("h\"11-13\""),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 3,
                    line: 0,
                    column: 3,
                }
            ))
        ));

        // err: missing the close quote
        assert!(matches!(
            lex_from_str("h\"11 13"),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: missing the close quote, ends with \n
        assert!(matches!(
            lex_from_str("h\"11 13\n"),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: missing the close quote, ends with whitespaces/other chars
        assert!(matches!(
            lex_from_str("h\"11 13\n    "),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));
    }

    #[test]
    fn test_lex_datetime() {
        let expect_date1 = DateTime::parse_from_rfc3339("2024-03-16T00:00:00Z").unwrap();
        let expect_date2 = DateTime::parse_from_rfc3339("2024-03-16T16:30:50Z").unwrap();
        let expect_date3 = DateTime::parse_from_rfc3339("2024-03-16T16:30:50+08:00").unwrap();

        assert_eq!(
            lex_from_str_without_location("d\"2024-03-16\"").unwrap(),
            vec![Token::Date(expect_date1)]
        );

        assert_eq!(
            lex_from_str_without_location("d\"2024-03-16T16:30:50Z\"").unwrap(),
            vec![Token::Date(expect_date2)]
        );

        assert_eq!(
            lex_from_str_without_location("d\"2024-03-16T16:30:50z\"").unwrap(),
            vec![Token::Date(expect_date2)]
        );

        assert_eq!(
            lex_from_str_without_location("d\"2024-03-16T16:30:50\"").unwrap(),
            vec![Token::Date(expect_date2)]
        );

        assert_eq!(
            lex_from_str_without_location("d\"2024-03-16t16:30:50\"").unwrap(),
            vec![Token::Date(expect_date2)]
        );

        assert_eq!(
            lex_from_str_without_location("d\"2024-03-16 16:30:50\"").unwrap(),
            vec![Token::Date(expect_date2)]
        );

        assert_eq!(
            lex_from_str_without_location("d\"2024-03-16T16:30:50+08:00\"").unwrap(),
            vec![Token::Date(expect_date3)]
        );

        // Testing the ranges

        assert_eq!(
            lex_from_str("d\"2024-03-16\" d\"2024-03-16T16:30:50+08:00\"").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::Date(expect_date1),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 13)
                ),
                TokenWithRange::new(
                    Token::Date(expect_date3),
                    Range::from_position_and_length(&Position::new(14, 0, 14), 28)
                ),
            ]
        );

        // err: syntex error, should be YYYY-MM-DD HH:mm:ss
        assert!(matches!(
            lex_from_str("d\"2024-3-16 4:30:50\""),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0
                    },
                    end_included: Position {
                        index: 19,
                        line: 0,
                        column: 19
                    }
                }
            ))
        ));

        // err: missing date part
        assert!(matches!(
            lex_from_str("d\"16:30:50\""),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0
                    },
                    end_included: Position {
                        index: 10,
                        line: 0,
                        column: 10
                    }
                }
            ))
        ));

        // err: invalid char
        assert!(matches!(
            lex_from_str("d\"Aug 8, 2024\""),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 2,
                    line: 0,
                    column: 2,
                }
            ))
        ));

        // err: incomplete date string
        assert!(matches!(
            lex_from_str("d\"2024-08-08"),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));
    }

    #[test]
    fn test_lex_variant() {
        assert_eq!(
            lex_from_str_without_location("Option::None").unwrap(),
            vec![Token::new_variant("Option", "None")]
        );

        assert_eq!(
            lex_from_str_without_location("Option::Some(123)").unwrap(),
            vec![
                Token::new_variant("Option", "Some"),
                Token::OpeningParenthesis,
                Token::Number(NumberToken::I32(123)),
                Token::ClosingParenthesis,
            ]
        );

        assert_eq!(
            lex_from_str_without_location("value: Result::Ok(456)").unwrap(),
            vec![
                Token::new_identifier("value"),
                Token::Colon,
                Token::new_variant("Result", "Ok"),
                Token::OpeningParenthesis,
                Token::Number(NumberToken::I32(456)),
                Token::ClosingParenthesis,
            ]
        );
    }

    #[test]
    fn test_lex_line_comment() {
        assert_eq!(
            lex_from_str_without_location(
                r#"
                7 //11
                13 17// 19 23
                //  29
                31//    37
                "#
            )
            .unwrap(),
            vec![
                Token::Number(NumberToken::I32(7)),
                Token::Number(NumberToken::I32(13)),
                Token::Number(NumberToken::I32(17)),
                Token::Number(NumberToken::I32(31)),
            ]
        );

        // Testing the ranges

        assert_eq!(
            lex_from_str("foo // bar").unwrap(),
            vec![TokenWithRange::new(
                Token::Identifier("foo".to_owned()),
                Range::from_position_and_length(&Position::new(0, 0, 0), 3)
            ),]
        );

        assert_eq!(
            lex_from_str("abc // def\n// xyz\n").unwrap(),
            vec![TokenWithRange::new(
                Token::Identifier("abc".to_owned()),
                Range::from_position_and_length(&Position::new(0, 0, 0), 3)
            ),]
        );
    }

    #[test]
    fn test_lex_block_comment() {
        assert_eq!(
            lex_from_str_without_location(
                r#"
                7 /* 11 13 */ 17
                "#
            )
            .unwrap(),
            vec![
                Token::Number(NumberToken::I32(7)),
                Token::Number(NumberToken::I32(17)),
            ]
        );

        // nested block comment
        assert_eq!(
            lex_from_str_without_location(
                r#"
                7 /* 11 /* 13 */ 17 */ 19
                "#
            )
            .unwrap(),
            vec![
                Token::Number(NumberToken::I32(7)),
                Token::Number(NumberToken::I32(19)),
            ]
        );

        // line comment chars "//" within the block comment
        assert_eq!(
            lex_from_str_without_location(
                r#"
                7 /* 11 // 13 17 */ 19
                "#
            )
            .unwrap(),
            vec![
                Token::Number(NumberToken::I32(7)),
                Token::Number(NumberToken::I32(19)),
            ]
        );

        // Testing the ranges

        assert_eq!(
            lex_from_str("foo /* hello */ bar").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::Identifier("foo".to_owned()),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 3)
                ),
                TokenWithRange::new(
                    Token::Identifier("bar".to_owned()),
                    Range::from_position_and_length(&Position::new(16, 0, 16), 3)
                ),
            ]
        );

        assert_eq!(lex_from_str("/* abc\nxyz */ /* hello */").unwrap(), vec![]);

        // err: incomplete, missing "*/"
        assert!(matches!(
            lex_from_str("7 /* 11"),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: incomplete, missing "*/", ends with \n
        assert!(matches!(
            lex_from_str("7 /* 11\n"),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: incomplete, unpaired, missing "*/"
        assert!(matches!(
            lex_from_str("a /* b /* c */"),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: incomplete, unpaired, missing "*/", ends with \n
        assert!(matches!(
            lex_from_str("a /* b /* c */\n"),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));
    }

    #[test]
    fn test_lex_combined_line_comments_and_block_comments() {
        assert_eq!(
            lex_from_str_without_location(
                r#"11 // line comment 1
                // line comment 2
                13 /* block comment 1 */
                /*
                block comment 2
                */
                17
                "#
            )
            .unwrap(),
            vec![
                Token::Number(NumberToken::I32(11)),
                Token::Number(NumberToken::I32(13)),
                Token::Number(NumberToken::I32(17)),
            ]
        );

        assert_eq!(
            lex_from_str(r#"11 /* foo */ 13"#).unwrap(),
            vec![
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(11)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 2)
                ),
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(13)),
                    Range::from_position_and_length(&Position::new(13, 0, 13), 2)
                ),
            ]
        );
    }

    #[test]
    fn test_lex_combined_comments_and_whitespaces() {
        assert_eq!(
            lex_from_str_without_location(
                r#"
                    [1,2,

                    3

                    ,4

                    ,

                    5
                    ,
                    // comment between commas
                    ,
                    6

                    // comment between blank lines

                    7
                    8
                    ]

                    "#
            )
            .unwrap(),
            vec![
                Token::OpeningBracket,
                Token::Number(NumberToken::I32(1)),
                Token::Number(NumberToken::I32(2)),
                Token::Number(NumberToken::I32(3)),
                Token::Number(NumberToken::I32(4)),
                Token::Number(NumberToken::I32(5)),
                Token::Number(NumberToken::I32(6)),
                Token::Number(NumberToken::I32(7)),
                Token::Number(NumberToken::I32(8)),
                Token::ClosingBracket,
            ]
        );

        // range

        // blanks -> blank
        assert_eq!(
            lex_from_str("11\n \n  \n13").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(11)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 2)
                ),
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(13)),
                    Range::from_position_and_length(&Position::new(8, 3, 0), 2)
                ),
            ]
        );

        // comma + blanks -> comma
        assert_eq!(
            lex_from_str(",\n\n\n11").unwrap(),
            vec![TokenWithRange::new(
                Token::Number(NumberToken::I32(11)),
                Range::from_position_and_length(&Position::new(4, 3, 0), 2)
            ),]
        );

        // blanks + comma -> comma
        assert_eq!(
            lex_from_str("11\n\n\n,").unwrap(),
            vec![TokenWithRange::new(
                Token::Number(NumberToken::I32(11)),
                Range::from_position_and_length(&Position::new(0, 0, 0), 2)
            ),]
        );

        // blanks + comma + blanks -> comma
        assert_eq!(
            lex_from_str("11\n\n,\n\n13").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(11)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 2)
                ),
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(13)),
                    Range::from_position_and_length(&Position::new(7, 4, 0), 2)
                ),
            ]
        );

        // comma + comment + comma -> comma + comma
        assert_eq!(lex_from_str(",//abc\n,").unwrap(), vec![]);

        // blanks + comment + blanks -> blank
        assert_eq!(
            lex_from_str("11\n\n//abc\n\n13").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(11)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 2)
                ),
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(13)),
                    Range::from_position_and_length(&Position::new(11, 4, 0), 2)
                ),
            ]
        );
    }

    #[test]
    fn test_lex_combined_tokens() {
        assert_eq!(
            lex_from_str_without_location(
                r#"
                {id: 123, name: "foo"}
                "#
            )
            .unwrap(),
            vec![
                Token::OpeningBrace,
                Token::new_identifier("id"),
                Token::Colon,
                Token::Number(NumberToken::I32(123)),
                Token::new_identifier("name"),
                Token::Colon,
                Token::new_string("foo"),
                Token::ClosingBrace,
            ]
        );

        assert_eq!(
            lex_from_str_without_location(
                r#"
                [123,456,789,]
                "#
            )
            .unwrap(),
            vec![
                Token::OpeningBracket,
                Token::Number(NumberToken::I32(123)),
                Token::Number(NumberToken::I32(456)),
                Token::Number(NumberToken::I32(789)),
                Token::ClosingBracket,
            ]
        );

        assert_eq!(
            lex_from_str_without_location(
                r#"
                (123 "foo" true) // line comment
                "#
            )
            .unwrap(),
            vec![
                Token::OpeningParenthesis,
                Token::Number(NumberToken::I32(123)),
                Token::new_string("foo"),
                Token::Boolean(true),
                Token::ClosingParenthesis,
            ]
        );

        assert_eq!(
            lex_from_str_without_location(
                r#"
                {
                    a: [1,2,3]
                    b: (false, d"2000-01-01 10:10:10")
                    c: {id: 11}
                }
                "#
            )
            .unwrap(),
            vec![
                Token::OpeningBrace, // {
                Token::new_identifier("a"),
                Token::Colon,
                Token::OpeningBracket, // [
                Token::Number(NumberToken::I32(1)),
                Token::Number(NumberToken::I32(2)),
                Token::Number(NumberToken::I32(3)),
                Token::ClosingBracket, // ]
                Token::new_identifier("b"),
                Token::Colon,
                Token::OpeningParenthesis, // (
                Token::Boolean(false),
                Token::Date(DateTime::parse_from_rfc3339("2000-01-01 10:10:10Z").unwrap()),
                Token::ClosingParenthesis, // )
                Token::new_identifier("c"),
                Token::Colon,
                Token::OpeningBrace, // {
                Token::new_identifier("id"),
                Token::Colon,
                Token::Number(NumberToken::I32(11)),
                Token::ClosingBrace, // }
                Token::ClosingBrace, // }
            ]
        );
    }
}
