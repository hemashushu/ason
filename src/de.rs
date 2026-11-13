// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::{collections::HashSet, io::Read, marker::PhantomData};

use crate::{
    char_with_position::CharsWithPositionIterator,
    date_time,
    error::AsonError,
    lexer::Lexer,
    normalizer::NormalizeSignedNumberIter,
    peekable_iterator::PeekableIterator,
    range::Range,
    token::{NumberToken, Token, TokenWithRange},
    utf8_char_iterator::UTF8CharIterator,
    value::{Value, VariantValue, validate_predefined_enumeration},
};
use serde::de::{self, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess};

/// Deserializes one ASON document from a string into a known Serde type.
///
/// `T` must own its deserialized data; it cannot borrow strings or bytes from
/// `s`. ASON numeric types must match the requested Rust types. For a document
/// whose structure is not known in advance, use [`de_value_from_str`] instead.
///
/// After the root value, only whitespace, commas, and comments are accepted.
///
/// # Errors
///
/// Returns an error for invalid ASON, a value incompatible with `T`, an
/// unsupported Serde representation, or a token after the root value.
///
/// # Examples
///
/// ```
/// use serde::Deserialize;
///
/// #[derive(Debug, Deserialize, PartialEq)]
/// struct Person {
///     name: String,
///     age: u8,
/// }
///
/// let person: Person = ason::de_from_str(r#"{ name: "Ada" age: 18_u8 }"#)?;
/// assert_eq!(person, Person { name: "Ada".into(), age: 18 });
/// # Ok::<(), ason::AsonError>(())
/// ```
pub fn de_from_str<T>(s: &str) -> Result<T, AsonError>
where
    T: de::DeserializeOwned,
{
    de_from_char_iterator(s.chars().map(Ok::<char, AsonError>))
}

/// Deserializes one UTF-8 ASON document from a reader into a known Serde type.
///
/// The reader is buffered and decoded as UTF-8. As with [`de_from_str`], `T`
/// must own its data and the document must match its requested types. The
/// function checks for EOF after the root value, so it can block on a live
/// reader that remains open. Use [`list_stream_reader`] for a root List that
/// should be processed incrementally.
///
/// # Errors
///
/// Returns an error for an I/O or UTF-8 failure, invalid ASON, a value
/// incompatible with `T`, or a token after the root value.
///
/// # Examples
///
/// ```
/// let input = "[1 2 3]".as_bytes();
/// let values: Vec<i32> = ason::de_from_reader(input)?;
/// assert_eq!(values, vec![1, 2, 3]);
/// # Ok::<(), ason::AsonError>(())
/// ```
pub fn de_from_reader<T, R: Read>(reader: R) -> Result<T, AsonError>
where
    T: de::DeserializeOwned,
{
    de_from_char_iterator(UTF8CharIterator::new(reader))
}

/// Parses one ASON document into an owned, dynamically typed [`Value`] tree.
///
/// Unlike [`de_from_str`], this does not deserialize into a caller-supplied
/// Serde type. Numeric types and compound value kinds are represented by
/// separate variants of [`Value`]. The ambiguous empty syntax `[]` is parsed
/// as an empty List, not an empty Map. Comments and original formatting are
/// not retained.
///
/// # Errors
///
/// Returns an error for invalid ASON, an invalid predefined enumeration, or a
/// token after the root value.
///
/// # Examples
///
/// ```
/// use ason::Value;
///
/// let value = ason::de_value_from_str(r#"{ name: "Ada" age: 18_u8 }"#)?;
/// assert_eq!(value, Value::Object(vec![
///     ("name".into(), Value::String("Ada".into())),
///     ("age".into(), Value::U8(18)),
/// ]));
/// # Ok::<(), ason::AsonError>(())
/// ```
pub fn de_value_from_str(s: &str) -> Result<Value, AsonError> {
    de_value_from_char_iterator(s.chars().map(Ok::<char, AsonError>))
}

/// Parses one UTF-8 ASON document from a reader into an owned [`Value`] tree.
///
/// This is the reader counterpart of [`de_value_from_str`]; no target Serde
/// type is required. The reader is buffered, and checking for EOF after the
/// root value can block if a live source stays open.
///
/// # Errors
///
/// Returns an error for an I/O or UTF-8 failure, invalid ASON, or a token
/// after the root value.
///
/// # Examples
///
/// ```
/// let value = ason::de_value_from_reader("42".as_bytes())?;
/// assert_eq!(value, ason::Value::I32(42));
/// # Ok::<(), ason::AsonError>(())
/// ```
pub fn de_value_from_reader<R: Read>(reader: R) -> Result<Value, AsonError> {
    de_value_from_char_iterator(UTF8CharIterator::new(reader))
}

fn de_value_from_char_iterator<CharIter>(char_iterator: CharIter) -> Result<Value, AsonError>
where
    CharIter: Iterator<Item = Result<char, AsonError>>,
{
    let mut deserializer = Deserializer::from_char_iterator(char_iterator);
    let value = deserializer.deserialize_value()?;
    match deserializer.next_token()? {
        Some(_) => Err(AsonError::MessageWithRange(
            "Unexpected token after the end of the document.".to_owned(),
            deserializer.last_range,
        )),
        None => Ok(value),
    }
}

fn de_from_char_iterator<T, CharIter>(char_iterator: CharIter) -> Result<T, AsonError>
where
    T: de::DeserializeOwned,
    CharIter: Iterator<Item = Result<char, AsonError>>,
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

/// Opens a root ASON List for incremental deserialization.
///
/// The opening `[` is consumed when this function is called. The returned
/// [`ListStreamReader`] yields each element as `Result<T, AsonError>` and
/// validates the closing `]` only when iteration reaches it. Stopping early
/// leaves the remainder unvalidated.
///
/// The stream does not check for trailing tokens or require EOF after `]`.
/// Buffering and lookahead can read past the List or block waiting for more
/// input; buffered trailing bytes are not returned to the caller.
///
/// # Errors
///
/// Returns an error if the opening List cannot be read. Errors in later
/// elements or the closing delimiter are returned by the iterator.
///
/// # Examples
///
/// ```
/// let stream = ason::list_stream_reader::<i32, _>("[1 2 3]".as_bytes())?;
/// let values = stream.collect::<Result<Vec<_>, _>>()?;
/// assert_eq!(values, vec![1, 2, 3]);
/// # Ok::<(), ason::AsonError>(())
/// ```
pub fn list_stream_reader<T, R>(reader: R) -> Result<ListStreamReader<T, R>, AsonError>
where
    T: de::DeserializeOwned,
    R: Read,
{
    let mut deserializer = Deserializer::from_char_iterator(UTF8CharIterator::new(reader));

    // consume the opening bracket '['
    deserializer.consume_token_and_assert(&Token::BracketOpen, "a \"List\"")?;

    Ok(ListStreamReader::new(deserializer))
}

/// Reads an ASON Result containing a streamed List or an error value.
///
/// Accepts `Ok([...])` and `Err(...)`, as well as their `Result::`-qualified
/// forms. The `Ok` branch yields List elements of type `T` incrementally; the
/// `Err` payload is fully deserialized as `E` before this function returns.
/// An `Err(E)` branch is a successfully decoded business error, not an
/// [`AsonError`].
///
/// For `Ok`, the closing `]` and `)` are validated only when the iterator is
/// exhausted. Neither branch checks trailing tokens or requires EOF, but
/// buffering and lookahead may read beyond the Result or block for more input.
/// Buffered trailing bytes are not exposed for reading another document.
///
/// # Errors
///
/// Returns an error if the Result header, error payload, or opening List
/// cannot be deserialized. Later List and closing-delimiter errors are
/// returned by the `Ok` branch's iterator.
///
/// # Examples
///
/// ```
/// let result = ason::result_stream_reader::<i32, String, _>("Ok([1 2])".as_bytes())?;
/// let ason::ResultStreamReader::Ok(items) = result else {
///     panic!("expected an Ok branch");
/// };
/// assert_eq!(items.collect::<Result<Vec<_>, _>>()?, vec![1, 2]);
/// # Ok::<(), ason::AsonError>(())
/// ```
pub fn result_stream_reader<T, E, R>(reader: R) -> Result<ResultStreamReader<T, E, R>, AsonError>
where
    T: de::DeserializeOwned,
    E: de::DeserializeOwned,
    R: Read,
{
    let mut deserializer = Deserializer::from_char_iterator(UTF8CharIterator::new(reader));
    let variant = match deserializer.next_token()? {
        Some(Token::Enumeration(name, variant)) if name == "Result" => variant,
        other => {
            return Err(
                deserializer.expect_error(other.is_some(), "an enumeration of type \"Result\"")
            );
        }
    };

    if variant != "Ok" && variant != "Err" {
        return Err(AsonError::MessageWithRange(
            format!(
                "Unknown Result variant \"{}\"; expect \"Ok\" or \"Err\".",
                variant
            ),
            deserializer.last_range,
        ));
    }
    deserializer.consume_opening_parenthesis()?;

    if variant == "Err" {
        let error = E::deserialize(&mut deserializer)?;
        deserializer.consume_closing_parenthesis()?;
        Ok(ResultStreamReader::Err(error))
    } else {
        deserializer.consume_token_and_assert(&Token::BracketOpen, "a \"List\"")?;
        let mut stream = ListStreamReader::new(deserializer);
        stream.wrapped_result = true;
        Ok(ResultStreamReader::Ok(Box::new(stream)))
    }
}

/// A successfully decoded Result with either a streamed List or an error value.
///
/// Parsing and I/O failures are returned separately as [`AsonError`] by
/// [`result_stream_reader`] or by the `Ok` iterator.
pub enum ResultStreamReader<T, E, R>
where
    T: de::DeserializeOwned,
    R: Read,
{
    /// A List whose elements and closing delimiters are read during iteration.
    Ok(Box<ListStreamReader<T, R>>),
    /// The fully deserialized error payload of the ASON Result.
    Err(E),
}

/// An iterator that deserializes a List one element at a time.
///
/// Its items are `Result<T, AsonError>`. A parse or I/O error is yielded once,
/// after which the iterator returns `None`. The closing `]`, and the closing
/// `)` when this List is inside a Result, are checked only upon reaching the
/// end. Trailing tokens are not checked, and the buffered reader may read
/// beyond those delimiters.
///
/// Construct this iterator with [`list_stream_reader`] or
/// [`result_stream_reader`].
pub struct ListStreamReader<T, R>
where
    T: de::DeserializeOwned,
    R: Read,
{
    deserializer: Deserializer<UTF8CharIterator<R>>,

    // Set when the closing bracket ']' has been consumed or an error has occurred,
    // after which the iterator only returns `None`.
    finished: bool,

    wrapped_result: bool,

    _marker: PhantomData<fn() -> T>,
}

impl<T, R> ListStreamReader<T, R>
where
    T: de::DeserializeOwned,
    R: Read,
{
    fn new(deserializer: Deserializer<UTF8CharIterator<R>>) -> Self {
        Self {
            deserializer,
            finished: false,
            wrapped_result: false,
            _marker: PhantomData,
        }
    }
}

impl<T, R> Iterator for ListStreamReader<T, R>
where
    T: de::DeserializeOwned,
    R: Read,
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
                if self.wrapped_result {
                    return self
                        .deserializer
                        .consume_closing_parenthesis()
                        .err()
                        .map(Err);
                }
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

/// The Serde deserializer backed by ASON's normalized token stream.
///
/// The public entry points construct this type internally. Serde operations
/// that require `deserialize_any` are not supported; use the `de_value_*`
/// functions when the value's type is not known in advance.
pub struct Deserializer<CharIter: Iterator<Item = Result<char, AsonError>>> {
    upstream: PeekableIterator<Result<TokenWithRange, AsonError>, UpstreamIterator<CharIter>>,

    /// The range of the last token consumed by `next_token()`.
    last_range: Range,
}

impl<CharIter: Iterator<Item = Result<char, AsonError>>> Deserializer<CharIter> {
    fn from_char_iterator(char_iterator: CharIter) -> Self {
        // The processing pipeline:
        // chars -> chars with positions -> tokens -> tokens with signed numbers normalized
        let char_position_iter = CharsWithPositionIterator::new(char_iterator);
        let lexer = Lexer::new(PeekableIterator::new(char_position_iter), false);
        let normalizer = NormalizeSignedNumberIter::new(PeekableIterator::new(lexer));

        Self {
            upstream: PeekableIterator::new(normalizer),
            last_range: Range::default(),
        }
    }

    fn deserialize_value(&mut self) -> Result<Value, AsonError> {
        match self.next_token()? {
            Some(Token::Number(number)) => Ok(match number {
                NumberToken::I8(value) => Value::I8(value as i8),
                NumberToken::I16(value) => Value::I16(value as i16),
                NumberToken::I32(value) => Value::I32(value as i32),
                NumberToken::I64(value) => Value::I64(value as i64),
                NumberToken::U8(value) => Value::U8(value),
                NumberToken::U16(value) => Value::U16(value),
                NumberToken::U32(value) => Value::U32(value),
                NumberToken::U64(value) => Value::U64(value),
                NumberToken::F32(value) => Value::F32(value),
                NumberToken::F64(value) => Value::F64(value),
            }),
            Some(Token::Boolean(value)) => Ok(Value::Bool(value)),
            Some(Token::Char(value)) => Ok(Value::Char(value)),
            Some(Token::String(value)) => Ok(Value::String(value)),
            Some(Token::DateTime(value)) => Ok(Value::DateTime(value)),
            Some(Token::HexadecimalByteData(value)) => Ok(Value::Bytes(value)),
            Some(Token::BracketOpen) => self.deserialize_list_or_map(),
            Some(Token::ParenthesisOpen) => {
                let values = self.deserialize_values_until(Token::ParenthesisClose, "Tuple")?;
                if values.is_empty() {
                    return Err(AsonError::MessageWithRange(
                        "A \"Tuple\" must contain at least one element.".to_owned(),
                        self.last_range,
                    ));
                }
                Ok(Value::Tuple(values))
            }
            Some(Token::BraceOpen) => self.deserialize_object(),
            Some(Token::Enumeration(name, variant)) => self.deserialize_enumeration(name, variant),
            other => Err(self.expect_error(other.is_some(), "a value")),
        }
    }

    fn deserialize_list_or_map(&mut self) -> Result<Value, AsonError> {
        if self.peek_token_and_equals(0, &Token::BracketClose)? {
            self.next_token()?;
            return Ok(Value::List(Vec::new()));
        }

        let first = self.deserialize_value()?;
        if self.peek_token_and_equals(0, &Token::Colon)? {
            self.consume_colon()?;
            let mut entries = vec![(first, self.deserialize_value()?)];
            while !self.peek_token_and_equals(0, &Token::BracketClose)? {
                if self.peek_token(0)?.is_none() {
                    return Err(AsonError::UnexpectedEndOfDocument(
                        "Incomplete Map.".to_owned(),
                    ));
                }
                let key = self.deserialize_value()?;
                self.consume_colon()?;
                entries.push((key, self.deserialize_value()?));
            }
            self.consume_closing_bracket()?;
            Ok(Value::Map(entries))
        } else {
            let mut values = vec![first];
            while !self.peek_token_and_equals(0, &Token::BracketClose)? {
                if self.peek_token(0)?.is_none() {
                    return Err(AsonError::UnexpectedEndOfDocument(
                        "Incomplete List.".to_owned(),
                    ));
                }
                values.push(self.deserialize_value()?);
            }
            self.consume_closing_bracket()?;
            Ok(Value::List(values))
        }
    }

    fn deserialize_values_until(
        &mut self,
        closing_token: Token,
        type_name: &str,
    ) -> Result<Vec<Value>, AsonError> {
        let mut values = Vec::new();
        while !self.peek_token_and_equals(0, &closing_token)? {
            if self.peek_token(0)?.is_none() {
                return Err(AsonError::UnexpectedEndOfDocument(format!(
                    "Incomplete {type_name}."
                )));
            }
            values.push(self.deserialize_value()?);
        }
        self.next_token()?;
        Ok(values)
    }

    fn deserialize_object(&mut self) -> Result<Value, AsonError> {
        let mut fields = Vec::new();
        let mut names = HashSet::new();
        while !self.peek_token_and_equals(0, &Token::BraceClose)? {
            if self.peek_token(0)?.is_none() {
                return Err(AsonError::UnexpectedEndOfDocument(
                    "Incomplete Object.".to_owned(),
                ));
            }
            let name = match self.next_token()? {
                Some(Token::Identifier(name)) => name,
                other => return Err(self.expect_error(other.is_some(), "an Object key")),
            };
            if !names.insert(name.clone()) {
                return Err(AsonError::MessageWithRange(
                    format!("Duplicate key \"{}\" in Object.", name),
                    self.last_range,
                ));
            }
            self.consume_colon()?;
            fields.push((name, self.deserialize_value()?));
        }
        self.consume_closing_brace()?;
        Ok(Value::Object(fields))
    }

    fn deserialize_enumeration(
        &mut self,
        name: String,
        variant: String,
    ) -> Result<Value, AsonError> {
        let payload = if self.peek_token_and_equals(0, &Token::ParenthesisOpen)? {
            self.next_token()?;
            let values = self.deserialize_values_until(Token::ParenthesisClose, "Enumeration")?;
            match values.len() {
                0 => {
                    return Err(AsonError::MessageWithRange(
                        "An enumeration payload must not be empty.".to_owned(),
                        self.last_range,
                    ));
                }
                1 => VariantValue::Newtype(Box::new(values.into_iter().next().unwrap())),
                _ => VariantValue::Tuple(values),
            }
        } else if self.peek_token_and_equals(0, &Token::BraceOpen)? {
            self.next_token()?;
            let Value::Object(fields) = self.deserialize_object()? else {
                unreachable!()
            };
            VariantValue::Struct(fields)
        } else {
            VariantValue::Unit
        };

        validate_predefined_enumeration(&name, &variant, payload.kind())
            .map_err(|message| AsonError::MessageWithRange(message, self.last_range))?;

        if name == "Option" {
            return Ok(Value::Option(match payload {
                VariantValue::Newtype(value) => Some(value),
                _ => None,
            }));
        }

        Ok(Value::Enumeration {
            name,
            variant,
            payload,
        })
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

    fn peek_token_with_range(&self, offset: usize) -> Result<Option<&TokenWithRange>, AsonError> {
        match self.upstream.peek(offset) {
            Some(Ok(token_with_range)) => Ok(Some(token_with_range)),
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
            Some(Token::BracketOpen) => self.skip_bracket_contents(),
            Some(Token::ParenthesisOpen) => self.skip_sequence(Token::ParenthesisClose, true),
            Some(Token::BraceOpen) => self.skip_object_contents(),
            Some(Token::Enumeration(..)) => {
                // A variant may carry a value in parentheses or braces.
                if self.peek_token_and_equals(0, &Token::ParenthesisOpen)? {
                    self.next_token()?;
                    self.skip_sequence(Token::ParenthesisClose, true)
                } else if self.peek_token_and_equals(0, &Token::BraceOpen)? {
                    self.next_token()?;
                    self.skip_object_contents()
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

    fn skip_sequence(
        &mut self,
        closing_token: Token,
        require_element: bool,
    ) -> Result<(), AsonError> {
        let mut element_count = 0;

        loop {
            if self.peek_token_and_equals(0, &closing_token)? {
                if require_element && element_count == 0 {
                    return Err(self.expect_error(true, "at least one value"));
                }
                self.next_token()?;
                return Ok(());
            }

            if self.peek_token(0)?.is_none() {
                return Err(AsonError::UnexpectedEndOfDocument(
                    "Incomplete value.".to_owned(),
                ));
            }

            self.skip_value()?;
            element_count += 1;
        }
    }

    fn skip_bracket_contents(&mut self) -> Result<(), AsonError> {
        if self.peek_token_and_equals(0, &Token::BracketClose)? {
            self.next_token()?;
            return Ok(());
        }

        if self.peek_token(0)?.is_none() {
            return Err(AsonError::UnexpectedEndOfDocument(
                "Incomplete value.".to_owned(),
            ));
        }

        self.skip_value()?;

        if self.peek_token_and_equals(0, &Token::Colon)? {
            loop {
                self.consume_colon()?;
                self.skip_value()?;

                if self.peek_token_and_equals(0, &Token::BracketClose)? {
                    self.next_token()?;
                    return Ok(());
                }

                if self.peek_token(0)?.is_none() {
                    return Err(AsonError::UnexpectedEndOfDocument(
                        "Incomplete Map.".to_owned(),
                    ));
                }

                self.skip_value()?;
            }
        }

        loop {
            if self.peek_token_and_equals(0, &Token::BracketClose)? {
                self.next_token()?;
                return Ok(());
            }

            if self.peek_token(0)?.is_none() {
                return Err(AsonError::UnexpectedEndOfDocument(
                    "Incomplete List.".to_owned(),
                ));
            }

            self.skip_value()?;
        }
    }

    fn skip_object_contents(&mut self) -> Result<(), AsonError> {
        let mut keys = HashSet::new();

        loop {
            if self.peek_token_and_equals(0, &Token::BraceClose)? {
                self.next_token()?;
                return Ok(());
            }

            if self.peek_token(0)?.is_none() {
                return Err(AsonError::UnexpectedEndOfDocument(
                    "Incomplete Object.".to_owned(),
                ));
            }

            let key = match self.next_token()? {
                Some(Token::Identifier(key)) => key,
                other => return Err(self.expect_error(other.is_some(), "an Object key")),
            };

            if !keys.insert(key.clone()) {
                return Err(AsonError::MessageWithRange(
                    format!("Duplicate key \"{}\" in Object.", key),
                    self.last_range,
                ));
            }

            self.consume_colon()?;
            self.skip_value()?;
        }
    }
}

impl<'de, CharIter: Iterator<Item = Result<char, AsonError>>> de::Deserializer<'de>
    for &mut Deserializer<CharIter>
{
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
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, AsonError>
    where
        V: de::Visitor<'de>,
    {
        if name == date_time::MARKER {
            return match self.next_token()? {
                Some(Token::DateTime(value)) => {
                    visitor.visit_newtype_struct(de::value::StringDeserializer::<AsonError>::new(
                        value.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                    ))
                }
                other => Err(self.expect_error(other.is_some(), "a \"DateTime\" value")),
            };
        }
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
struct SeqAccessor<'a, CharIter: Iterator<Item = Result<char, AsonError>>> {
    de: &'a mut Deserializer<CharIter>,
    closing_token: Token,
    type_name: &'static str,
}

impl<'a, CharIter: Iterator<Item = Result<char, AsonError>>> SeqAccessor<'a, CharIter> {
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

impl<'de, CharIter: Iterator<Item = Result<char, AsonError>>> SeqAccess<'de>
    for SeqAccessor<'_, CharIter>
{
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
struct MapAccessor<'a, CharIter: Iterator<Item = Result<char, AsonError>>> {
    de: &'a mut Deserializer<CharIter>,
    closing_token: Token,
    type_name: &'static str,
    seen_object_keys: HashSet<String>,
}

impl<'a, CharIter: Iterator<Item = Result<char, AsonError>>> MapAccessor<'a, CharIter> {
    fn new(
        de: &'a mut Deserializer<CharIter>,
        closing_token: Token,
        type_name: &'static str,
    ) -> Self {
        Self {
            de,
            closing_token,
            type_name,
            seen_object_keys: HashSet::new(),
        }
    }
}

impl<'de, CharIter: Iterator<Item = Result<char, AsonError>>> MapAccess<'de>
    for MapAccessor<'_, CharIter>
{
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

        if self.type_name == "Object"
            && let Some(TokenWithRange {
                token: Token::Identifier(key),
                range,
            }) = self.de.peek_token_with_range(0)?
            && !self.seen_object_keys.insert(key.clone())
        {
            return Err(AsonError::MessageWithRange(
                format!("Duplicate key \"{}\" in Object.", key),
                *range,
            ));
        }

        // Deserialize the key of a Map entry or Object field.
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

struct VariantAccessor<'a, CharIter: Iterator<Item = Result<char, AsonError>>> {
    de: &'a mut Deserializer<CharIter>,
    variant_name: String,
}

impl<'a, CharIter: Iterator<Item = Result<char, AsonError>>> VariantAccessor<'a, CharIter> {
    fn new(de: &'a mut Deserializer<CharIter>, variant_name: String) -> Self {
        Self { de, variant_name }
    }
}

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de, CharIter: Iterator<Item = Result<char, AsonError>>> EnumAccess<'de>
    for VariantAccessor<'_, CharIter>
{
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
impl<'de, CharIter: Iterator<Item = Result<char, AsonError>>> VariantAccess<'de>
    for VariantAccessor<'_, CharIter>
{
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
    use std::io::{self, Read};

    use crate::{
        de::{de_from_reader, de_from_str, de_value_from_str, list_stream_reader},
        error::AsonError,
        value::{Value, VariantValue},
    };
    use pretty_assertions::assert_eq;
    use serde::Deserialize;
    use serde_bytes::ByteBuf;

    use std::collections::HashMap;

    struct FailingReader {
        bytes: &'static [u8],
    }

    #[test]
    fn test_dynamic_value_deserialization() {
        assert_eq!(de_value_from_str("-12_i16").unwrap(), Value::I16(-12));
        assert_eq!(de_value_from_str("2.5_f32").unwrap(), Value::F32(2.5));
        assert_eq!(
            de_value_from_str(r#"[1 "two"]"#).unwrap(),
            Value::List(vec![Value::I32(1), Value::String("two".into())])
        );
        assert_eq!(
            de_value_from_str(r#"["one": 1 "two": 2]"#).unwrap(),
            Value::Map(vec![
                (Value::String("one".into()), Value::I32(1)),
                (Value::String("two".into()), Value::I32(2)),
            ])
        );
        assert_eq!(
            de_value_from_str("{name: \"Ada\"}").unwrap(),
            Value::Object(vec![("name".into(), Value::String("Ada".into()))])
        );
        assert_eq!(
            de_value_from_str("(1 \"two\")").unwrap(),
            Value::Tuple(vec![Value::I32(1), Value::String("two".into())])
        );
        assert_eq!(
            de_value_from_str("Option::None").unwrap(),
            Value::Option(None)
        );
        assert_eq!(
            de_value_from_str("Option::Some(1)").unwrap(),
            Value::Option(Some(Box::new(Value::I32(1))))
        );
        assert_eq!(
            de_value_from_str("Action::Take(\"item\" 42)").unwrap(),
            Value::Enumeration {
                name: "Action".into(),
                variant: "Take".into(),
                payload: VariantValue::Tuple(vec![Value::String("item".into()), Value::I32(42),]),
            }
        );
        assert!(de_value_from_str("Option::Some(1) true").is_err());

        for text in [
            "None(1)",
            "Some",
            "Some(1, 2)",
            "Some{a: 1}",
            "Option::Foo",
            "Ok",
            "Ok(1, 2)",
            "Err",
            "Err{a: 1}",
            "Result::Foo(1)",
            "Result::ok(1)",
        ] {
            assert!(de_value_from_str(text).is_err(), "{text}");
        }
        assert!(de_value_from_str("Ok(1)").is_ok());
        assert!(de_value_from_str("Err(\"x\")").is_ok());
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

    #[test]
    fn test_reader_errors_propagate_through_deserializers() {
        let error = de_from_reader::<Vec<i32>, _>(FailingReader { bytes: b"[1 " }).unwrap_err();
        assert!(matches!(
            error,
            AsonError::Message(message) if message.contains("test read failure")
        ));

        let mut reader = list_stream_reader::<i32, _>(FailingReader { bytes: b"[1 " }).unwrap();
        assert_eq!(reader.next().unwrap().unwrap(), 1);
        assert!(matches!(
            reader.next().unwrap(),
            Err(AsonError::Message(message)) if message.contains("test read failure")
        ));
        assert!(reader.next().is_none());
    }

    #[test]
    fn test_result_stream_reader_io_errors() {
        for bytes in [b"Result::".as_slice(), b"Result::Err(\"failed\" "] {
            let error = match super::result_stream_reader::<i32, String, _>(FailingReader { bytes })
            {
                Err(error) => error,
                Ok(_) => panic!("expected a read error"),
            };
            assert!(
                matches!(error, AsonError::Message(message) if message.contains("test read failure"))
            );
        }

        for bytes in [b"Result::Ok([1 ".as_slice(), b"Result::Ok([1] "] {
            let super::ResultStreamReader::Ok(mut stream) =
                super::result_stream_reader::<i32, String, _>(FailingReader { bytes }).unwrap()
            else {
                panic!("expected a streamed List");
            };
            assert_eq!(stream.next().unwrap().unwrap(), 1);
            assert!(
                matches!(stream.next().unwrap(), Err(AsonError::Message(message)) if message.contains("test read failure"))
            );
            assert!(stream.next().is_none());
        }
    }

    #[test]
    fn test_asonb_input_has_friendly_error() {
        let input = b"\0ASONB\0\x01";
        let assert_asonb_error = |error: AsonError| {
            assert!(matches!(
                error,
                AsonError::MessageWithPosition(message, position)
                    if message.contains("ASONB binary data") && position.index == 0
            ));
        };

        assert_asonb_error(de_from_reader::<i32, _>(&input[..]).unwrap_err());

        let error = match list_stream_reader::<i32, _>(&input[..]) {
            Err(error) => error,
            Ok(_) => panic!("ASONB data should not be accepted as ASON text"),
        };
        assert_asonb_error(error);
    }

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
        assert_eq!(de_from_str::<ByteBuf>(r#"h"""#).unwrap(), ByteBuf::new());
        assert_eq!(
            de_from_str::<ByteBuf>(r#"h"00 FF""#).unwrap(),
            ByteBuf::from(vec![0x00, 0xff]),
        );
        for text in ["", r#""abc""#, r#"h"0""#, r#"h"gg""#, r#"h"00" true"#] {
            assert!(de_from_str::<ByteBuf>(text).is_err(), "{text}");
        }
        assert!(de_from_str::<Vec<u8>>(r#"h"00 ff""#).is_err());
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
    fn test_enum_shorthand() {
        for (short, full, expected) in [
            ("None", "Option::None", None),
            ("Some(123)", "Option::Some(123)", Some(123)),
        ] {
            assert_eq!(de_from_str::<Option<i32>>(short).unwrap(), expected);
            assert_eq!(
                de_value_from_str(short).unwrap(),
                de_value_from_str(full).unwrap()
            );
        }

        for (short, full, expected) in [
            ("Ok(123)", "Result::Ok(123)", Ok(123)),
            (
                r#"Err("failed")"#,
                r#"Result::Err("failed")"#,
                Err("failed".to_owned()),
            ),
        ] {
            assert_eq!(de_from_str::<Result<i32, String>>(short).unwrap(), expected);
            assert_eq!(
                de_value_from_str(short).unwrap(),
                de_value_from_str(full).unwrap()
            );
        }

        let nested = "[Some(Ok(Some(123))), Some(Err(None)), None]";
        let expected = vec![Some(Ok(Some(123))), Some(Err(None)), None];
        type Nested = Vec<Option<Result<Option<i32>, Option<String>>>>;
        assert_eq!(de_from_str::<Nested>(nested).unwrap(), expected);
        assert_eq!(
            de_from_reader::<Nested, _>(nested.as_bytes()).unwrap(),
            expected
        );

        for invalid in [
            "some(1)",
            "SOME(1)",
            "none",
            "NONE",
            "Some",
            "Some()",
            "Some(1, 2)",
            "None(1)",
        ] {
            assert!(de_from_str::<Option<i32>>(invalid).is_err(), "{invalid}");
            assert!(de_value_from_str(invalid).is_err(), "{invalid}");
        }
        for invalid in [
            "ok(1)",
            "OK(1)",
            "err(2)",
            "ERR(2)",
            "Ok",
            "Ok()",
            "Ok(1, 2)",
            "Err",
            "Err()",
            "Err(1, 2)",
        ] {
            assert!(
                de_from_str::<Result<i32, i32>>(invalid).is_err(),
                "{invalid}"
            );
        }
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
    fn test_invalid_utf8_propagates_through_deserializer() {
        let error = de_from_reader::<String, _>(&b"\"x\xff\""[..]).unwrap_err();
        assert!(matches!(
            error,
            AsonError::Message(message) if message.contains("Invalid UTF-8")
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
    fn test_object_rejects_reserved_words_as_keys() {
        #[derive(Deserialize, Debug, Default, PartialEq)]
        struct EmptyObject {}

        assert_eq!(de_from_str::<EmptyObject>("{}").unwrap(), EmptyObject {});
        assert_eq!(
            de_from_str::<EmptyObject>("{ordinary: 1}").unwrap(),
            EmptyObject {}
        );

        for key in [
            "true", "false", "NaN", "NaN_f32", "NaN_f64", "Inf", "+Inf", "-Inf", "Inf_f32",
            "+Inf_f32", "-Inf_f32", "Inf_f64", "+Inf_f64", "-Inf_f64", "Some", "None", "Ok", "Err",
        ] {
            let document = format!("{{{key}: 1}}");
            assert!(
                de_from_str::<EmptyObject>(&document).is_err(),
                "accepted reserved Object key {key}"
            );
        }
    }

    #[test]
    fn test_object_rejects_duplicate_unknown_keys() {
        #[derive(Deserialize, Debug, Default, PartialEq)]
        struct EmptyObject {}

        assert!(de_from_str::<EmptyObject>("{unknown: 1, unknown: 2}").is_err());
        assert!(de_from_str::<EmptyObject>("{unknown: {nested: 1, nested: 2}}").is_err());
        assert!(de_from_str::<EmptyObject>("{unknown: [{nested: 1, nested: 2}]}").is_err());
        assert_eq!(
            de_from_str::<EmptyObject>("{unknown: [\"first\": 1, \"second\": 2]}").unwrap(),
            EmptyObject {}
        );

        #[allow(dead_code)]
        #[derive(Deserialize, Debug)]
        enum EnumObject {
            Variant { value: i32 },
        }

        assert!(
            de_from_str::<EnumObject>("EnumObject::Variant{value: 0, extra: 1, extra: 2}").is_err()
        );
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

        let mut de = list_stream_reader(s.as_bytes()).unwrap();

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
        let mut de = list_stream_reader(data).unwrap();

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
        let mut de = list_stream_reader::<i32, _>(r#"[11 "x" 13]"#.as_bytes()).unwrap();
        assert_eq!(11, de.next().unwrap().unwrap());
        assert!(de.next().unwrap().is_err());
        assert!(de.next().is_none());

        // A lexing error is returned rather than ending the iteration silently.
        let mut de = list_stream_reader::<i32, _>(r#"[11 &]"#.as_bytes()).unwrap();
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
