// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::io::Read;

use crate::{
    char_with_position::CharsWithPositionIterator,
    error::AsonError,
    lexer::{Lexer, PEEK_BUFFER_LENGTH_LEX},
    normalizer::{NormalizeSignedNumberIter, PEEK_BUFFER_LENGTH_NORMALIZE},
    parser::PEEK_BUFFER_LENGTH_PARSE,
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
    let mut chars = s.chars();
    de_from_char_iterator(&mut chars)
}

pub fn de_from_reader<T, R: Read>(reader: R) -> Result<T, AsonError>
where
    T: de::DeserializeOwned,
{
    let mut char_iter = UTF8CharIterator::new(reader);
    de_from_char_iterator(&mut char_iter)
}

pub fn de_from_char_iterator<T>(
    char_iterator: &mut dyn Iterator<Item = char>,
) -> Result<T, AsonError>
where
    T: de::DeserializeOwned,
{
    // There are two main ways to write Deserialize trait bounds, see:
    // https://serde.rs/lifetimes.html

    let char_position_iter = CharsWithPositionIterator::new(char_iterator);

    // Lex
    let peekable_char_position_iter =
        PeekableIterator::new(char_position_iter, PEEK_BUFFER_LENGTH_LEX);
    let lexer = Lexer::new(peekable_char_position_iter);

    // Normalize signed numbers
    let peekable_lexer_iter = PeekableIterator::new(lexer, PEEK_BUFFER_LENGTH_NORMALIZE);
    let normalizer_iter = NormalizeSignedNumberIter::new(peekable_lexer_iter);

    // Deserialize
    let peekable_token_stream_iter =
        PeekableIterator::new(normalizer_iter, PEEK_BUFFER_LENGTH_PARSE);

    let mut deserializer = Deserializer::new(peekable_token_stream_iter);

    let value = T::deserialize(&mut deserializer)?;

    // // Check extraneous tokens
    // match deserializer.next_token()? {
    //     Some(_) => Err(AsonError::MessageWithRange(
    //         "Extraneous token found after document end.".to_owned(),
    //         deserializer.last_range,
    //     )),
    //     None => Ok(value),
    // }

    Ok(value)
}

type UpstreamIterator<'a> =
    NormalizeSignedNumberIter<Lexer<CharsWithPositionIterator<&'a mut dyn Iterator<Item = char>>>>;

pub struct Deserializer<'a> {
    upstream: PeekableIterator<Result<TokenWithRange, AsonError>, UpstreamIterator<'a>>,

    /// The range of the last consumed token by `next_token` or `next_token_with_range`.
    last_range: Range,
}

impl<'a> Deserializer<'a> {
    fn new(
        upstream: PeekableIterator<Result<TokenWithRange, AsonError>, UpstreamIterator<'a>>,
    ) -> Self {
        Self {
            upstream,
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

    #[allow(dead_code)]
    fn peek_range(&self, offset: usize) -> Result<Option<&Range>, AsonError> {
        match self.upstream.peek(offset) {
            Some(Ok(TokenWithRange { range, .. })) => Ok(Some(range)),
            Some(Err(e)) => Err(e.clone()),
            None => Ok(None),
        }
    }

    // Peek the next token and check if it equals to the expected token,
    // return false if not equals or no more token,
    // error if lexing error occurs
    fn peek_token_and_equals(
        &self,
        offset: usize,
        expected_token: &Token,
    ) -> Result<bool, AsonError> {
        Ok(matches!(
            self.peek_token(offset)?,
            Some(token) if token == expected_token))
    }

    // Consume the next token and assert it equals to the expected token,
    // error if not equal or no more token
    fn consume_token_and_assert(
        &mut self,
        expected_token: &Token,
        token_description: &str,
    ) -> Result<(), AsonError> {
        match self.next_token()? {
            Some(token) => {
                if &token == expected_token {
                    Ok(())
                } else {
                    Err(AsonError::MessageWithRange(
                        format!("Expect token: {}.", token_description),
                        self.last_range,
                    ))
                }
            }
            None => Err(AsonError::UnexpectedEndOfDocument(format!(
                "Expect token: {}.",
                token_description
            ))),
        }
    }

    // Consume '(', error if the next token is not '(' or no more token
    fn consume_opening_parenthesis(&mut self) -> Result<(), AsonError> {
        self.consume_token_and_assert(&Token::OpeningParenthesis, "opening parenthesis")
    }

    // Consume ')', error if the next token is not ')' or no more token
    fn consume_closing_parenthesis(&mut self) -> Result<(), AsonError> {
        self.consume_token_and_assert(&Token::ClosingParenthesis, "closing parenthesis")
    }

    // Consume ']', error if the next token is not ']' or no more token
    fn consume_closing_bracket(&mut self) -> Result<(), AsonError> {
        self.consume_token_and_assert(&Token::ClosingBracket, "closing bracket")
    }

    // Consume '}', error if the next token is not '}' or no more token
    fn consume_closing_brace(&mut self) -> Result<(), AsonError> {
        self.consume_token_and_assert(&Token::ClosingBrace, "closing brace")
    }

    // Consume ':', error if the next token is not ':' or no more token
    fn consume_colon(&mut self) -> Result<(), AsonError> {
        self.consume_token_and_assert(&Token::Colon, "colon")
    }
}

impl<'de> de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = AsonError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        Err(AsonError::MessageWithRange(
            "Unexpected value.".to_owned(),
            self.last_range,
        ))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Boolean(v)) => visitor.visit_bool(v),
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect a \"Boolean\" value.".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect a \"Boolean\" value.".to_owned(),
            )),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::I8(v))) => visitor.visit_i8(v as i8),
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect an \"i8\" value.".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect an \"i8\" value.".to_owned(),
            )),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::I16(v))) => visitor.visit_i16(v as i16),
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect an \"i16\" value.".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect an \"i16\" value.".to_owned(),
            )),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::I32(v))) => visitor.visit_i32(v as i32),
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect an \"i32\" value.".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect an \"i32\" value.".to_owned(),
            )),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::I64(v))) => visitor.visit_i64(v as i64),
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect an \"i64\" value.".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect an \"i64\" value.".to_owned(),
            )),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::U8(v))) => visitor.visit_u8(v),
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect an \"u8\" value.".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect an \"u8\" value.".to_owned(),
            )),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::U16(v))) => visitor.visit_u16(v),
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect an \"u16\" value.".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect an \"u16\" value.".to_owned(),
            )),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::U32(v))) => visitor.visit_u32(v),
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect an \"u32\" value.".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect an \"u32\" value.".to_owned(),
            )),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::U64(v))) => visitor.visit_u64(v),
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect an \"u64\" value.".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect an \"u64\" value.".to_owned(),
            )),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::F32(v))) => visitor.visit_f32(v),
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect a \"f32\" value.".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect a \"f32\" value.".to_owned(),
            )),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Number(NumberToken::F64(v))) => visitor.visit_f64(v),
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect a \"f64\" value.".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect a \"f64\" value.".to_owned(),
            )),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Char(c)) => visitor.visit_char(c),
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect a \"Char\" value.".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect a \"Char\" value.".to_owned(),
            )),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::String(s)) => visitor.visit_str(&s),
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect a \"String\" value.".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect a \"String\" value.".to_owned(),
            )),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::String(s)) => visitor.visit_string(s),
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect a \"String\" value.".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect a \"String\" value.".to_owned(),
            )),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::HexadecimalByteData(d)) => visitor.visit_bytes(&d),
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect \"Hexadecimal Byte Data\".".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect \"Hexadecimal Byte Data\".".to_owned(),
            )),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::HexadecimalByteData(d)) => visitor.visit_byte_buf(d),
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect \"Hexadecimal Byte Data\".".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect \"Hexadecimal Byte Data\".".to_owned(),
            )),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::Enumeration(type_name, variant_name)) => {
                if type_name == "Option" {
                    if variant_name == "None"
                        && !self.peek_token_and_equals(0, &Token::OpeningParenthesis)?
                    {
                        visitor.visit_none()
                    } else if variant_name == "Some"
                        && self.peek_token_and_equals(0, &Token::OpeningParenthesis)?
                    {
                        self.next_token()?; // consume '('
                        let v = visitor.visit_some(&mut *self);
                        self.consume_closing_parenthesis()?;
                        v
                    } else {
                        Err(AsonError::MessageWithRange(
                            "Expect \"None\" or \"Some\" variant of enum \"Option\".".to_owned(),
                            self.last_range,
                        ))
                    }
                } else {
                    Err(AsonError::MessageWithRange(
                        "Expect the enum \"Option\".".to_owned(),
                        self.last_range,
                    ))
                }
            }
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect the enum \"Option\".".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect the enum \"Option\".".to_owned(),
            )),
        }
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // The type of `()` in Rust.
        Err(AsonError::Message("Does not support Unit.".to_owned()))
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // A unit struct is a struct that has no fields, for example `struct Unit;`.
        Err(AsonError::Message(
            "Does not support \"Unit\" style Struct.".to_owned(),
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
            "Does not support \"New-Type\" style Struct.".to_owned(),
        ))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // seq = List/Vector

        match self.next_token()? {
            Some(Token::OpeningBracket) => {
                let value = visitor.visit_seq(ArrayAccessor::new(self))?;
                self.consume_closing_bracket()?; // consume ']'
                Ok(value)
            }
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect a \"List\".".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect a \"List\".".to_owned(),
            )),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        match self.next_token()? {
            Some(Token::OpeningParenthesis) => {
                let value = visitor.visit_seq(TupleAccessor::new(self))?;
                self.consume_closing_parenthesis()?; // consume ')'
                Ok(value)
            }
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect a \"Tuple\".".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect a \"Tuple\".".to_owned(),
            )),
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
            "Does not support \"Tuple\" style Struct.".to_owned(),
        ))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // map = Named List
        match self.next_token()? {
            Some(Token::OpeningBracket) => {
                let value = visitor.visit_map(MapAccessor::new(self))?;
                self.consume_closing_bracket()?; // consume ']'

                Ok(value)
            }
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect a \"Named List\".".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect a \"Named List\".".to_owned(),
            )),
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
            Some(Token::OpeningBrace) => {
                let value = visitor.visit_map(ObjectAccessor::new(self))?;
                self.consume_closing_brace()?; // consume '}'

                Ok(value)
            }
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect an \"Object\".".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect an \"Object\".".to_owned(),
            )),
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
            Some(Token::Enumeration(type_name, variant_name)) => {
                if type_name == name {
                    if self.peek_token_and_equals(0, &Token::OpeningParenthesis)? {
                        // variant with single value or tuple-like variant
                        let v = visitor.visit_enum(VariantAccessor::new(self, &variant_name))?;
                        Ok(v)
                    } else if self.peek_token_and_equals(0, &Token::OpeningBrace)? {
                        // object-like variant
                        let v = visitor.visit_enum(VariantAccessor::new(self, &variant_name))?;
                        Ok(v)
                    } else {
                        // variant without value
                        visitor.visit_enum(variant_name.into_deserializer())
                    }
                } else {
                    Err(AsonError::MessageWithRange(
                        format!("Expect an \"Enum\" \"{}\".", name,),
                        self.last_range,
                    ))
                }
            }
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect an \"Enum\".".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect an \"Enum\".".to_owned(),
            )),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        // An identifier in Serde is the type that identifies a field of a struct.
        match self.next_token()? {
            Some(Token::Identifier(id)) => visitor.visit_string(id),
            // Some(Token::String(id)) => visitor.visit_string(id),
            Some(_) => Err(AsonError::MessageWithRange(
                "Expect an identifier.".to_owned(),
                self.last_range,
            )),
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Expect an identifier.".to_owned(),
            )),
        }
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        unreachable!()
    }
}

// List/Vector Accessor
struct ArrayAccessor<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> ArrayAccessor<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self {
            de,
        }
    }
}

impl<'de> SeqAccess<'de> for ArrayAccessor<'_, 'de> {
    type Error = AsonError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, AsonError>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.de.peek_token_and_equals(0, &Token::ClosingBracket)? {
            // exits the procedure when the end marker ']' is encountered.
            return Ok(None);
        }

        if self.de.peek_token(0)?.is_none() {
            return Err(AsonError::UnexpectedEndOfDocument(
                "Incomplete List.".to_owned(),
            ));
        }

        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct TupleAccessor<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> TupleAccessor<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self {
            de,
        }
    }
}

impl<'de> SeqAccess<'de> for TupleAccessor<'_, 'de> {
    type Error = AsonError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, AsonError>
    where
        T: de::DeserializeSeed<'de>,
    {
        // the deserializer knows the number of members of the
        // target tuple, so it doesn't need to check the
        // ending marker ')'.

        if self.de.peek_token(0)?.is_none() {
            return Err(AsonError::UnexpectedEndOfDocument(
                "Incomplete Tuple.".to_owned(),
            ));
        }

        seed.deserialize(&mut *self.de).map(Some)
    }
}

// Map/Named List Accessor
struct MapAccessor<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> MapAccessor<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self {
            de,
        }
    }
}

impl<'de> MapAccess<'de> for MapAccessor<'_, 'de> {
    type Error = AsonError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, AsonError>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.de.peek_token_and_equals(0, &Token::ClosingBracket)? {
            return Ok(None);
        }

        if self.de.peek_token(0)?.is_none() {
            return Err(AsonError::UnexpectedEndOfDocument(
                "Incomplete List.".to_owned(),
            ));
        }

        // Deserialize a field name.
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, AsonError>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.de.consume_colon()?;

        // Deserialize a field value.
        seed.deserialize(&mut *self.de)
    }
}

struct ObjectAccessor<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> ObjectAccessor<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self {
            de,
        }
    }
}

impl<'de> MapAccess<'de> for ObjectAccessor<'_, 'de> {
    type Error = AsonError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, AsonError>
    where
        K: de::DeserializeSeed<'de>,
    {
        // the MapAccess wouldn't stop automatically when it encounters the last item.
        if self.de.peek_token_and_equals(0, &Token::ClosingBrace)? {
            return Ok(None);
        }

        if self.de.peek_token(0)?.is_none() {
            return Err(AsonError::UnexpectedEndOfDocument(
                "Incomplete Object.".to_owned(),
            ));
        }

        // Deserialize a field key.
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, AsonError>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.de.consume_colon()?;

        // Deserialize a field value.
        seed.deserialize(&mut *self.de)
    }
}

struct VariantAccessor<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    variant_name: &'a str,
}

impl<'a, 'de> VariantAccessor<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, variant_name: &'a str) -> Self {
        Self { de, variant_name }
    }
}

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de> EnumAccess<'de> for VariantAccessor<'_, 'de> {
    type Error = AsonError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), AsonError>
    where
        V: de::DeserializeSeed<'de>,
    {
        let de: de::value::StrDeserializer<AsonError> = self.variant_name.into_deserializer();
        let value = seed.deserialize(de)?;
        Ok((value, self))
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de> VariantAccess<'de> for VariantAccessor<'_, 'de> {
    type Error = AsonError;

    // If the `Visitor` expected this variant to be a unit variant, the input
    // should have been the plain string case handled in `deserialize_enum`.
    fn unit_variant(self) -> Result<(), AsonError> {
        unreachable!()
    }

    // Newtype variants are represented in ASON as `(value)` so
    // deserialize the value here.
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, AsonError>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.de.consume_opening_parenthesis()?; // consume '('
        let v = seed.deserialize(&mut *self.de);
        self.de.consume_closing_parenthesis()?; // consume ')'
        v
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
    use crate::{de::de_from_str, error::AsonError};
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
    fn test_named_list() {
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
}
