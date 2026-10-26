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
    peekable_iterator::PeekableIterator,
    position::Position,
    range::Range,
    token::{NumberToken, Token, TokenWithRange},
};

pub struct Lexer<T>
where
    T: Iterator<Item = CharWithPosition>,
{
    upstream: PeekableIterator<CharWithPosition, T>,

    // The position of the last character consumed by `next_char()`.
    last_position: Position,

    // A stack of positions.
    // It stores the start positions of tokens while their characters are consumed,
    // so that the `Range` of each token can be built afterwards.
    position_stack: Vec<Position>,
}

impl<T> Lexer<T>
where
    T: Iterator<Item = CharWithPosition>,
{
    pub fn new(upstream: PeekableIterator<CharWithPosition, T>) -> Self {
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

    /// Checks whether the character at the given offset equals the expected character,
    /// without consuming any characters.
    ///
    /// Returns `false` if there is no character at the given offset.
    fn peek_char_and_equals(&self, offset: usize, expected_char: char) -> bool {
        matches!(
            self.upstream.peek(offset),
            Some(CharWithPosition { character, .. }) if character == &expected_char)
    }

    /// Pushes the current position onto the stack.
    ///
    /// The current position is the position of the next character,
    /// i.e., the value returned by `self.peek_position(0)`.
    fn push_peek_position_into_stack(&mut self) {
        let position = *self.peek_position(0).unwrap();
        self.position_stack.push(position);
    }

    /// Pops a position from the stack.
    ///
    /// It is usually called after `push_peek_position_into_stack()` to build the `Range` of a token.
    fn pop_position_from_stack(&mut self) -> Position {
        self.position_stack.pop().unwrap()
    }
}

impl<T> Iterator for Lexer<T>
where
    T: Iterator<Item = CharWithPosition>,
{
    type Item = Result<TokenWithRange, AsonError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lex()
    }
}

impl<T> Lexer<T>
where
    T: Iterator<Item = CharWithPosition>,
{
    fn lex(&mut self) -> Option<Result<TokenWithRange, AsonError>> {
        // ```diagram
        // c....
        // ^____ current char, not EOF, validated
        // ```

        let result = loop {
            let current_char = {
                let c = self.peek_char(0)?;
                *c
            };

            match current_char {
                '/' if self.peek_char_and_equals(1, '/') => {
                    // Line comment
                    self.lex_line_comment()
                }
                '/' if self.peek_char_and_equals(1, '*') => {
                    // Block comment
                    if let Err(e) = self.lex_block_comment() {
                        break Err(e);
                    }
                }
                ' ' | '\t' | '\r' | '\n' | ',' => {
                    // Whitespace (including a lone '\r') and commas only separate tokens,
                    // so they are consumed and discarded.
                    self.next_char();
                }
                ':' => {
                    self.next_char(); // consume ':'

                    // A standalone ':' is never the enumeration separator "::",
                    // because "::" is consumed as part of the enumeration token.
                    break Ok(TokenWithRange::new(
                        Token::Colon,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                '{' => {
                    self.next_char(); // consume '{'

                    break Ok(TokenWithRange::new(
                        Token::BraceOpen,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                '}' => {
                    self.next_char(); // consume '}'

                    break Ok(TokenWithRange::new(
                        Token::BraceClose,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                '[' => {
                    self.next_char(); // consume '['

                    break Ok(TokenWithRange::new(
                        Token::BracketOpen,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                ']' => {
                    self.next_char(); // consume ']'

                    break Ok(TokenWithRange::new(
                        Token::BracketClose,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                '(' => {
                    self.next_char(); // consume '('

                    break Ok(TokenWithRange::new(
                        Token::ParenthesisOpen,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                ')' => {
                    self.next_char(); // consume ')'

                    break Ok(TokenWithRange::new(
                        Token::ParenthesisClose,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                '+' => {
                    self.next_char(); // consume '+'

                    break Ok(TokenWithRange::new(
                        Token::_Plus,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                '-' => {
                    self.next_char(); // consume '-'

                    break Ok(TokenWithRange::new(
                        Token::_Minus,
                        Range::from_position_and_length(&self.last_position, 1),
                    ));
                }
                '0' if matches!(self.peek_char(1), Some('x' | 'X')) => {
                    // Hexadecimal number
                    break self.lex_hexadecimal_number();
                }
                '0' if matches!(self.peek_char(1), Some('b' | 'B')) => {
                    // Binary number
                    break self.lex_binary_or_octal_number(2);
                }
                '0' if matches!(self.peek_char(1), Some('o' | 'O')) => {
                    // Octal number
                    break self.lex_binary_or_octal_number(8);
                }
                '0'..='9' => {
                    // Decimal number
                    break self.lex_decimal_number();
                }
                'h' if self.peek_char_and_equals(1, '"') => {
                    // Hexadecimal byte data
                    break self.lex_hexadecimal_byte_data();
                }
                'd' if self.peek_char_and_equals(1, '"') => {
                    // DateTime
                    break self.lex_datetime();
                }
                'r' if self.peek_char_and_equals(1, '"') || self.peek_char_and_equals(1, '#') => {
                    // Raw string
                    break self.lex_raw_string();
                }
                '"' => {
                    // String
                    if self.peek_char_and_equals(1, '"') && self.peek_char_and_equals(2, '"') {
                        // Auto-trimmed string
                        break self.lex_auto_trimmed_string();
                    } else {
                        // Regular string
                        break self.lex_string();
                    }
                }
                '\'' => {
                    // Char
                    break self.lex_char();
                }
                'a'..='z' | 'A'..='Z' | '_' | '\u{a0}'..='\u{d7ff}' | '\u{e000}'..='\u{10ffff}' => {
                    // Identifier
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

        // Whether "::" has been found.
        // "::" separates the type name and the variant name of an enumeration.
        let mut found_double_colon = false;

        self.push_peek_position_into_stack();

        while let Some(current_char) = self.peek_char(0) {
            match current_char {
                '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' => {
                    identifier_buffer.push(*current_char);
                    self.next_char(); // consume char
                }
                ':' if self.peek_char_and_equals(1, ':') => {
                    if found_double_colon {
                        return Err(AsonError::MessageWithPosition(
                            "An enumeration can contain only one \"::\".".to_owned(),
                            *self.peek_position(0).unwrap(),
                        ));
                    }

                    found_double_colon = true;
                    identifier_buffer.push_str("::");
                    self.next_char(); // consume the 1st ':'
                    self.next_char(); // consume the 2nd ':'
                }
                '\u{a0}'..='\u{d7ff}' | '\u{e000}'..='\u{10ffff}' => {
                    // A Rust `char` is a "Unicode scalar value", i.e., any Unicode code point
                    // in the range 0 to 0x10FFFF (inclusive) except the surrogate code points
                    // 0xD800 to 0xDFFF, which are used by UTF-16.
                    // See: https://doc.rust-lang.org/std/primitive.char.html
                    //
                    // The most common CJK characters are in the range '\u{4e00}'..='\u{9fff}'.
                    // For the complete list of CJK characters, see section 18.1
                    // "Han: CJK Unified Ideographs" of the Unicode Standard:
                    //
                    // | Block                                   | Range       | Comment                                              |
                    // |-----------------------------------------|-------------|------------------------------------------------------|
                    // | CJK Unified Ideographs                  | 4E00-9FFF   | Common                                               |
                    // | CJK Unified Ideographs Extension A      | 3400-4DBF   | Rare                                                 |
                    // | CJK Unified Ideographs Extension B      | 20000-2A6DF | Rare, historic                                       |
                    // | CJK Unified Ideographs Extension C      | 2A700-2B73F | Rare, historic                                       |
                    // | CJK Unified Ideographs Extension D      | 2B740-2B81F | Uncommon, some in current use                        |
                    // | CJK Unified Ideographs Extension E      | 2B820-2CEAF | Rare, historic                                       |
                    // | CJK Unified Ideographs Extension F      | 2CEB0-2EBEF | Rare, historic                                       |
                    // | CJK Unified Ideographs Extension G      | 30000-3134F | Rare, historic                                       |
                    // | CJK Unified Ideographs Extension H      | 31350-323AF | Rare, historic                                       |
                    // | CJK Compatibility Ideographs            | F900-FAFF   | Duplicates, unifiable variants, corporate characters |
                    // | CJK Compatibility Ideographs Supplement | 2F800-2FA1F | Unifiable variants                                   |
                    //
                    // See also:
                    //
                    // - https://www.unicode.org/versions/Unicode15.0.0/ch18.pdf
                    // - https://en.wikipedia.org/wiki/CJK_Unified_Ideographs
                    // - https://www.unicode.org/versions/Unicode15.0.0/
                    // - https://www.unicode.org/reports/tr31/tr31-37.html

                    identifier_buffer.push(*current_char);
                    self.next_char(); // consume char
                }
                c if is_terminator(*c) => {
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
            let (type_name, variant_name) = identifier_buffer.split_once("::").unwrap();

            // The variant name must be a valid identifier, i.e., non-empty and not starting with a digit.
            if !matches!(variant_name.chars().next(), Some(c) if !c.is_ascii_digit()) {
                return Err(AsonError::MessageWithRange(
                    format!("Invalid variant name \"{}\" for enumeration.", variant_name),
                    identifier_range,
                ));
            }

            Token::Enumeration(type_name.to_owned(), variant_name.to_owned())
        } else {
            match identifier_buffer.as_str() {
                "true" => Token::Boolean(true),
                "false" => Token::Boolean(false),
                // The default floating-point type is `f64`.
                "NaN" | "NaN_f64" => Token::Number(NumberToken::F64(f64::NAN)),
                "NaN_f32" => Token::Number(NumberToken::F32(f32::NAN)),
                "Inf" | "Inf_f64" => Token::Number(NumberToken::F64(f64::INFINITY)),
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

        // Whether '.' has been found.
        // The presence of '.' indicates a floating-point number.
        let mut found_point = false;

        // Whether 'e' has been found.
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
                    // Valid digits for decimal numbers.
                    number_buffer.push(*current_char);
                    self.next_char(); // consume digit
                }
                '_' => {
                    self.next_char(); // consume '_'
                }
                '.' if !found_point && !found_e => {
                    // Examples of decimal floating-point numbers with '.':
                    // 3.14
                    // 271.828

                    found_point = true;
                    number_buffer.push(*current_char);
                    self.next_char(); // consume '.'
                }
                'e' | 'E' if !found_e => {
                    // Examples of decimal floating-point numbers with 'e':
                    // 123e45
                    // 123e+45
                    // 123e-45

                    found_e = true;
                    number_buffer.push('e');
                    self.next_char(); // consume 'e'
                }
                '+' | '-' if number_buffer.ends_with('e') => {
                    // The sign of the exponent.
                    // Underscores are not pushed into the buffer, so "1e_-5" is accepted too.
                    number_buffer.push(*current_char);
                    self.next_char(); // consume '+' or '-'
                }
                'i' | 'u' | 'f'
                    if number_type_opt.is_none()
                        && matches!(self.peek_char(1), Some('0'..='9')) =>
                {
                    let number_type = self.lex_number_type_suffix()?;
                    number_type_opt.replace(number_type);
                    break;
                }
                c if is_terminator(*c) => {
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

        let number_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        // Validate syntax.

        // Leading zeros are not allowed in the integer part, e.g. "0123" and "00.5".
        let integer_part = number_buffer.split(['.', 'e']).next().unwrap();
        if integer_part.len() > 1 && integer_part.starts_with('0') {
            return Err(AsonError::MessageWithRange(
                "Leading zeros are not allowed in decimal numbers.".to_owned(),
                number_range,
            ));
        }

        // The decimal point must be followed by at least one digit, e.g. "123." and "1.e5" are invalid.
        if number_buffer.ends_with('.') || number_buffer.contains(".e") {
            return Err(AsonError::MessageWithRange(
                "The decimal point must be followed by at least one digit.".to_owned(),
                number_range,
            ));
        }

        // The exponent must contain at least one digit, e.g. "1e", "1e+" and "1e-" are invalid.
        if number_buffer.ends_with(['e', '+', '-']) {
            return Err(AsonError::MessageWithRange(
                "The exponent of a decimal number must contain at least one digit.".to_owned(),
                number_range,
            ));
        }

        // The default types are `f64` for floating-point numbers and `i32` for integers.
        let number_type = number_type_opt.unwrap_or(if found_point || found_e {
            NumberType::F64
        } else {
            NumberType::I32
        });

        let float_error = || {
            AsonError::MessageWithRange(
                format!(
                    "Cannot convert \"{}\" to type \"{}\" (the value is invalid or out of range).",
                    number_buffer, number_type
                ),
                number_range,
            )
        };

        // Parsing a too-large value yields infinity rather than an error, so check for it as well.
        let number_token = match number_type {
            NumberType::F32 => match number_buffer.parse::<f32>() {
                Ok(v) if v.is_finite() => NumberToken::F32(v),
                _ => return Err(float_error()),
            },
            NumberType::F64 => match number_buffer.parse::<f64>() {
                Ok(v) if v.is_finite() => NumberToken::F64(v),
                _ => return Err(float_error()),
            },
            _ => convert_integer_number_string_with_data_type(
                &number_buffer,
                10,
                number_type,
                number_range,
            )?,
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

        let mut number_buffer = String::new();

        // Number type suffix, e.g.,
        // "0xff_u8", "0x1.8p3_f32"
        let mut number_type_opt: Option<NumberType> = None;

        // Whether '.' has been found.
        // The presence of '.' indicates a floating-point number.
        let mut found_point = false;

        // Whether 'p' has been found.
        // The presence of 'p' indicates a hexadecimal floating-point number.
        let mut found_p = false;

        self.push_peek_position_into_stack();

        self.next_char(); // consume '0'
        self.next_char(); // consume 'x'

        while let Some(current_char) = self.peek_char(0) {
            match current_char {
                'f' if number_type_opt.is_none()
                    && found_p
                    && matches!(self.peek_char(1), Some('0'..='9')) =>
                {
                    // 'f' starts a type suffix only after the exponent 'p',
                    // otherwise it is a hexadecimal digit.
                    let number_type = self.lex_number_type_suffix()?;
                    number_type_opt.replace(number_type);
                    break;
                }
                '0'..='9' => {
                    number_buffer.push(*current_char);
                    self.next_char(); // consume digit
                }
                'a'..='f' | 'A'..='F' if !found_p => {
                    // The exponent is a decimal number, so hexadecimal letters
                    // are only allowed before 'p'.
                    number_buffer.push(*current_char);
                    self.next_char(); // consume digit
                }
                '_' => {
                    self.next_char(); // consume '_'
                }
                '.' if !found_point && !found_p => {
                    // Examples of hexadecimal floating-point numbers with '.':
                    // 0x1.9p0
                    // 0x12.bcp0

                    found_point = true;
                    number_buffer.push(*current_char);
                    self.next_char(); // consume '.'
                }
                'p' | 'P' if !found_p => {
                    // Examples of hexadecimal floating-point numbers with 'p':
                    // 0x1.123p45
                    // 0x1.123p+45
                    // 0x1.123p-45

                    found_p = true;
                    number_buffer.push('p');
                    self.next_char(); // consume 'p'
                }
                '+' | '-' if number_buffer.ends_with('p') => {
                    // The sign of the exponent.
                    number_buffer.push(*current_char);
                    self.next_char(); // consume '+' or '-'
                }
                'i' | 'u'
                    if number_type_opt.is_none()
                        && !found_point
                        && !found_p
                        && matches!(self.peek_char(1), Some('0'..='9')) =>
                {
                    // Only integer type suffixes ('i' and 'u') are allowed for hexadecimal integers,
                    // because 'f' is a valid hexadecimal digit.
                    let number_type = self.lex_number_type_suffix()?;
                    number_type_opt.replace(number_type);
                    break;
                }
                c if is_terminator(*c) => {
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

        let number_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        // Validate syntax.

        // An empty hexadecimal number is not allowed, e.g., "0x".
        if number_buffer.is_empty() {
            return Err(AsonError::MessageWithRange(
                "Empty hexadecimal number.".to_owned(),
                number_range,
            ));
        }

        let number_token = if found_point || found_p {
            // A hexadecimal floating-point number must have digits on both sides of '.',
            // and an exponent with at least one digit, e.g., "0x1.8p3".
            let is_valid = match number_buffer.split_once('p') {
                Some((significand, exponent)) => {
                    matches!(significand.split_once('.'),
                        Some((integer, fraction)) if !integer.is_empty() && !fraction.is_empty())
                        && !exponent.trim_start_matches(['+', '-']).is_empty()
                }
                None => false,
            };

            if !is_valid {
                return Err(AsonError::MessageWithRange(
                    format!(
                        "Invalid hexadecimal floating-point number \"0x{}\", the expected format is like \"0x1.8p3\".",
                        number_buffer
                    ),
                    number_range,
                ));
            }

            // The default floating-point type is `f64`.
            let number_type = number_type_opt.unwrap_or(NumberType::F64);
            number_buffer.insert_str(0, "0x");

            // `hexfloat2::parse` does not provide a detailed error message.
            let float_error = || {
                AsonError::MessageWithRange(
                    format!(
                        "Cannot convert \"{}\" to type \"{}\" (the value is invalid or out of range).",
                        number_buffer, number_type
                    ),
                    number_range,
                )
            };

            match number_type {
                NumberType::F32 => match hexfloat2::parse::<f32>(&number_buffer) {
                    Ok(v) if v.is_finite() => NumberToken::F32(v),
                    _ => return Err(float_error()),
                },
                NumberType::F64 => match hexfloat2::parse::<f64>(&number_buffer) {
                    Ok(v) if v.is_finite() => NumberToken::F64(v),
                    _ => return Err(float_error()),
                },
                _ => {
                    return Err(AsonError::MessageWithRange(
                        format!(
                            "Invalid type \"{}\" for hexadecimal floating-point numbers, only \"f32\" and \"f64\" are allowed.",
                            number_type
                        ),
                        number_range,
                    ));
                }
            }
        } else {
            // The default integer type is `i32`.
            convert_integer_number_string_with_data_type(
                &number_buffer,
                16,
                number_type_opt.unwrap_or(NumberType::I32),
                number_range,
            )?
        };

        Ok(TokenWithRange::new(
            Token::Number(number_token),
            number_range,
        ))
    }

    /// Lexes a binary (radix 2) or an octal (radix 8) integer.
    ///
    /// These two formats only support integers, not floating-point numbers.
    fn lex_binary_or_octal_number(&mut self, radix: u32) -> Result<TokenWithRange, AsonError> {
        // ```diagram
        // 0b1010T  //
        // ^^    ^__// to here
        // ||_______// validated
        // |________// current char, validated
        //
        // T = terminator chars || EOF
        // ```

        let format_name = if radix == 2 { "binary" } else { "octal" };

        let mut number_buffer = String::new();
        let mut number_type_opt: Option<NumberType> = None;

        self.push_peek_position_into_stack();

        self.next_char(); // consume '0'
        self.next_char(); // consume 'b' or 'o'

        while let Some(current_char) = self.peek_char(0) {
            match current_char {
                c if c.is_digit(radix) => {
                    number_buffer.push(*current_char);
                    self.next_char(); // consume digit
                }
                '_' => {
                    self.next_char(); // consume '_'
                }
                'i' | 'u'
                    if number_type_opt.is_none()
                        && matches!(self.peek_char(1), Some('0'..='9')) =>
                {
                    let number_type = self.lex_number_type_suffix()?;
                    number_type_opt.replace(number_type);
                    break;
                }
                c if is_terminator(*c) => {
                    break;
                }
                _ => {
                    return Err(AsonError::MessageWithPosition(
                        format!(
                            "Invalid char '{}' for {} number.",
                            current_char, format_name
                        ),
                        *self.peek_position(0).unwrap(),
                    ));
                }
            }
        }

        let number_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        // An empty number is not allowed, e.g., "0b" and "0o".
        if number_buffer.is_empty() {
            return Err(AsonError::MessageWithRange(
                format!("Empty {} number.", format_name),
                number_range,
            ));
        }

        // The default integer type is `i32`.
        let number_token = convert_integer_number_string_with_data_type(
            &number_buffer,
            radix,
            number_type_opt.unwrap_or(NumberType::I32),
            number_range,
        )?;

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
            if current_char.is_ascii_digit() {
                type_buffer.push(*current_char);
                self.next_char(); // consume digit
            } else {
                break;
            }
        }

        let type_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        let number_type = convert_str_to_number_type(&type_buffer)
            .map_err(|msg| AsonError::MessageWithRange(msg, type_range))?;

        // The type suffix must be the end of the number token, e.g., "1u8abc" is invalid.
        if let Some(current_char) = self.peek_char(0)
            && !is_terminator(*current_char)
        {
            return Err(AsonError::MessageWithPosition(
                format!(
                    "Invalid char '{}' after the number type suffix.",
                    current_char
                ),
                *self.peek_position(0).unwrap(),
            ));
        }

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
                    '\\' => match self.next_char() {
                        Some(escape_type) => self.unescape_char(escape_type)?,
                        None => {
                            // `\` + EOF
                            return Err(AsonError::UnexpectedEndOfDocument(
                                "Incomplete escape sequence.".to_owned(),
                            ));
                        }
                    },
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
                    "Incomplete char.".to_owned(),
                ));
            }
        };

        // consume the closing single quote
        match self.next_char() {
            Some('\'') => {
                // Ok
            }
            Some(_) => {
                // `'a?`
                return Err(AsonError::MessageWithPosition(
                    "Expect the closing quote of the char literal.".to_owned(),
                    self.last_position,
                ));
            }
            None => {
                // `'aEOF`
                return Err(AsonError::UnexpectedEndOfDocument(
                    "Incomplete char.".to_owned(),
                ));
            }
        }

        let character_range = Range::new(&self.pop_position_from_stack(), &self.last_position);
        Ok(TokenWithRange::new(Token::Char(character), character_range))
    }

    /// Converts the character following a backslash into the character it represents.
    ///
    /// The escape sequences are shared by char and string literals.
    fn unescape_char(&mut self, escape_type: char) -> Result<char, AsonError> {
        let c = match escape_type {
            '\\' => '\\',
            // A single quote does not need to be escaped in a string literal, and
            // a double quote does not need to be escaped in a char literal,
            // but both are supported for consistency.
            '\'' => '\'',
            '"' => '"',
            't' => '\t', // horizontal tab (U+0009)
            'r' => '\r', // carriage return (CR, U+000D)
            'n' => '\n', // line feed (LF, U+000A)
            '0' => '\0', // null char (U+0000)
            'u' => {
                if self.peek_char_and_equals(0, '{') {
                    // Unicode code point, e.g. '\u{2d}', '\u{6587}'
                    self.unescape_unicode_code_point()?
                } else {
                    return Err(AsonError::MessageWithPosition(
                        "Missing the opening brace of the Unicode escape sequence.".to_owned(),
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
        };

        Ok(c)
    }

    fn unescape_unicode_code_point(&mut self) -> Result<char, AsonError> {
        // ```diagram
        // \u{6587}?  //
        //   ^     ^__// to here
        //   |________// current char, validated
        // ```

        self.push_peek_position_into_stack();

        self.next_char(); // consume char '{'

        let mut codepoint_buffer = String::new();

        loop {
            match self.next_char() {
                Some(current_char) => match current_char {
                    '}' => break,
                    '0'..='9' | 'a'..='f' | 'A'..='F' => codepoint_buffer.push(current_char),
                    _ => {
                        return Err(AsonError::MessageWithPosition(
                            format!(
                                "Invalid char '{}' for Unicode escape sequence.",
                                current_char
                            ),
                            self.last_position,
                        ));
                    }
                },
                None => {
                    // EOF
                    return Err(AsonError::UnexpectedEndOfDocument(
                        "Incomplete Unicode escape sequence.".to_owned(),
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
                "Unicode code point exceeds six digits.".to_owned(),
                codepoint_range,
            ));
        }

        if codepoint_buffer.is_empty() {
            return Err(AsonError::MessageWithRange(
                "Empty Unicode code point.".to_owned(),
                codepoint_range,
            ));
        }

        let codepoint = u32::from_str_radix(&codepoint_buffer, 16).unwrap();

        // `char::from_u32` rejects values above 0x10FFFF and surrogates (0xD800 to 0xDFFF).
        // See: https://doc.rust-lang.org/std/primitive.char.html
        char::from_u32(codepoint).ok_or_else(|| {
            AsonError::MessageWithRange("Invalid Unicode code point.".to_owned(), codepoint_range)
        })
    }

    fn lex_string(&mut self) -> Result<TokenWithRange, AsonError> {
        // ```diagram
        // "abc"?  //
        // ^    ^__// to here
        // |_______// current char, validated
        // ```

        // Save the start position of the string literal (i.e. the opening '"')
        self.push_peek_position_into_stack();

        self.next_char(); // consume '"'

        let mut string_buffer = String::new();

        loop {
            match self.next_char() {
                Some(current_char) => {
                    match current_char {
                        '\\' => match self.next_char() {
                            Some('\r') if self.peek_char_and_equals(0, '\n') => {
                                // Line continuation (concatenated string)
                                self.next_char(); // consume '\n'
                                self.consume_all_leading_whitespaces()?;
                            }
                            Some('\n') => {
                                // Line continuation (concatenated string)
                                self.consume_all_leading_whitespaces()?;
                            }
                            Some(escape_type) => {
                                let c = self.unescape_char(escape_type)?;
                                string_buffer.push(c);
                            }
                            None => {
                                // `\` + EOF
                                return Err(AsonError::UnexpectedEndOfDocument(
                                    "Incomplete escape sequence.".to_owned(),
                                ));
                            }
                        },
                        '"' => {
                            // The closing double quote, i.e., the end of the string literal.
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
                Some(' ' | '\t') => {
                    self.next_char(); // consume ' ' or '\t'
                }
                Some(_) => {
                    break;
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
        // r#"abc"#?  //
        // ^^^     ^__// to here
        // |||________// validated
        // ||_________// validated
        // |__________// current char, validated
        //
        // hash symbol = '#', i.e. the pound sign
        // ```

        self.push_peek_position_into_stack();

        let mut hash_count = 0;

        self.next_char(); // consume char 'r'
        while self.peek_char_and_equals(0, '#') {
            self.next_char(); // consume '#'
            hash_count += 1;
        }

        // consume the opening '"'
        match self.next_char() {
            Some('"') => {}
            Some(current_char) => {
                return Err(AsonError::MessageWithPosition(
                    format!(
                        "Expect '\"' after the hash symbols of the raw string, found '{}'.",
                        current_char
                    ),
                    self.last_position,
                ));
            }
            None => {
                return Err(AsonError::UnexpectedEndOfDocument(
                    "Incomplete raw string.".to_owned(),
                ));
            }
        }

        let mut string_buffer = String::new();

        loop {
            match self.next_char() {
                Some(current_char) => {
                    match current_char {
                        '"' => {
                            // A closing double quote ends the string literal
                            // only if it is followed by the same number of hash symbols
                            // as the opening delimiter.

                            // count the number of hash symbols following the double quote
                            let mut closing_hash_count = 0;
                            while closing_hash_count < hash_count
                                && self.peek_char_and_equals(closing_hash_count, '#')
                            {
                                closing_hash_count += 1;
                            }

                            if closing_hash_count != hash_count {
                                // Not the end of the string, so the '"' is part of the content.
                                string_buffer.push('"');
                            } else {
                                // consume the hash symbols
                                for _ in 0..closing_hash_count {
                                    self.next_char();
                                }
                                break;
                            }
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
        // Note:
        // - The opening `"""` must be immediately followed by a line break.
        // - The closing `"""` must be on a new line (it may be indented).
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
                "The content of an auto-trimmed string must start on a new line.".to_owned(),
                self.last_position,
            ));
        }

        // Each line includes its trailing line break.
        let mut lines: Vec<Vec<char>> = vec![];
        let mut current_line = vec![];

        loop {
            match self.next_char() {
                Some(current_char) => {
                    match current_char {
                        '\n' => {
                            current_line.push('\n');
                            lines.push(std::mem::take(&mut current_line));
                        }
                        '"' if current_line.iter().all(|&c| c == ' ' || c == '\t')
                            && self.peek_char_and_equals(0, '"')
                            && self.peek_char_and_equals(1, '"') =>
                        {
                            // The closing `"""`
                            self.next_char(); // consume '"'
                            self.next_char(); // consume '"'
                            break;
                        }
                        _ => {
                            // ordinary char, including the '\r' of "\r\n"
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

        let count_leading_whitespaces =
            |line: &[char]| line.iter().take_while(|&&c| c == ' ' || c == '\t').count();

        // Empty lines and blank lines (lines containing only whitespace)
        // are ignored when calculating the minimum indentation.
        let is_blank = |line: &[char]| {
            line.iter()
                .all(|&c| c == ' ' || c == '\t' || c == '\r' || c == '\n')
        };

        let min_indent = lines
            .iter()
            .filter(|line| !is_blank(line))
            .map(|line| count_leading_whitespaces(line))
            .min()
            .unwrap_or(0);

        let mut content = String::new();
        for line in &lines {
            // A blank line may be shorter than the minimum indentation.
            let start = min_indent.min(count_leading_whitespaces(line));
            content.extend(&line[start..]);
        }

        // The line break before the closing `"""` is not part of the content.
        if content.ends_with('\n') {
            content.pop();
            if content.ends_with('\r') {
                content.pop();
            }
        }

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
        self.next_char(); // consume the opening quote

        let mut date_buffer = String::new();

        loop {
            match self.next_char() {
                Some(current_char) => {
                    match current_char {
                        '"' => {
                            // The closing quote
                            break;
                        }
                        '0'..='9' | '-' | ':' | ' ' | 't' | 'T' | 'z' | 'Z' | '+' => {
                            // valid chars
                            date_buffer.push(current_char);
                        }
                        _ => {
                            return Err(AsonError::MessageWithPosition(
                                format!("Invalid char '{}' for DateTime.", current_char),
                                self.last_position,
                            ));
                        }
                    }
                }
                None => {
                    // d"...EOF
                    return Err(AsonError::UnexpectedEndOfDocument(
                        "Incomplete DateTime.".to_owned(),
                    ));
                }
            }
        }

        let date_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        // Complete the omitted parts so that the string can be parsed as RFC 3339.
        match date_buffer.len() {
            10 => {
                // YYYY-MM-DD
                date_buffer.push_str("T00:00:00Z");
            }
            19 => {
                // YYYY-MM-DD HH:mm:ss
                date_buffer.push('Z');
            }
            20 | 25 => {
                // YYYY-MM-DDTHH:mm:ssZ
                // YYYY-MM-DDTHH:mm:ss+08:00
            }
            _ => {
                return Err(AsonError::MessageWithRange(
                    format!(
                        "Invalid DateTime \"{}\", the expected format is \"YYYY-MM-DD\", \"YYYY-MM-DD HH:mm:ss\" or \"YYYY-MM-DDTHH:mm:ss(Z|+HH:MM|-HH:MM)\".",
                        date_buffer
                    ),
                    date_range,
                ));
            }
        }

        let rfc3339 = DateTime::parse_from_rfc3339(&date_buffer).map_err(|_| {
            AsonError::MessageWithRange(
                format!("Cannot convert \"{}\" to DateTime.", date_buffer),
                date_range,
            )
        })?;

        Ok(TokenWithRange::new(Token::DateTime(rfc3339), date_range))
    }

    fn lex_hexadecimal_byte_data(&mut self) -> Result<TokenWithRange, AsonError> {
        // ```diagram
        // h"00 11 aa bb"?  //
        // ^^            ^__// to here
        // ||_______________// validated
        // |________________// current char, validated
        // ```

        self.push_peek_position_into_stack();

        self.next_char(); // consume char 'h'
        self.next_char(); // consume the opening quote

        let mut bytes: Vec<u8> = Vec::new();

        // Bytes must be separated by whitespace, so a byte can only start
        // at the beginning of the data or after whitespace.
        let mut after_whitespace = true;

        loop {
            match self.next_char() {
                Some('"') => {
                    // The closing quote
                    break;
                }
                Some(' ' | '\t' | '\r' | '\n') => {
                    after_whitespace = true;
                }
                Some(high_char) => {
                    if !after_whitespace {
                        return Err(AsonError::MessageWithPosition(
                            "Expect whitespace between two bytes of the hexadecimal byte data."
                                .to_owned(),
                            self.last_position,
                        ));
                    }

                    let high = self.hex_digit_value(high_char)?;

                    let low = match self.next_char() {
                        Some(low_char) => self.hex_digit_value(low_char)?,
                        None => {
                            return Err(AsonError::UnexpectedEndOfDocument(
                                "Incomplete hexadecimal byte data.".to_owned(),
                            ));
                        }
                    };

                    bytes.push((high << 4) | low);
                    after_whitespace = false;
                }
                None => {
                    // h"...EOF
                    return Err(AsonError::UnexpectedEndOfDocument(
                        "Incomplete hexadecimal byte data.".to_owned(),
                    ));
                }
            }
        }

        let bytes_range = Range::new(&self.pop_position_from_stack(), &self.last_position);

        Ok(TokenWithRange::new(
            Token::HexadecimalByteData(bytes),
            bytes_range,
        ))
    }

    /// Converts the hexadecimal digit most recently consumed to its value.
    fn hex_digit_value(&self, c: char) -> Result<u8, AsonError> {
        match c.to_digit(16) {
            Some(v) => Ok(v as u8),
            None => Err(AsonError::MessageWithPosition(
                format!("Invalid digit '{}' for hexadecimal byte data.", c),
                self.last_position,
            )),
        }
    }

    fn lex_line_comment(&mut self) {
        // ```diagram
        // //.....?[\r]\n
        // ^^     ^__// to here ('?' = any char or EOF)
        // ||________// validated
        // |_________// current char, validated
        // ```

        self.next_char(); // consume the 1st '/'
        self.next_char(); // consume the 2nd '/'

        while let Some(current_char) = self.peek_char(0) {
            // Ignore all chars until '\n' or "\r\n".
            // The line break itself does not belong to the line comment, so it is not consumed.
            match current_char {
                '\n' => {
                    break;
                }
                '\r' if self.peek_char_and_equals(1, '\n') => {
                    break;
                }
                _ => {
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
                            // Ignore all chars except "/*" and "*/".
                            // Note that "//" has no special meaning within block comments.
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

/// Returns `true` if the character ends a number or an identifier token.
///
/// Note that '/' is a terminator only when it starts a comment, but it is
/// accepted here unconditionally, and an invalid '/' is reported by the next `lex()` call.
fn is_terminator(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t' | '\r' | '\n' | ',' | ':' | '{' | '}' | '[' | ']' | '(' | ')' | '/'
    )
}

#[derive(Debug, Clone, Copy)]
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
        let name = match self {
            NumberType::I8 => "i8",
            NumberType::I16 => "i16",
            NumberType::I32 => "i32",
            NumberType::I64 => "i64",
            NumberType::U8 => "u8",
            NumberType::U16 => "u16",
            NumberType::U32 => "u32",
            NumberType::U64 => "u64",
            NumberType::F32 => "f32",
            NumberType::F64 => "f64",
        };
        f.write_str(name)
    }
}

/// Converts the digits of an integer (without the prefix, sign and underscores)
/// to a `NumberToken` of the given integer type.
///
/// Signed integers are stored as their unsigned counterparts here, and the valid range of
/// signed integers is checked by the normalizer after the sign has been applied.
fn convert_integer_number_string_with_data_type(
    number_string: &str,
    radix: u32,
    number_type: NumberType,
    number_range: Range,
) -> Result<NumberToken, AsonError> {
    let max_value = match number_type {
        NumberType::I8 | NumberType::U8 => u8::MAX as u64,
        NumberType::I16 | NumberType::U16 => u16::MAX as u64,
        NumberType::I32 | NumberType::U32 => u32::MAX as u64,
        NumberType::I64 | NumberType::U64 => u64::MAX,
        NumberType::F32 | NumberType::F64 => unreachable!(),
    };

    let v = u64::from_str_radix(number_string, radix)
        .ok()
        .filter(|v| *v <= max_value)
        .ok_or_else(|| {
            let prefix = match radix {
                2 => "0b",
                8 => "0o",
                16 => "0x",
                _ => "",
            };

            AsonError::MessageWithRange(
                format!(
                    "Cannot convert \"{}{}\" to type \"{}\" (the value is invalid or out of range).",
                    prefix, number_string, number_type
                ),
                number_range,
            )
        })?;

    // The casts below cannot truncate, because `v` has been checked against `max_value`.
    let token = match number_type {
        NumberType::I8 => NumberToken::I8(v as u8),
        NumberType::U8 => NumberToken::U8(v as u8),
        NumberType::I16 => NumberToken::I16(v as u16),
        NumberType::U16 => NumberToken::U16(v as u16),
        NumberType::I32 => NumberToken::I32(v as u32),
        NumberType::U32 => NumberToken::U32(v as u32),
        NumberType::I64 => NumberToken::I64(v),
        NumberType::U64 => NumberToken::U64(v),
        NumberType::F32 | NumberType::F64 => unreachable!(),
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
        char_with_position::CharsWithPositionIterator,
        error::AsonError,
        lexer::{NumberToken, TokenWithRange},
        peekable_iterator::PeekableIterator,
        position::Position,
        range::Range,
    };

    use super::{Lexer, Token};

    impl Token {
        pub fn new_enumeration(type_name: &str, variant_name: &str) -> Self {
            Token::Enumeration(type_name.to_owned(), variant_name.to_owned())
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
        let chars = s.chars();

        let char_position_iter = CharsWithPositionIterator::new(chars);
        let peekable_char_position_iter = PeekableIterator::new(char_position_iter);
        let lexer = Lexer::new(peekable_char_position_iter);

        // Collect tokens
        //
        // do not use `iter.collect::<Vec<_>>()` to collect tokens,
        // because the `Lexer` throws exceptions via `next() -> Option<Result<...>>`.
        //
        // If we use `collect()`, once an error occurs,
        // the iterator would not stop immediately; instead, it would continue iterating to the end.
        let mut token_with_ranges = vec![];
        for result in lexer {
            let twr = result?;
            token_with_ranges.push(twr);
        }

        Ok(token_with_ranges)
    }

    /// Helper function to lex tokens from a string, without location info
    fn lex_from_str_without_location(s: &'static str) -> Result<Vec<Token>, AsonError> {
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
            vec![Token::ParenthesisOpen, Token::ParenthesisClose]
        );

        assert_eq!(
            lex_from_str_without_location("(  )").unwrap(),
            vec![Token::ParenthesisOpen, Token::ParenthesisClose]
        );

        assert_eq!(
            lex_from_str_without_location("( , )").unwrap(),
            vec![Token::ParenthesisOpen, Token::ParenthesisClose]
        );

        assert_eq!(
            lex_from_str_without_location("( , , ,, )").unwrap(),
            vec![Token::ParenthesisOpen, Token::ParenthesisClose]
        );

        assert_eq!(
            lex_from_str_without_location("(\t\r\n\n\n)").unwrap(),
            vec![Token::ParenthesisOpen, Token::ParenthesisClose,]
        );

        assert_eq!(
            lex_from_str_without_location("(\n,\n)").unwrap(),
            vec![Token::ParenthesisOpen, Token::ParenthesisClose]
        );

        // A lone '\r' is whitespace.
        assert_eq!(
            lex_from_str_without_location("1\r2").unwrap(),
            vec![
                Token::Number(NumberToken::I32(1)),
                Token::Number(NumberToken::I32(2)),
            ]
        );

        // Testing the ranges

        assert_eq!(lex_from_str("  ").unwrap(), vec![]);

        assert_eq!(
            lex_from_str("()").unwrap(),
            vec![
                TokenWithRange::new(Token::ParenthesisOpen, Range::from_detail(0, 0, 0, 1)),
                TokenWithRange::new(Token::ParenthesisClose, Range::from_detail(1, 0, 1, 1)),
            ]
        );

        assert_eq!(
            lex_from_str("(  )").unwrap(),
            vec![
                TokenWithRange::new(Token::ParenthesisOpen, Range::from_detail(0, 0, 0, 1)),
                TokenWithRange::new(Token::ParenthesisClose, Range::from_detail(3, 0, 3, 1)),
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
                TokenWithRange::new(Token::ParenthesisOpen, Range::from_detail(0, 0, 0, 1)),
                TokenWithRange::new(Token::ParenthesisClose, Range::from_detail(6, 3, 0, 1)),
            ]
        );
    }

    #[test]
    fn test_lex_punctuations() {
        assert_eq!(
            lex_from_str_without_location(",:{}[]()+-").unwrap(),
            vec![
                Token::Colon,
                Token::BraceOpen,
                Token::BraceClose,
                Token::BracketOpen,
                Token::BracketClose,
                Token::ParenthesisOpen,
                Token::ParenthesisClose,
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
                Token::ParenthesisOpen,
                Token::new_identifier("name"),
                Token::ParenthesisClose,
            ]
        );

        assert_eq!(
            lex_from_str_without_location("( a )").unwrap(),
            vec![
                Token::ParenthesisOpen,
                Token::new_identifier("a"),
                Token::ParenthesisClose,
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
                    Token::BracketOpen,
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
                    Token::BracketClose,
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
                Token::ParenthesisOpen,
                Token::Number(NumberToken::I32(211)),
                Token::ParenthesisClose,
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
                    end_inclusive: Position {
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
                    end_inclusive: Position {
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

        // Underscores are allowed between the exponent marker and its sign.
        assert_eq!(
            lex_from_str_without_location("1e_-5").unwrap(),
            vec![Token::Number(NumberToken::F64(1e-5))]
        );

        assert_eq!(
            lex_from_str_without_location("1E5").unwrap(),
            vec![Token::Number(NumberToken::F64(1e5))]
        );

        assert_eq!(
            lex_from_str_without_location("0x1.8P1").unwrap(),
            vec![Token::Number(NumberToken::F64(3.0))]
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
                    end_inclusive: Position {
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
                    end_inclusive: Position {
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

        // err: the decimal point must be followed by digits
        assert!(lex_from_str("1.e5").is_err());

        // err: leading zeros in the integer part
        assert!(lex_from_str("00.5").is_err());
        assert!(lex_from_str("01e5").is_err());

        // err: missing exponent digits
        assert!(lex_from_str("1e+").is_err());
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
                        end_inclusive: Position {
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
                        end_inclusive: Position {
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
                        end_inclusive: Position {
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
                        end_inclusive: Position {
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
                        end_inclusive: Position {
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
                        end_inclusive: Position {
                            index: 10,
                            line: 0,
                            column: 10
                        }
                    }
                ))
            ));
        }

        // err: invalid char after the type suffix
        assert!(matches!(
            lex_from_str("1u8abc"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 3,
                    line: 0,
                    column: 3,
                }
            ))
        ));
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
                    end_inclusive: Position {
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
                    end_inclusive: Position {
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
                        end_inclusive: Position {
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
                        end_inclusive: Position {
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
                        end_inclusive: Position {
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
                        end_inclusive: Position {
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
                    end_inclusive: Position {
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
                    end_inclusive: Position {
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
                    end_inclusive: Position {
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

        // err: hexadecimal floating-point numbers require digits on both sides of '.'
        assert!(lex_from_str("0x1p3").is_err());
        assert!(lex_from_str("0x1.p3").is_err());
        assert!(lex_from_str("0x.8p1").is_err());
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
                    end_inclusive: Position {
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
                    end_inclusive: Position {
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
                        end_inclusive: Position {
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
                        end_inclusive: Position {
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
                        end_inclusive: Position {
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
                        end_inclusive: Position {
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
                    end_inclusive: Position {
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
                    end_inclusive: Position {
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
                        end_inclusive: Position {
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
                        end_inclusive: Position {
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
                        end_inclusive: Position {
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
                        end_inclusive: Position {
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
                Token::ParenthesisOpen,
                Token::Char('a'),
                Token::ParenthesisClose
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
                    end_inclusive: Position {
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
                    end_inclusive: Position {
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
                    end_inclusive: Position {
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
                    end_inclusive: Position {
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
                Token::ParenthesisOpen,
                Token::new_string("abc"),
                Token::ParenthesisClose,
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
                    end_inclusive: Position {
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
                    end_inclusive: Position {
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
                    end_inclusive: Position {
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

        // Testing raw string with hash symbol
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

        // err: missing '"' after the hash symbols of a raw string
        assert!(lex_from_str("r#abc\"#").is_err());

        // Testing raw string with multiple hash symbols
        assert_eq!(
            lex_from_str_without_location("r##\"abc\n    xyz\"## r###\"foo\\nbar\"###").unwrap(),
            vec![
                Token::new_string("abc\n    xyz"),
                Token::new_string("foo\\nbar")
            ]
        );

        // Test the ranges for raw strings with multiple hash symbols
        assert_eq!(
            lex_from_str("r##\"abc\n    xyz\"## r###\"foo\\nbar\"###").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::new_string("abc\n    xyz"),
                    Range::new(&Position::new(0, 0, 0), &Position::new(17, 1, 9))
                ),
                TokenWithRange::new(
                    Token::new_string("foo\\nbar"),
                    Range::new(&Position::new(19, 1, 11), &Position::new(35, 1, 27))
                )
            ]
        );

        // Test hash symbols within raw strings
        assert_eq!(
            lex_from_str("r##\"abc#def\"## r##\"foo\"#bar\"##").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::new_string("abc#def"),
                    Range::new(&Position::new(0, 0, 0), &Position::new(13, 0, 13))
                ),
                TokenWithRange::new(
                    Token::new_string("foo\"#bar"),
                    Range::new(&Position::new(15, 0, 15), &Position::new(29, 0, 29))
                )
            ]
        );

        // err: un-matched hash symbols
        assert!(matches!(
            lex_from_str("r##\"abc\n    xyz\"#").unwrap_err(),
            AsonError::UnexpectedEndOfDocument(_)
        ));

        assert!(matches!(
            lex_from_str("r##\"abc\n    xyz\"###").unwrap_err(),
            // The 3rd '#' will be unexpected character since it doesn't match the opening '##' raw string delimiter
            AsonError::MessageWithPosition(
                _,
                Position {
                    index: 18,
                    line: 1,
                    column: 10
                }
            )
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

        // Blank lines are ignored when calculating the minimum indentation.
        assert_eq!(
            lex_from_str_without_location("\"\"\"\n    a\n  \n    b\n\"\"\"").unwrap(),
            vec![Token::new_string("a\n\nb")]
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
                    Token::BracketOpen,
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
                    Token::BracketClose,
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

        // err: too many digits, no whitespace between two bytes
        assert!(matches!(
            lex_from_str("h\"11 1317\""),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 7,
                    line: 0,
                    column: 7,
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
                    index: 4,
                    line: 0,
                    column: 4,
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
            vec![Token::DateTime(expect_date1)]
        );

        assert_eq!(
            lex_from_str_without_location("d\"2024-03-16T16:30:50Z\"").unwrap(),
            vec![Token::DateTime(expect_date2)]
        );

        assert_eq!(
            lex_from_str_without_location("d\"2024-03-16T16:30:50z\"").unwrap(),
            vec![Token::DateTime(expect_date2)]
        );

        assert_eq!(
            lex_from_str_without_location("d\"2024-03-16T16:30:50\"").unwrap(),
            vec![Token::DateTime(expect_date2)]
        );

        assert_eq!(
            lex_from_str_without_location("d\"2024-03-16t16:30:50\"").unwrap(),
            vec![Token::DateTime(expect_date2)]
        );

        assert_eq!(
            lex_from_str_without_location("d\"2024-03-16 16:30:50\"").unwrap(),
            vec![Token::DateTime(expect_date2)]
        );

        assert_eq!(
            lex_from_str_without_location("d\"2024-03-16T16:30:50+08:00\"").unwrap(),
            vec![Token::DateTime(expect_date3)]
        );

        // Testing the ranges

        assert_eq!(
            lex_from_str("d\"2024-03-16\" d\"2024-03-16T16:30:50+08:00\"").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::DateTime(expect_date1),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 13)
                ),
                TokenWithRange::new(
                    Token::DateTime(expect_date3),
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
                    end_inclusive: Position {
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
                    end_inclusive: Position {
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
            vec![Token::new_enumeration("Option", "None")]
        );

        assert_eq!(
            lex_from_str_without_location("Option::Some(123)").unwrap(),
            vec![
                Token::new_enumeration("Option", "Some"),
                Token::ParenthesisOpen,
                Token::Number(NumberToken::I32(123)),
                Token::ParenthesisClose,
            ]
        );

        assert_eq!(
            lex_from_str_without_location("value: Result::Ok(456)").unwrap(),
            vec![
                Token::new_identifier("value"),
                Token::Colon,
                Token::new_enumeration("Result", "Ok"),
                Token::ParenthesisOpen,
                Token::Number(NumberToken::I32(456)),
                Token::ParenthesisClose,
            ]
        );

        // err: invalid variant
        assert!(matches!(
            lex_from_str("Enum::Foo::Bar"),
            Err(AsonError::MessageWithPosition(
                _,
                Position {
                    index: 9,
                    line: 0,
                    column: 9,
                }
            ))
        ));

        // err: invalid variant names
        assert!(lex_from_str("Color::").is_err());
        assert!(lex_from_str("Color::1").is_err());
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

        // A line comment ends before "\r\n".
        assert_eq!(
            lex_from_str_without_location("// abc\r\n1").unwrap(),
            vec![Token::Number(NumberToken::I32(1))]
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
                Token::BracketOpen,
                Token::Number(NumberToken::I32(1)),
                Token::Number(NumberToken::I32(2)),
                Token::Number(NumberToken::I32(3)),
                Token::Number(NumberToken::I32(4)),
                Token::Number(NumberToken::I32(5)),
                Token::Number(NumberToken::I32(6)),
                Token::Number(NumberToken::I32(7)),
                Token::Number(NumberToken::I32(8)),
                Token::BracketClose,
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
                Token::BraceOpen,
                Token::new_identifier("id"),
                Token::Colon,
                Token::Number(NumberToken::I32(123)),
                Token::new_identifier("name"),
                Token::Colon,
                Token::new_string("foo"),
                Token::BraceClose,
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
                Token::BracketOpen,
                Token::Number(NumberToken::I32(123)),
                Token::Number(NumberToken::I32(456)),
                Token::Number(NumberToken::I32(789)),
                Token::BracketClose,
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
                Token::ParenthesisOpen,
                Token::Number(NumberToken::I32(123)),
                Token::new_string("foo"),
                Token::Boolean(true),
                Token::ParenthesisClose,
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
                Token::BraceOpen, // {
                Token::new_identifier("a"),
                Token::Colon,
                Token::BracketOpen, // [
                Token::Number(NumberToken::I32(1)),
                Token::Number(NumberToken::I32(2)),
                Token::Number(NumberToken::I32(3)),
                Token::BracketClose, // ]
                Token::new_identifier("b"),
                Token::Colon,
                Token::ParenthesisOpen, // (
                Token::Boolean(false),
                Token::DateTime(DateTime::parse_from_rfc3339("2000-01-01 10:10:10Z").unwrap()),
                Token::ParenthesisClose, // )
                Token::new_identifier("c"),
                Token::Colon,
                Token::BraceOpen, // {
                Token::new_identifier("id"),
                Token::Colon,
                Token::Number(NumberToken::I32(11)),
                Token::BraceClose, // }
                Token::BraceClose, // }
            ]
        );
    }
}
