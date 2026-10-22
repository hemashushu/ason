// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::{io::Read, marker::PhantomData};

use crate::{
    char_with_position::CharsWithPositionIterator,
    error::AsonError,
    lexer::Lexer,
    normalizer::NormalizeSignedNumberIter,
    peekable_iterator::PeekableIterator,
    range::Range,
    token::{NumberToken, Token, TokenWithRange},
    utf8_char_iterator::UTF8CharIterator,
};
use serde::de::{self, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess};

pub fn de_from_str<T>(s: &str) -> Result<T, AsonError>
where
    T: de::DeserializeOwned,
{
    de_from_char_iterator(s.chars())
}

pub fn de_from_reader<T, R: Read>(reader: R) -> Result<T, AsonError>
where
    T: de::DeserializeOwned,
{
    de_from_char_iterator(UTF8CharIterator::new(reader))
}

pub fn de_from_char_iterator<T, CharIter>(char_iterator: CharIter) -> Result<T, AsonError>
where
    T: de::DeserializeOwned,
    CharIter: Iterator<Item = char>,
{
    // `DeserializeOwned` is required because tokens own their data,
    // so nothing can be borrowed from the input. See: https://serde.rs/lifetimes.html

    let mut deserializer = Deserializer::from_char_iterator(char_iterator);
    let value = T::deserialize(&mut deserializer)?;

    // Check for extraneous tokens after the root value.
    match deserializer.next_token()? {
        Some(_) => Err(AsonError::MessageWithRange(
            "Unexpected token after the end of the document.".to_owned(),
            deserializer.last_range,
        )),
        None => Ok(value),
    }
}

/// Opens a reader that deserializes the elements of a root List one at a time.
///
/// It returns a `ListStreamReader`, which is an iterator over the deserialized elements.
/// The input can be an owned character iterator or a mutable reference to one.
///
/// - If the source is a `std::io::Read`, use `let mut char_iter = UTF8CharIterator::new(&mut reader)`
///   to convert it to a character iterator.
/// - If the source is a string slice, use `let mut chars = s.chars()` to convert it to a character iterator.
pub fn list_stream_reader<T, CharIter>(
    char_iterator: CharIter,
) -> Result<ListStreamReader<T, CharIter>, AsonError>
where
    T: de::DeserializeOwned,
    CharIter: Iterator<Item = char>,
{
    let mut deserializer = Deserializer::from_char_iterator(char_iterator);

    // consume the opening bracket '['
    deserializer.consume_token_and_assert(&Token::BracketOpen, "a \"List\"")?;

    Ok(ListStreamReader::new(deserializer))
}

pub struct ListStreamReader<T, CharIter>
where
    T: de::DeserializeOwned,
    CharIter: Iterator<Item = char>,
{
    deserializer: Deserializer<CharIter>,

    // Set when the closing bracket ']' has been consumed or an error has occurred,
    // after which the iterator only returns `None`.
    finished: bool,

    _marker: PhantomData<fn() -> T>,
}

impl<T, CharIter> ListStreamReader<T, CharIter>
where
    T: de::DeserializeOwned,
    CharIter: Iterator<Item = char>,
{
    fn new(deserializer: Deserializer<CharIter>) -> Self {
        Self {
            deserializer,
            finished: false,
            _marker: PhantomData,
        }
    }
}

impl<T, CharIter> Iterator for ListStreamReader<T, CharIter>
where
    T: de::DeserializeOwned,
    CharIter: Iterator<Item = char>,
{
    type Item = Result<T, AsonError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let result = match self
            .deserializer
            .peek_token_and_equals(0, &Token::BracketClose)
        {
            Ok(true) => {
                // The closing bracket ']' ends the list.
                // It has been peeked successfully, so consuming it cannot fail.
                self.finished = true;
                let _ = self.deserializer.next_token();
                return None;
            }
            Ok(false) => T::deserialize(&mut self.deserializer),
            Err(e) => Err(e),
        };

        // The token stream cannot be resynchronized after an error.
        if result.is_err() {
            self.finished = true;
        }

        Some(result)
    }
}

type UpstreamIterator<CharIter> =
    NormalizeSignedNumberIter<Lexer<CharsWithPositionIterator<CharIter>>>;

pub struct Deserializer<CharIter: Iterator<Item = char>> {
    upstream: PeekableIterator<Result<TokenWithRange, AsonError>, UpstreamIterator<CharIter>>,

    /// The range of the last token consumed by `next_token()`.
    last_range: Range,
}

impl<CharIter: Iterator<Item = char>> Deserializer<CharIter> {
    fn from_char_iterator(char_iterator: CharIter) -> Self {
        // The processing pipeline:
        // chars -> chars with positions -> tokens -> tokens with signed numbers normalized
        let char_position_iter = CharsWithPositionIterator::new(char_iterator);
        let lexer = Lexer::new(PeekableIterator::new(char_position_iter));
        let normalizer = NormalizeSignedNumberIter::new(PeekableIterator::new(lexer));

        Self {
            upstream: PeekableIterator::new(normalizer),
            last_range: Range::default(),
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, AsonError> {
        match self.upstream.next() {
            Some(Ok(TokenWithRange { token, range })) => {
                self.last_range = range;
                Ok(Some(token))
            }
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    fn peek_token(&self, offset: usize) -> Result<Option<&Token>, AsonError> {
        match self.upstream.peek(offset) {
            Some(Ok(TokenWithRange { token, .. })) => Ok(Some(token)),
            Some(Err(e)) => Err(e.clone()),
            None => Ok(None),
        }
    }

    // Peek at the next token and compare it with the expected token.
    // Return false when it does not match or when there are no more tokens.
    // Return an error if lexing fails while peeking.
    fn peek_token_and_equals(
        &self,
        offset: usize,
        expected_token: &Token,
    ) -> Result<bool, AsonError> {
        Ok(matches!(
            self.peek_token(offset)?,
            Some(token) if token == expected_token))
    }

    // Build the error for a token that does not match the expectation.
    // `found_token` is false when the end of the document has been reached,
    // otherwise the error points to the last consumed token.
    fn expect_error(&self, found_token: bool, expected: &str) -> AsonError {
        let message = format!("Expect {}.", expected);
        if found_token {
            AsonError::MessageWithRange(message, self.last_range)
        } else {
            AsonError::UnexpectedEndOfDocument(message)
        }
    }

    // Consume the next token and assert that it matches the expected token.
    // Return an error if it does not match or if there are no more tokens.
    fn consume_token_and_assert(
        &mut self,
        expected_token: &Token,
        expected: &str,
    ) -> Result<(), AsonError> {
        match self.next_token()? {
            Some(token) if &token == expected_token => Ok(()),
            other => Err(self.expect_error(other.is_some(), expected)),
        }
    }

    // Consume '(', error if the next token is not '(' or there are no more tokens.
    fn consume_opening_parenthesis(&mut self) -> Result<(), AsonError> {
        self.consume_token_and_assert(&Token::ParenthesisOpen, "an opening parenthesis \"(\"")
    }

    // Consume ')', error if the next token is not ')' or there are no more tokens.
    fn consume_closing_parenthesis(&mut self) -> Result<(), AsonError> {
        self.consume_token_and_assert(&Token::ParenthesisClose, "a closing parenthesis \")\"")
    }

    // Consume ']', error if the next token is not ']' or there are no more tokens.
    fn consume_closing_bracket(&mut self) -> Result<(), AsonError> {
        self.consume_token_and_assert(&Token::BracketClose, "a closing bracket \"]\"")
    }

    // Consume '}', error if the next token is not '}' or there are no more tokens.
    fn consume_closing_brace(&mut self) -> Result<(), AsonError> {
        self.consume_token_and_assert(&Token::BraceClose, "a closing brace \"}\"")
    }

    // Consume ':', error if the next token is not ':' or there are no more tokens.
    fn consume_colon(&mut self) -> Result<(), AsonError> {
        self.consume_token_and_assert(&Token::Colon, "a colon \":\"")
    }

    // Consume a complete value without interpreting it.
    // It is used to skip values that are not needed, such as unknown Object fields.
    fn skip_value(&mut self) -> Result<(), AsonError> {
        match self.next_token()? {
            Some(Token::BracketOpen) => self.skip_until_closing(Token::BracketClose),
            Some(Token::ParenthesisOpen) => self.skip_until_closing(Token::ParenthesisClose),
            Some(Token::BraceOpen) => self.skip_until_closing(Token::BraceClose),
            Some(Token::Enumeration(..)) => {
                // A variant may carry a value in parentheses or braces.
                if self.peek_token_and_equals(0, &Token::ParenthesisOpen)? {
                    self.next_token()?;
                    self.skip_until_closing(Token::ParenthesisClose)
                } else if self.peek_token_and_equals(0, &Token::BraceOpen)? {
                    self.next_token()?;
                    self.skip_until_closing(Token::BraceClose)
                } else {
                    Ok(())
                }
            }
            Some(
                Token::Number(_)
                | Token::Char(_)
                | Token::String(_)
                | Token::Boolean(_)
                | Token::DateTime(_)
                | Token::HexadecimalByteData(_),
            ) => Ok(()),
            other => Err(self.expect_error(other.is_some(), "a value")),
        }
    }

    // Consume tokens until the closing token that matches an already consumed opening token.
    fn skip_until_closing(&mut self, closing_token: Token) -> Result<(), AsonError> {
        // The expected closing tokens of the nested compound values.
        let mut closing_stack = vec![closing_token];

        while !closing_stack.is_empty() {
            match self.next_token()? {
                Some(Token::BracketOpen) => closing_stack.push(Token::BracketClose),
                Some(Token::ParenthesisOpen) => closing_stack.push(Token::ParenthesisClose),
                Some(Token::BraceOpen) => closing_stack.push(Token::BraceClose),
                Some(
                    token @ (Token::BracketClose | Token::ParenthesisClose | Token::BraceClose),
                ) => {
                    if closing_stack.last() == Some(&token) {
                        closing_stack.pop();
                    } else {
                        return Err(AsonError::MessageWithRange(
                            "Mismatched closing bracket.".to_owned(),
                            self.last_range,
                        ));
                    }
                }
                Some(_) => {}
                None => {
                    return Err(AsonError::UnexpectedEndOfDocument(
                        "Incomplete value.".to_owned(),
                    ));
                }
            }
        }

        Ok(())
    }
}

impl<'de, CharIter: Iterator<Item = char>> de::Deserializer<'de> for &mut Deserializer<CharIter> {
    type Error = AsonError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // ASON values are always deserialized into a concrete type, so there
        // is no support for self-describing deserialization, such as untagged enums.
        Err(AsonError::Message(
            "The ASON deserializer requires the target type to be known (\"deserialize_any\" is not supported)."
                .to_owned(),
        ))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Boolean(v)) => visitor.visit_bool(v),
            other => Err(self.expect_error(other.is_some(), "a \"Boolean\" value")),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::I8(v))) => visitor.visit_i8(v as i8),
            other => Err(self.expect_error(other.is_some(), "an \"i8\" value")),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::I16(v))) => visitor.visit_i16(v as i16),
            other => Err(self.expect_error(other.is_some(), "an \"i16\" value")),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::I32(v))) => visitor.visit_i32(v as i32),
            other => Err(self.expect_error(other.is_some(), "an \"i32\" value")),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::I64(v))) => visitor.visit_i64(v as i64),
            other => Err(self.expect_error(other.is_some(), "an \"i64\" value")),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::U8(v))) => visitor.visit_u8(v),
            other => Err(self.expect_error(other.is_some(), "a \"u8\" value")),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::U16(v))) => visitor.visit_u16(v),
            other => Err(self.expect_error(other.is_some(), "a \"u16\" value")),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::U32(v))) => visitor.visit_u32(v),
            other => Err(self.expect_error(other.is_some(), "a \"u32\" value")),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::U64(v))) => visitor.visit_u64(v),
            other => Err(self.expect_error(other.is_some(), "a \"u64\" value")),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::F32(v))) => visitor.visit_f32(v),
            other => Err(self.expect_error(other.is_some(), "an \"f32\" value")),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::F64(v))) => visitor.visit_f64(v),
            other => Err(self.expect_error(other.is_some(), "an \"f64\" value")),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Char(c)) => visitor.visit_char(c),
            other => Err(self.expect_error(other.is_some(), "a \"Char\" value")),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // The string is owned by the token, so it cannot be borrowed from the input.
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::String(s)) => visitor.visit_string(s),
            other => Err(self.expect_error(other.is_some(), "a \"String\" value")),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // The bytes are owned by the token, so they cannot be borrowed from the input.
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::HexadecimalByteData(d)) => visitor.visit_byte_buf(d),
            other => Err(self.expect_error(other.is_some(), "\"Hexadecimal Byte Data\"")),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Enumeration(type_name, variant_name)) if type_name == "Option" => {
                match variant_name.as_str() {
                    "None" if !self.peek_token_and_equals(0, &Token::ParenthesisOpen)? => {
                        visitor.visit_none()
                    }
                    "Some" if self.peek_token_and_equals(0, &Token::ParenthesisOpen)? => {
                        self.next_token()?; // consume '('
                        let value = visitor.visit_some(&mut *self)?;
                        self.consume_closing_parenthesis()?;
                        Ok(value)
                    }
                    _ => Err(AsonError::MessageWithRange(
                        "Expect \"Option::None\" or \"Option::Some(...)\".".to_owned(),
                        self.last_range,
                    )),
                }
            }
            other => Err(self.expect_error(other.is_some(), "an \"Option\" enumeration")),
        }
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // The type of `()` in Rust.
        Err(AsonError::Message(
            "ASON does not support the unit type \"()\".".to_owned(),
        ))
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // A unit struct is a struct without fields, for example `struct Unit;`.
        Err(AsonError::Message(
            "ASON does not support unit structs.".to_owned(),
        ))
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // A newtype struct is a tuple struct with a single field, for example `struct NewType(u8);`.
        Err(AsonError::Message(
            "ASON does not support newtype structs.".to_owned(),
        ))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // seq = List
        match self.next_token()? {
            Some(Token::BracketOpen) => {
                let value =
                    visitor.visit_seq(SeqAccessor::new(self, Token::BracketClose, "List"))?;
                self.consume_closing_bracket()?; // consume ']'
                Ok(value)
            }
            other => Err(self.expect_error(other.is_some(), "a \"List\"")),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // Note that Rust fixed-length arrays, such as `[i32; 4]`, are also treated as tuples.
        match self.next_token()? {
            Some(Token::ParenthesisOpen) => {
                if self.peek_token_and_equals(0, &Token::ParenthesisClose)? {
                    self.next_token()?; // consume ')'
                    return Err(AsonError::MessageWithRange(
                        "A \"Tuple\" must contain at least one element.".to_owned(),
                        self.last_range,
                    ));
                }

                let value =
                    visitor.visit_seq(SeqAccessor::new(self, Token::ParenthesisClose, "Tuple"))?;
                self.consume_closing_parenthesis()?; // consume ')'
                Ok(value)
            }
            other => Err(self.expect_error(other.is_some(), "a \"Tuple\"")),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // A tuple struct is a struct that looks like a tuple, for example `struct TupleStruct(u8, String);`.
        Err(AsonError::Message(
            "ASON does not support tuple structs.".to_owned(),
        ))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // map = Map
        match self.next_token()? {
            Some(Token::BracketOpen) => {
                let value =
                    visitor.visit_map(MapAccessor::new(self, Token::BracketClose, "Map"))?;
                self.consume_closing_bracket()?; // consume ']'
                Ok(value)
            }
            other => Err(self.expect_error(other.is_some(), "a \"Map\"")),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // struct = Object
        match self.next_token()? {
            Some(Token::BraceOpen) => {
                let value =
                    visitor.visit_map(MapAccessor::new(self, Token::BraceClose, "Object"))?;
                self.consume_closing_brace()?; // consume '}'
                Ok(value)
            }
            other => Err(self.expect_error(other.is_some(), "an \"Object\"")),
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Enumeration(type_name, variant_name)) if type_name == name => {
                if self.peek_token_and_equals(0, &Token::ParenthesisOpen)?
                    || self.peek_token_and_equals(0, &Token::BraceOpen)?
                {
                    // A variant with a value: single-value, tuple-like or object-like variant.
                    visitor.visit_enum(VariantAccessor::new(self, variant_name))
                } else {
                    // A variant without a value.
                    visitor.visit_enum(variant_name.into_deserializer())
                }
            }
            Some(Token::Enumeration(type_name, _)) => Err(AsonError::MessageWithRange(
                format!(
                    "Expect an enumeration of type \"{}\", found \"{}\".",
                    name, type_name
                ),
                self.last_range,
            )),
            other => Err(self.expect_error(
                other.is_some(),
                &format!("an enumeration of type \"{}\"", name),
            )),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // In Serde, an identifier is the name of a struct field (or of an enum variant).
        match self.next_token()? {
            Some(Token::Identifier(id)) => visitor.visit_string(id),
            other => Err(self.expect_error(other.is_some(), "an identifier")),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // It is called for values that are not needed, such as the values of
        // unknown Object fields (unless `#[serde(deny_unknown_fields)]` is specified).
        self.skip_value()?;
        visitor.visit_unit()
    }
}

// Accessor for Lists and Tuples.
struct SeqAccessor<'a, CharIter: Iterator<Item = char>> {
    de: &'a mut Deserializer<CharIter>,
    closing_token: Token,
    type_name: &'static str,
}

impl<'a, CharIter: Iterator<Item = char>> SeqAccessor<'a, CharIter> {
    fn new(
        de: &'a mut Deserializer<CharIter>,
        closing_token: Token,
        type_name: &'static str,
    ) -> Self {
        Self {
            de,
            closing_token,
            type_name,
        }
    }
}

impl<'de, CharIter: Iterator<Item = char>> SeqAccess<'de> for SeqAccessor<'_, CharIter> {
    type Error = AsonError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, AsonError>
    where
        T: de::DeserializeSeed<'de>,
    {
        // The closing token ends the sequence. For Tuples, this allows the visitor
        // to report an "invalid length" error when there are fewer elements than expected.
        if self.de.peek_token_and_equals(0, &self.closing_token)? {
            return Ok(None);
        }

        if self.de.peek_token(0)?.is_none() {
            return Err(AsonError::UnexpectedEndOfDocument(format!(
                "Incomplete \"{}\".",
                self.type_name
            )));
        }

        seed.deserialize(&mut *self.de).map(Some)
    }
}

// Accessor for Maps and Objects.
struct MapAccessor<'a, CharIter: Iterator<Item = char>> {
    de: &'a mut Deserializer<CharIter>,
    closing_token: Token,
    type_name: &'static str,
}

impl<'a, CharIter: Iterator<Item = char>> MapAccessor<'a, CharIter> {
    fn new(
        de: &'a mut Deserializer<CharIter>,
        closing_token: Token,
        type_name: &'static str,
    ) -> Self {
        Self {
            de,
            closing_token,
            type_name,
        }
    }
}

impl<'de, CharIter: Iterator<Item = char>> MapAccess<'de> for MapAccessor<'_, CharIter> {
    type Error = AsonError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, AsonError>
    where
        K: de::DeserializeSeed<'de>,
    {
        // `MapAccess` does not stop automatically, so check for the closing token.
        if self.de.peek_token_and_equals(0, &self.closing_token)? {
            return Ok(None);
        }

        if self.de.peek_token(0)?.is_none() {
            return Err(AsonError::UnexpectedEndOfDocument(format!(
                "Incomplete \"{}\".",
                self.type_name
            )));
        }

        // Deserialize the key (the name of a Map entry, or the key of an Object field).
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, AsonError>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.de.consume_colon()?;
        seed.deserialize(&mut *self.de)
    }
}

struct VariantAccessor<'a, CharIter: Iterator<Item = char>> {
    de: &'a mut Deserializer<CharIter>,
    variant_name: String,
}

impl<'a, CharIter: Iterator<Item = char>> VariantAccessor<'a, CharIter> {
    fn new(de: &'a mut Deserializer<CharIter>, variant_name: String) -> Self {
        Self { de, variant_name }
    }
}

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de, CharIter: Iterator<Item = char>> EnumAccess<'de> for VariantAccessor<'_, CharIter> {
    type Error = AsonError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), AsonError>
    where
        V: de::DeserializeSeed<'de>,
    {
        let deserializer: de::value::StrDeserializer<AsonError> =
            self.variant_name.as_str().into_deserializer();
        let value = seed.deserialize(deserializer)?;
        Ok((value, self))
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de, CharIter: Iterator<Item = char>> VariantAccess<'de> for VariantAccessor<'_, CharIter> {
    type Error = AsonError;

    // This accessor is only created for variants followed by `(...)` or `{...}`,
    // so the input does not match a unit variant.
    fn unit_variant(self) -> Result<(), AsonError> {
        Err(AsonError::MessageWithRange(
            format!(
                "The unit variant \"{}\" must not carry a value.",
                self.variant_name
            ),
            self.de.last_range,
        ))
    }

    // Newtype variants are represented in ASON as `(value)`.
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, AsonError>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.de.consume_opening_parenthesis()?; // consume '('
        let value = seed.deserialize(&mut *self.de)?;
        self.de.consume_closing_parenthesis()?; // consume ')'
        Ok(value)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_tuple(self.de, len, visitor)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_struct(self.de, "", fields, visitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        de::{de_from_str, list_stream_reader},
        error::AsonError,
        utf8_char_iterator::UTF8CharIterator,
    };
    use pretty_assertions::assert_eq;
    use serde::Deserialize;
    use serde_bytes::ByteBuf;

    use std::collections::HashMap;

    #[test]
    fn test_primitive_values() {
        // bool
        {
            assert_eq!(de_from_str::<bool>(r#"true"#).unwrap(), true);
            assert_eq!(de_from_str::<bool>(r#"false"#).unwrap(), false);
        }

        // signed integers
        {
            assert_eq!(de_from_str::<i8>(r#"11_i8"#).unwrap(), 11);
            assert_eq!(de_from_str::<i16>(r#"13_i16"#).unwrap(), 13);
            assert_eq!(de_from_str::<i32>(r#"17"#).unwrap(), 17);
            assert_eq!(de_from_str::<i32>(r#"17_i32"#).unwrap(), 17);
            assert_eq!(de_from_str::<i64>(r#"19_i64"#).unwrap(), 19);
        }

        // unsigned integers
        {
            assert_eq!(de_from_str::<u8>(r#"11_u8"#).unwrap(), 11);
            assert_eq!(de_from_str::<u16>(r#"13_u16"#).unwrap(), 13);
            assert_eq!(de_from_str::<u32>(r#"17_u32"#).unwrap(), 17);
            assert_eq!(de_from_str::<u64>(r#"19_u64"#).unwrap(), 19);
        }

        // f32
        {
            assert_eq!(de_from_str::<f32>(r#"123_f32"#).unwrap(), 123_f32);
            assert_eq!(de_from_str::<f32>(r#"-4.56_f32"#).unwrap(), -4.56_f32);
            assert_eq!(
                de_from_str::<f32>(r#"3.1415927_f32"#).unwrap(),
                std::f32::consts::PI
            );
            assert_eq!(de_from_str::<f32>(r#"0_f32"#).unwrap(), 0_f32);
            assert_eq!(de_from_str::<f32>(r#"-0_f32"#).unwrap(), 0_f32); // -0 == 0
            assert!(de_from_str::<f32>(r#"NaN_f32"#).unwrap().is_nan()); // NaN != NaN, use `is_nan()` to check if it's NaN
            assert_eq!(de_from_str::<f32>(r#"Inf_f32"#).unwrap(), f32::INFINITY);
            assert_eq!(
                de_from_str::<f32>(r#"-Inf_f32"#).unwrap(),
                f32::NEG_INFINITY
            );
        }

        // f64
        {
            assert_eq!(de_from_str::<f64>(r#"123.0"#).unwrap(), 123_f64);
            assert_eq!(de_from_str::<f64>(r#"123_f64"#).unwrap(), 123_f64);
            assert_eq!(de_from_str::<f64>(r#"-4.56"#).unwrap(), -4.56_f64);
            assert_eq!(
                de_from_str::<f64>(r#"3.141592653589793"#).unwrap(),
                std::f64::consts::PI
            );
            assert_eq!(de_from_str::<f64>(r#"0_f64"#).unwrap(), 0_f64);
            assert_eq!(de_from_str::<f64>(r#"-0_f64"#).unwrap(), 0_f64); // -0 == 0
            assert!(de_from_str::<f64>(r#"NaN"#).unwrap().is_nan()); // NaN != NaN, use `is_nan()` to check if it's NaN
            assert!(de_from_str::<f64>(r#"NaN_f64"#).unwrap().is_nan()); // NaN != NaN, use `is_nan()` to check if it's NaN
            assert_eq!(de_from_str::<f64>(r#"Inf"#).unwrap(), f64::INFINITY);
            assert_eq!(de_from_str::<f64>(r#"-Inf"#).unwrap(), f64::NEG_INFINITY);
            assert_eq!(de_from_str::<f64>(r#"Inf_f64"#).unwrap(), f64::INFINITY);
            assert_eq!(
                de_from_str::<f64>(r#"-Inf_f64"#).unwrap(),
                f64::NEG_INFINITY
            );
        }

        // char
        {
            assert_eq!(de_from_str::<char>(r#"'a'"#).unwrap(), 'a');
            assert_eq!(de_from_str::<char>(r#"'文'"#).unwrap(), '文');
            assert_eq!(de_from_str::<char>(r#"'🍒'"#).unwrap(), '🍒');

            assert_eq!(de_from_str::<char>(r#"'\\'"#).unwrap(), '\\');
            assert_eq!(de_from_str::<char>(r#"'\''"#).unwrap(), '\'');
            assert_eq!(de_from_str::<char>(r#"'\"'"#).unwrap(), '"');
            assert_eq!(de_from_str::<char>(r#"'\t'"#).unwrap(), '\t');
            assert_eq!(de_from_str::<char>(r#"'\r'"#).unwrap(), '\r');
            assert_eq!(de_from_str::<char>(r#"'\n'"#).unwrap(), '\n');
            assert_eq!(de_from_str::<char>(r#"'\0'"#).unwrap(), '\0');

            assert_eq!(de_from_str::<char>(r#"'\u{8431}'"#).unwrap(), '萱');
        }

        // string
        {
            assert_eq!(
                de_from_str::<String>(r#""abc文字🍒""#).unwrap(),
                "abc文字🍒".to_owned()
            );
            assert_eq!(
                de_from_str::<String>(r#""abc\"\'\\\t\0xyz""#).unwrap(),
                "abc\"\'\\\t\0xyz".to_owned()
            );
            assert_eq!(
                de_from_str::<String>(r#""hello\r\nworld""#).unwrap(),
                "hello\r\nworld".to_owned()
            );
            assert_eq!(
                de_from_str::<String>(r#""\u{5c0f}\u{8431}脚本""#).unwrap(),
                "小萱脚本".to_owned()
            );

            // multi-line string
            assert_eq!(
                de_from_str::<String>("\"a\n b\n  c\"").unwrap(),
                "a\n b\n  c".to_owned()
            );

            // concatenated string
            assert_eq!(
                de_from_str::<String>("\"a\\\n b\\\n  c\"").unwrap(),
                "abc".to_owned()
            );

            // raw string
            assert_eq!(
                de_from_str::<String>(
                    r#"
            r"a\nb"
            "#
                )
                .unwrap(),
                "a\\nb".to_owned()
            );

            // raw string variant
            assert_eq!(
                de_from_str::<String>(
                    r##"
            r#"a\n"&"\nb"#
            "##
                )
                .unwrap(),
                "a\\n\"&\"\\nb".to_owned()
            );

            // auto-trim string
            assert_eq!(
                de_from_str::<String>(
                    r#"
            """
            a
              b
                c
            """
            "#
                )
                .unwrap(),
                "a\n  b\n    c".to_owned()
            );
        }
    }

    #[test]
    fn test_hexadecimal_byte_data() {
        assert_eq!(
            de_from_str::<ByteBuf>(r#"h"0d 0f 11 13""#).unwrap(),
            ByteBuf::from(vec![0x0d, 0x0f, 0x11, 0x13])
        );

        assert_eq!(
            de_from_str::<ByteBuf>(r#"h"61 62 63""#).unwrap(),
            ByteBuf::from(b"abc")
        );
    }

    #[test]
    fn test_enum_option() {
        assert_eq!(de_from_str::<Option<i32>>(r#"Option::None"#).unwrap(), None);
        assert_eq!(
            de_from_str::<Option<i32>>(r#"Option::Some(123)"#).unwrap(),
            Some(123)
        );
    }

    #[test]
    fn test_list() {
        assert_eq!(
            de_from_str::<Vec<i32>>(r#"[11,13,17,19]"#).unwrap(),
            vec![11, 13, 17, 19]
        );

        // trailing comma is allowed
        assert_eq!(
            de_from_str::<Vec<i32>>(r#"[11,13,17,19,]"#).unwrap(),
            vec![11, 13, 17, 19]
        );

        // separate by spaces
        assert_eq!(
            de_from_str::<Vec<i32>>(r#"[11 13 17 19]"#).unwrap(),
            vec![11, 13, 17, 19]
        );

        // separate by new lines
        assert_eq!(
            de_from_str::<Vec<i32>>(
                r#"[
    11
    13
    17
    19
]"#
            )
            .unwrap(),
            vec![11, 13, 17, 19]
        );

        // separate by new lines and commas
        assert_eq!(
            de_from_str::<Vec<i32>>(
                r#"[
    11,
    13,
    17,
    19
]"#
            )
            .unwrap(),
            vec![11, 13, 17, 19]
        );

        // separate by new lines and commas, with trailing comma
        assert_eq!(
            de_from_str::<Vec<i32>>(
                r#"[
    11,
    13,
    17,
    19,
]"#
            )
            .unwrap(),
            vec![11, 13, 17, 19]
        );

        // Vec<u8>
        assert_eq!(
            de_from_str::<Vec<u8>>(
                r#"[
    97_u8
    98_u8
    99_u8
]"#
            )
            .unwrap(),
            b"abc"
        );

        // Vec<String>
        assert_eq!(
            de_from_str::<Vec<String>>(
                r#"[
    "foo"
    "bar"
    "2024"
]"#
            )
            .unwrap(),
            vec!["foo", "bar", "2024"]
        );

        // nested list
        assert_eq!(
            de_from_str::<Vec<Vec<i32>>>(
                r#"[
    [11,13]
    [17,19]
    [23,29]
]"#
            )
            .unwrap(),
            vec![vec![11, 13], vec![17, 19], vec![23, 29]]
        );

        // err: missing ']', EOF
        assert!(matches!(
            de_from_str::<Vec<i32>>(r#"[11,13"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));
    }

    #[test]
    fn test_tuple() {
        assert_eq!(
            de_from_str::<(i32, i32, i32, i32)>(r#"(11, 13, 17, 19)"#).unwrap(),
            (11, 13, 17, 19)
        );

        // trailing comma is allowed
        assert_eq!(
            de_from_str::<(i32, i32, i32, i32)>(r#"(11, 13, 17, 19,)"#).unwrap(),
            (11, 13, 17, 19)
        );

        // separate by spaces
        assert_eq!(
            de_from_str::<(i32, i32, i32, i32)>(r#"(11 13 17 19)"#).unwrap(),
            (11, 13, 17, 19)
        );

        // separate by new lines
        assert_eq!(
            de_from_str::<(i32, i32, i32, i32)>(
                r#"(
    11
    13
    17
    19
)"#
            )
            .unwrap(),
            (11, 13, 17, 19)
        );

        // separate by new lines and commas
        assert_eq!(
            de_from_str::<(i32, i32, i32, i32)>(
                r#"(
    11,
    13,
    17,
    19
)"#
            )
            .unwrap(),
            (11, 13, 17, 19)
        );

        // separate by new lines and commas, with trailing comma
        assert_eq!(
            de_from_str::<(i32, i32, i32, i32)>(
                r#"(
    11,
    13,
    17,
    19,
)"#
            )
            .unwrap(),
            (11, 13, 17, 19)
        );

        // a fixed-length array is treated as tuple
        assert_eq!(
            de_from_str::<[u8; 3]>(
                r#"(
97_u8
98_u8
99_u8
)"#
            )
            .unwrap(),
            b"abc".to_owned()
        );

        // tuple of strings
        assert_eq!(
            de_from_str::<(String, String, String)>(
                r#"(
"foo", "bar", "2024", )"#
            )
            .unwrap(),
            ("foo".to_owned(), "bar".to_owned(), "2024".to_owned())
        );

        // nested tuple
        assert_eq!(
            de_from_str::<((i32, i32), (i32, i32), (i32, i32))>(
                r#"((11, 13), (17, 19), (23, 29))"#
            )
            .unwrap(),
            ((11, 13), (17, 19), (23, 29))
        );

        // err: missing ')', EOF
        assert!(matches!(
            de_from_str::<(i32, i32, i32, i32)>(r#"(11, 13"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));
    }

    #[test]
    fn test_object() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Object {
            id: i32,
            name: String,
            checked: bool,
        }

        assert_eq!(
            de_from_str::<Object>(r#"{id: 123, name: "foo", checked: true}"#).unwrap(),
            Object {
                id: 123,
                name: "foo".to_owned(),
                checked: true
            }
        );

        // trailing comma is allowed
        assert_eq!(
            de_from_str::<Object>(r#"{id: 123, name: "foo", checked: true,}"#).unwrap(),
            Object {
                id: 123,
                name: "foo".to_owned(),
                checked: true
            }
        );

        // separate by spaces
        assert_eq!(
            de_from_str::<Object>(r#"{id: 123 name: "foo" checked: true}"#).unwrap(),
            Object {
                id: 123,
                name: "foo".to_owned(),
                checked: true
            }
        );

        // separate by new lines
        assert_eq!(
            de_from_str::<Object>(
                r#"{
    id: 123
    name: "foo"
    checked: true
}"#
            )
            .unwrap(),
            Object {
                id: 123,
                name: "foo".to_owned(),
                checked: true
            }
        );

        // separate by new lines and commas
        assert_eq!(
            de_from_str::<Object>(
                r#"{
    id: 123,
    name: "foo",
    checked: true
}"#
            )
            .unwrap(),
            Object {
                id: 123,
                name: "foo".to_owned(),
                checked: true
            }
        );

        // separate by new lines and commas, with trailing comma
        assert_eq!(
            de_from_str::<Object>(
                r#"{
    id: 123,
    name: "foo",
    checked: true,
}"#
            )
            .unwrap(),
            Object {
                id: 123,
                name: "foo".to_owned(),
                checked: true
            }
        );

        // nested object
        #[derive(Deserialize, Debug, PartialEq)]
        struct Address {
            code: i32,
            city: String,
        }

        #[derive(Deserialize, Debug, PartialEq)]
        struct NestedObject {
            id: i32,
            name: String,
            address: Box<Address>,
        }

        assert_eq!(
            de_from_str::<NestedObject>(
                r#"{
    id: 456
    name: "bar"
    address: {
        code: 518000
        city: "sz"
    }
}"#
            )
            .unwrap(),
            NestedObject {
                id: 456,
                name: "bar".to_owned(),
                address: Box::new(Address {
                    code: 518000,
                    city: "sz".to_owned()
                })
            }
        );

        // object with missing field
        #[derive(Deserialize, Debug, PartialEq)]
        struct Member {
            name: String,

            #[serde(default)]
            age: u32,
        }

        assert_eq!(
            de_from_str::<Member>(r#"{name: "foo"}"#).unwrap(),
            Member {
                name: "foo".to_owned(),
                age: 0
            }
        );

        // err: missing '}', EOF
        assert!(matches!(
            de_from_str::<Object>(r#"{id: 123"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));
    }

    #[test]
    fn test_map() {
        let s0 = r#"
        [
            "red": 0xff0000
            "green": 0x00ff00
            "blue": 0x0000ff
        ]
        "#;

        let m0: HashMap<String, i32> = de_from_str(s0).unwrap();
        assert_eq!(m0.get("red").unwrap(), &0xff0000);
        assert_eq!(m0.get("green").unwrap(), &0x00ff00);
        assert_eq!(m0.get("blue").unwrap(), &0x0000ff);

        let s1 = r#"
        [
            223: Option::Some("hello")
            227: Option::None
            229: Option::Some("world")
        ]
        "#;

        let m1: HashMap<i32, Option<String>> = de_from_str(s1).unwrap();
        assert_eq!(m1.get(&223).unwrap(), &Option::Some("hello".to_owned()));
        assert_eq!(m1.get(&227).unwrap(), &Option::None);
        assert_eq!(m1.get(&229).unwrap(), &Option::Some("world".to_owned()));
    }

    #[test]
    fn test_enum() {
        #[derive(Deserialize, Debug, PartialEq)]
        enum Color {
            Red,
            Green,
            Blue,
        }

        assert_eq!(de_from_str::<Color>(r#"Color::Red"#).unwrap(), Color::Red);
        assert_eq!(
            de_from_str::<Color>(r#"Color::Green"#).unwrap(),
            Color::Green
        );
        assert_eq!(de_from_str::<Color>(r#"Color::Blue"#).unwrap(), Color::Blue);
    }

    #[test]
    fn test_variant_with_primitive_value() {
        #[derive(Deserialize, Debug, PartialEq)]
        enum Color {
            Red,
            Green,
            Blue,
            Grey(u8),
        }

        assert_eq!(de_from_str::<Color>(r#"Color::Red"#).unwrap(), Color::Red);
        assert_eq!(
            de_from_str::<Color>(r#"Color::Grey(11_u8)"#).unwrap(),
            Color::Grey(11)
        );

        // nested
        #[derive(Deserialize, Debug, PartialEq)]
        enum Apperance {
            Transparent,
            Color(Color),
        }

        assert_eq!(
            de_from_str::<Apperance>(r#"Apperance::Transparent"#).unwrap(),
            Apperance::Transparent
        );

        assert_eq!(
            de_from_str::<Apperance>(r#"Apperance::Color(Color::Blue)"#).unwrap(),
            Apperance::Color(Color::Blue)
        );

        assert_eq!(
            de_from_str::<Apperance>(r#"Apperance::Color(Color::Grey(13_u8))"#).unwrap(),
            Apperance::Color(Color::Grey(13))
        );
    }

    #[test]
    fn test_variant_with_list_value() {
        #[derive(Deserialize, Debug, PartialEq)]
        enum Item {
            Empty,
            List(Vec<i32>),
        }

        assert_eq!(
            de_from_str::<Vec<Item>>(
                r#"[
    Item::Empty
    Item::List([11,13])
]"#
            )
            .unwrap(),
            vec![Item::Empty, Item::List(vec![11, 13]),]
        );
    }

    #[test]
    fn test_variant_with_object_value() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Object {
            id: i32,
            name: String,
        }

        #[derive(Deserialize, Debug, PartialEq)]
        enum Item {
            Empty,
            Object(Object),
        }

        assert_eq!(
            de_from_str::<Vec<Item>>(
                r#"[
    Item::Empty
    Item::Object({
        id: 13
        name: "foo"
    })
]"#
            )
            .unwrap(),
            vec![
                Item::Empty,
                Item::Object(Object {
                    id: 13,
                    name: "foo".to_owned()
                }),
            ]
        );
    }

    #[test]
    fn test_tuple_like_variant() {
        #[allow(clippy::upper_case_acronyms)]
        #[derive(Deserialize, Debug, PartialEq)]
        enum Color {
            Grey(u8),
            RGB(u8, u8, u8),
        }

        assert_eq!(
            de_from_str::<Color>(r#"Color::Grey(127_u8)"#).unwrap(),
            Color::Grey(127)
        );

        assert_eq!(
            de_from_str::<Color>(r#"Color::RGB(255_u8,127_u8,63_u8)"#).unwrap(),
            Color::RGB(255, 127, 63)
        );

        // trailing comma is allowed
        assert_eq!(
            de_from_str::<Color>(r#"Color::RGB(255_u8,127_u8,63_u8,)"#).unwrap(),
            Color::RGB(255, 127, 63)
        );

        // separate by spaces
        assert_eq!(
            de_from_str::<Color>(r#"Color::RGB(255_u8 127_u8 63_u8)"#).unwrap(),
            Color::RGB(255, 127, 63)
        );

        // separate by new lines
        assert_eq!(
            de_from_str::<Color>(
                r#"Color::RGB(
    255_u8
    127_u8
    63_u8
)"#
            )
            .unwrap(),
            Color::RGB(255, 127, 63)
        );

        // separate by new lines and commas
        assert_eq!(
            de_from_str::<Color>(
                r#"Color::RGB(
    255_u8,
    127_u8,
    63_u8
)"#
            )
            .unwrap(),
            Color::RGB(255, 127, 63)
        );

        // separate by new lines and commas, with trailing comma
        assert_eq!(
            de_from_str::<Color>(
                r#"Color::RGB(
    255_u8,
    127_u8,
    63_u8,
)"#
            )
            .unwrap(),
            Color::RGB(255, 127, 63)
        );

        // err: missing ')', EOF
        assert!(matches!(
            de_from_str::<Color>(r#"Color::RGB(255_u8"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));
    }

    #[test]
    fn test_object_like_variant() {
        #[derive(Deserialize, Debug, PartialEq)]
        enum Shape {
            Circle(i32),
            Rect { width: i32, height: i32 },
        }

        assert_eq!(
            de_from_str::<Shape>(r#"Shape::Circle(127)"#).unwrap(),
            Shape::Circle(127)
        );

        assert_eq!(
            de_from_str::<Shape>(
                r#"Shape::Rect{
    width: 200
    height: 100
}"#
            )
            .unwrap(),
            Shape::Rect {
                width: 200,
                height: 100
            }
        );

        // separate by commas
        assert_eq!(
            de_from_str::<Shape>(r#"Shape::Rect{width: 200, height: 100}"#).unwrap(),
            Shape::Rect {
                width: 200,
                height: 100
            }
        );

        // trailing comma is allowed
        assert_eq!(
            de_from_str::<Shape>(r#"Shape::Rect{width: 200, height: 100,}"#).unwrap(),
            Shape::Rect {
                width: 200,
                height: 100
            }
        );

        // separate by new lines and commas
        assert_eq!(
            de_from_str::<Shape>(
                r#"Shape::Rect{
    width: 200,
    height: 100
}"#
            )
            .unwrap(),
            Shape::Rect {
                width: 200,
                height: 100
            }
        );

        // separate by new lines and commas, with trailing comma
        assert_eq!(
            de_from_str::<Shape>(
                r#"Shape::Rect{
    width: 200,
    height: 100,
}"#
            )
            .unwrap(),
            Shape::Rect {
                width: 200,
                height: 100
            }
        );

        // err: missing '}', EOF
        assert!(matches!(
            de_from_str::<Shape>(r#"Shape::Rect{width: 200"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));
    }

    #[test]
    fn test_list_with_tuple_elements() {
        assert_eq!(
            de_from_str::<Vec<(i32, String)>>(
                r#"[
    (1, "foo")
    (2, "bar")
]"#
            )
            .unwrap(),
            vec![(1, "foo".to_owned()), (2, "bar".to_owned())]
        );
    }

    #[test]
    fn test_tuple_with_list_elements() {
        assert_eq!(
            de_from_str::<(Vec<i32>, Vec<String>)>(
                r#"([
    11
    13
], [
    "foo"
    "bar"
])"#
            )
            .unwrap(),
            (vec![11, 13], vec!["foo".to_owned(), "bar".to_owned()])
        );
    }

    #[test]
    fn test_list_with_object_elements() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Object {
            id: i32,
            name: String,
        }

        assert_eq!(
            de_from_str::<Vec<Object>>(
                r#"[
    {
        id: 11
        name: "foo"
    }
    {
        id: 13
        name: "bar"
    }
]"#
            )
            .unwrap(),
            vec![
                Object {
                    id: 11,
                    name: "foo".to_owned()
                },
                Object {
                    id: 13,
                    name: "bar".to_owned()
                }
            ]
        );
    }

    #[test]
    fn test_object_with_list_field() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct ObjectList {
            id: i32,
            items: Vec<i32>,
        }

        assert_eq!(
            de_from_str::<ObjectList>(
                r#"{
    id: 456
    items: [
        11
        13
        17
        19
    ]
}"#
            )
            .unwrap(),
            ObjectList {
                id: 456,
                items: vec![11, 13, 17, 19]
            }
        );
    }

    #[test]
    fn test_tuple_with_object_elements() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Object {
            id: i32,
            name: String,
        }

        assert_eq!(
            de_from_str::<(i32, Object)>(
                r#"(123, {
                id: 11
                name: "foo"
            })"#
            )
            .unwrap(),
            (
                123,
                Object {
                    id: 11,
                    name: "foo".to_owned()
                }
            )
        );
    }

    #[test]
    fn test_object_with_tuple_field() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct ObjectDetail {
            id: i32,
            address: (i32, String),
        }

        assert_eq!(
            de_from_str::<ObjectDetail>(
                r#"{
    id: 456
    address: (11, "sz")
}"#
            )
            .unwrap(),
            ObjectDetail {
                id: 456,
                address: (11, "sz".to_owned())
            }
        );
    }

    #[test]
    fn test_deserialize_stream_list() {
        let s = r#"[11 13]"#;

        let mut chars = s.chars();
        let mut de = list_stream_reader(&mut chars).unwrap();

        assert_eq!(11, de.next().unwrap().unwrap());
        assert_eq!(13, de.next().unwrap().unwrap());
        assert!(de.next().is_none());

        #[derive(Deserialize, Debug, PartialEq)]
        struct Object {
            id: i32,
            name: String,
        }

        let s = r#"[
    {
        id: 11
        name: "foo"
    }
    {
        id: 13
        name: "bar"
    }
]"#;

        let data = s.as_bytes();
        let mut char_iter = UTF8CharIterator::new(data);
        let mut de = list_stream_reader(&mut char_iter).unwrap();

        let o1: Object = de.next().unwrap().unwrap();
        assert_eq!(
            o1,
            Object {
                id: 11,
                name: "foo".to_owned()
            }
        );

        let o2: Object = de.next().unwrap().unwrap();
        assert_eq!(
            o2,
            Object {
                id: 13,
                name: "bar".to_owned()
            }
        );

        assert!(de.next().is_none());

        // An error is returned once, and then the iterator stops.
        let mut chars = r#"[11 "x" 13]"#.chars();
        let mut de = list_stream_reader::<i32, _>(&mut chars).unwrap();
        assert_eq!(11, de.next().unwrap().unwrap());
        assert!(de.next().unwrap().is_err());
        assert!(de.next().is_none());

        // A lexing error is returned rather than ending the iteration silently.
        let mut chars = r#"[11 &]"#.chars();
        let mut de = list_stream_reader::<i32, _>(&mut chars).unwrap();
        assert_eq!(11, de.next().unwrap().unwrap());
        assert!(de.next().unwrap().is_err());
        assert!(de.next().is_none());
    }

    #[test]
    fn test_unknown_object_fields_are_ignored() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Object {
            id: i32,
            name: String,
        }

        assert_eq!(
            de_from_str::<Object>(
                r#"{
    id: 11
    extra: [1, (2, "x"), {a: Option::Some(Color::Rgb(1, 2, 3))}, Shape::Rect{w: 1}]
    name: "foo"
}"#
            )
            .unwrap(),
            Object {
                id: 11,
                name: "foo".to_owned()
            }
        );

        // err: mismatched brackets within the ignored value
        assert!(de_from_str::<Object>(r#"{id: 11, extra: [1, 2), name: "foo"}"#).is_err());

        #[derive(Deserialize, Debug)]
        #[serde(deny_unknown_fields)]
        #[allow(dead_code)]
        struct StrictObject {
            id: i32,
        }

        // err: unknown field
        assert!(de_from_str::<StrictObject>(r#"{id: 11, extra: 13}"#).is_err());
    }

    #[test]
    fn test_invalid_values() {
        #[allow(clippy::upper_case_acronyms)]
        #[derive(Deserialize, Debug, PartialEq)]
        enum Color {
            Red,
            RGB(u8, u8, u8),
        }

        // err: a unit variant with a value
        assert!(de_from_str::<Color>(r#"Color::Red(1)"#).is_err());

        // err: a tuple-like variant without values
        assert!(de_from_str::<Color>(r#"Color::RGB()"#).is_err());

        // err: wrong enumeration type
        assert!(de_from_str::<Color>(r#"Colour::Red"#).is_err());

        // err: too few elements
        assert!(de_from_str::<(i32, i32, i32)>(r#"(1, 2)"#).is_err());

        // err: too many elements
        assert!(de_from_str::<(i32, i32)>(r#"(1, 2, 3)"#).is_err());

        // err: empty tuple
        assert!(de_from_str::<[i32; 0]>(r#"()"#).is_err());

        // err: the error of the inner value is reported, rather than the missing ')'
        assert!(matches!(
            de_from_str::<Option<i32>>(r#"Option::Some("x")"#),
            Err(AsonError::MessageWithRange(_, range)) if range.start.index == 13
        ));

        // err: extraneous token
        assert!(de_from_str::<i32>(r#"11 13"#).is_err());

        // err: quoted object key
        #[derive(Deserialize, Debug)]
        #[allow(dead_code)]
        struct Object {
            id: i32,
        }

        assert!(de_from_str::<Object>(r#"{"id": 11}"#).is_err());
    }
}
