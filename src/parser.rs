// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::io::Read;

use crate::{
    ast::{AsonNode, Enumeration, KeyValuePair, NamedListEntry, Number},
    char_with_position::CharsWithPositionIterator,
    error::AsonError,
    lexer::Lexer,
    normalizer::NormalizeSignedNumberIter,
    peekable_iterator::PeekableIterator,
    range::Range,
    token::{NumberToken, Token, TokenWithRange},
    utf8_char_iterator::UTF8CharIterator,
};

pub fn parse_from_str(s: &str) -> Result<AsonNode, AsonError> {
    let chars = s.chars();
    parse_from_char_iterator(chars)
}

pub fn parse_from_reader<R>(reader: R) -> Result<AsonNode, AsonError>
where
    R: Read,
{
    let char_iter = UTF8CharIterator::new(reader);
    parse_from_char_iterator(char_iter)
}

pub fn parse_from_char_iterator<T>(char_iterator: T) -> Result<AsonNode, AsonError>
where
    T: Iterator<Item = char>,
{
    let char_position_iter = CharsWithPositionIterator::new(char_iterator);

    // Lex
    let peekable_char_position_iter = PeekableIterator::new(char_position_iter);
    let lexer = Lexer::new(peekable_char_position_iter);

    // Normalize signed numbers
    let peekable_lexer_iter = PeekableIterator::new(lexer);
    let normalizer_iter = NormalizeSignedNumberIter::new(peekable_lexer_iter);

    // Parse
    let peekable_token_stream_iter = PeekableIterator::new(normalizer_iter);
    let mut parser = Parser::new(peekable_token_stream_iter);
    let root = parser.parse_node()?;

    // Check extraneous tokens
    match parser.next_token()? {
        Some(_) => Err(AsonError::MessageWithRange(
            "Extraneous token found after document end.".to_owned(),
            parser.last_range,
        )),
        None => Ok(root),
    }
}

struct Parser<T>
where
    T: Iterator<Item = Result<TokenWithRange, AsonError>>,
{
    upstream: PeekableIterator<Result<TokenWithRange, AsonError>, T>,

    /// The range of the last consumed token by `next_token` or `next_token_with_range`.
    last_range: Range,
}

impl<T> Parser<T>
where
    T: Iterator<Item = Result<TokenWithRange, AsonError>>,
{
    fn new(upstream: PeekableIterator<Result<TokenWithRange, AsonError>, T>) -> Self {
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

    fn peek_range(&self, offset: usize) -> Result<Option<&Range>, AsonError> {
        match self.upstream.peek(offset) {
            Some(Ok(TokenWithRange { range, .. })) => Ok(Some(range)),
            Some(Err(e)) => Err(e.clone()),
            None => Ok(None),
        }
    }

    // Peek the next token and check if it equals to the expected token,
    // return false if not equals or no more token,
    // error if lexing error occurs during peeking.
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

    // Consume ')', error if the next token is not ')' or no more token
    fn consume_closing_parenthesis(&mut self) -> Result<(), AsonError> {
        self.consume_token_and_assert(&Token::ParenthesisClose, "closing parenthesis")
    }

    // Consume ']', error if the next token is not ']' or no more token
    fn consume_closing_bracket(&mut self) -> Result<(), AsonError> {
        self.consume_token_and_assert(&Token::BracketClose, "closing bracket")
    }

    // Consume '}', error if the next token is not '}' or no more token
    fn consume_closing_brace(&mut self) -> Result<(), AsonError> {
        self.consume_token_and_assert(&Token::BraceClose, "closing brace")
    }

    // Consume ':', error if the next token is not ':' or no more token
    fn consume_colon(&mut self) -> Result<(), AsonError> {
        self.consume_token_and_assert(&Token::Colon, "colon")
    }
}

impl<T> Parser<T>
where
    T: Iterator<Item = Result<TokenWithRange, AsonError>>,
{
    fn parse_node(&mut self) -> Result<AsonNode, AsonError> {
        match self.peek_token(0)? {
            Some(current_token) => {
                let node = match current_token {
                    Token::Number(n) => {
                        let v = convert_number_token(n);
                        self.next_token()?;
                        v
                    }
                    Token::Boolean(b) => {
                        let v = AsonNode::Boolean(*b);
                        self.next_token()?;
                        v
                    }
                    Token::Char(c) => {
                        let v = AsonNode::Char(*c);
                        self.next_token()?;
                        v
                    }
                    Token::String(s) => {
                        let v = AsonNode::String(s.to_owned());
                        self.next_token()?;
                        v
                    }
                    Token::DateTime(d) => {
                        let v = AsonNode::DateTime(*d);
                        self.next_token()?;
                        v
                    }
                    Token::Enumeration(type_name, variant_name) => {
                        match self.peek_token(1)? {
                            Some(Token::ParenthesisOpen) => {
                                // tuple-like variant or the single value variant
                                self.parse_tuple_like_variant()?
                            }
                            Some(Token::BraceOpen) => {
                                // object-like variant
                                self.parse_object_like_variant()?
                            }
                            _ => {
                                // unit variant (that is, without value)
                                let v = AsonNode::Enumeration(Enumeration::new(
                                    type_name,
                                    variant_name,
                                ));
                                self.next_token()?;
                                v
                            }
                        }
                    }
                    Token::HexadecimalByteData(b) => {
                        let v = AsonNode::HexadecimalByteData(b.to_owned());
                        self.next_token()?;
                        v
                    }
                    Token::BraceOpen => {
                        // object: {key:value, ...}
                        self.parse_object()?
                    }
                    Token::BracketOpen => {
                        // list: [...]
                        // named-list (map): ["name":value, ...]
                        self.parse_list()?
                    }
                    Token::ParenthesisOpen => {
                        // tuple: (...)
                        self.parse_tuple()?
                    }
                    _ => {
                        return Err(AsonError::MessageWithRange(
                            "Unexpected token.".to_owned(),
                            *self.peek_range(0)?.unwrap(),
                        ));
                    }
                };

                Ok(node)
            }
            None => Err(AsonError::UnexpectedEndOfDocument(
                "Incomplete document.".to_owned(),
            )),
        }
    }

    /// Parse:
    /// - tuple-like variant: `type::variant(..., ..., ...)`
    /// - single value variant: `type::variant(value)`
    fn parse_tuple_like_variant(&mut self) -> Result<AsonNode, AsonError> {
        // ```diagram
        // type::variant(...)?  //
        // ^            ^    ^__// to here
        // |            |-------// opening parenthesis, validated
        // |--------------------// current token, validated
        // ```

        // consume variant token
        let (type_name, variant_name) =
            if let Some(Token::Enumeration(type_name, variant_name)) = self.next_token()? {
                (type_name, variant_name)
            } else {
                unreachable!()
            };

        self.next_token()?; // consume '('
        // self.consume_new_line_if_exist()?; // consume new-line if exists

        let mut items = vec![];

        // Collect items
        while let Some(token) = self.peek_token(0)? {
            if token == &Token::ParenthesisClose {
                break;
            }

            let value = self.parse_node()?;
            items.push(value);

            // let found_sep = self.consume_new_line_or_comma_if_exist()?;
            // if !found_sep {
            //     break;
            // }
        }

        self.consume_closing_parenthesis()?; // consume ')'

        let variant_item = match items.len() {
            0 => {
                return Err(AsonError::MessageWithRange(
                    "Empty value variant is not allowed, add one value or remove the parentheses."
                        .to_owned(),
                    self.last_range,
                ));
            }
            1 => Enumeration::with_value(&type_name, &variant_name, items.remove(0)),
            _ => Enumeration::with_tuple_like(&type_name, &variant_name, items),
        };

        Ok(AsonNode::Enumeration(variant_item))
    }

    fn parse_object_like_variant(&mut self) -> Result<AsonNode, AsonError> {
        // ```diagram
        // type::variant{...}?  //
        // ^            ^    ^__// to here
        // |            |_______// opening brace, validated
        // |--------------------// current token, validated
        // ```

        // consume variant token
        let Some(Token::Enumeration(type_name, variant_name)) = self.next_token()? else {
            unreachable!()
        };

        let kvps = self.parse_key_value_pairs()?;

        Ok(AsonNode::Enumeration(Enumeration::with_object_like(
            &type_name,
            &variant_name,
            kvps,
        )))
    }

    fn parse_object(&mut self) -> Result<AsonNode, AsonError> {
        let kvps = self.parse_key_value_pairs()?;
        Ok(AsonNode::Object(kvps))
    }

    fn parse_key_value_pairs(&mut self) -> Result<Vec<KeyValuePair>, AsonError> {
        // ```diagram
        // {key: value, ...}?  //
        // ^                ^__// to here
        // |-------------------// current token, validated
        // ```

        self.next_token()?; // consume '{'

        let mut kvps: Vec<KeyValuePair> = vec![];

        // Collect key-value pairs
        while let Some(token) = self.peek_token(0)? {
            if token == &Token::BraceClose {
                break;
            }

            let name = match self.next_token()? {
                Some(Token::Identifier(n)) => n,
                Some(_) => {
                    return Err(AsonError::MessageWithRange(
                        "Expect an identifier as the key name.".to_owned(),
                        self.last_range,
                    ));
                }
                None => {
                    return Err(AsonError::UnexpectedEndOfDocument(
                        "Expect an identifier as the key name.".to_owned(),
                    ));
                }
            };

            self.consume_colon()?;

            let value = self.parse_node()?;
            let name_value_pair = KeyValuePair {
                key: name,
                value: Box::new(value),
            };
            kvps.push(name_value_pair);
        }

        self.consume_closing_brace()?; // consume '}'

        Ok(kvps)
    }

    /// Parse:
    /// - list: `[value, value, ...]`
    /// - named-list (map): `[name: value, name: value, ...]`
    fn parse_list(&mut self) -> Result<AsonNode, AsonError> {
        // ```diagram
        // [...]?  //
        // ^    ^__// to here
        // |-------// current token, validated
        // ```

        self.next_token()?; // consume '['

        let mut list_items: Vec<AsonNode> = vec![];
        let mut named_list_entries: Vec<NamedListEntry> = vec![];

        #[derive(PartialEq)]
        enum ListType {
            Unspecified,
            List,
            NamedList,
        }

        let mut list_type = ListType::Unspecified;

        while let Some(token) = self.peek_token(0)? {
            if token == &Token::BracketClose {
                break;
            }

            let item = self.parse_node()?;

            if list_type == ListType::Unspecified {
                if self.peek_token_and_equals(0, &Token::Colon)? {
                    list_type = ListType::NamedList
                } else {
                    list_type = ListType::List
                }
            }

            if list_type == ListType::List {
                // Regular list
                list_items.push(item);
            } else {
                // Named list

                self.consume_colon()?;

                let value = self.parse_node()?;
                let entry = NamedListEntry {
                    name: Box::new(item),
                    value: Box::new(value),
                };
                named_list_entries.push(entry);
            }
        }

        // self.next_token()?; // consume ']'
        self.consume_closing_bracket()?; // consume ']'

        if list_type == ListType::List {
            Ok(AsonNode::List(list_items))
        } else {
            Ok(AsonNode::NamedList(named_list_entries))
        }
    }

    fn parse_tuple(&mut self) -> Result<AsonNode, AsonError> {
        // ```diagram
        // (...)?  //
        // ^    ^__// to here
        // |-------// current token, validated
        // ```

        self.next_token()?; // consume '('

        let mut items: Vec<AsonNode> = vec![];

        while let Some(token) = self.peek_token(0)? {
            if token == &Token::ParenthesisClose {
                break;
            }

            let value = self.parse_node()?;
            items.push(value);
        }

        self.consume_closing_parenthesis()?; // consume ')'

        if items.is_empty() {
            Err(AsonError::MessageWithRange(
                "Empty tuple is not allowed.".to_owned(),
                self.last_range,
            ))
        } else {
            Ok(AsonNode::Tuple(items))
        }
    }
}

fn convert_number_token(token: &NumberToken) -> AsonNode {
    let number = match token {
        NumberToken::I8(v) => Number::I8(*v as i8),
        NumberToken::U8(v) => Number::U8(*v),
        NumberToken::I16(v) => Number::I16(*v as i16),
        NumberToken::U16(v) => Number::U16(*v),
        NumberToken::I32(v) => Number::I32(*v as i32),
        NumberToken::U32(v) => Number::U32(*v),
        NumberToken::I64(v) => Number::I64(*v as i64),
        NumberToken::U64(v) => Number::U64(*v),
        NumberToken::F32(v) => Number::F32(*v),
        NumberToken::F64(v) => Number::F64(*v),
    };

    AsonNode::Number(number)
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use pretty_assertions::assert_eq;

    use crate::{
        ast::{Enumeration, KeyValuePair, NamedListEntry, Number},
        error::AsonError,
        parser::parse_from_str,
        position::Position,
        range::Range,
    };

    use super::AsonNode;

    #[test]
    fn test_parse_primitive_value() {
        assert_eq!(
            parse_from_str(
                r#"
            123
            "#
            )
            .unwrap(),
            AsonNode::Number(Number::I32(123))
        );

        assert_eq!(
            parse_from_str(
                r#"
            true
            "#
            )
            .unwrap(),
            AsonNode::Boolean(true)
        );

        assert_eq!(
            parse_from_str(
                r#"
            '🍒'
            "#
            )
            .unwrap(),
            AsonNode::Char('🍒')
        );

        assert_eq!(
            parse_from_str(
                r#"
            "hello"
            "#
            )
            .unwrap(),
            AsonNode::String("hello".to_owned())
        );
    }

    #[test]
    fn test_parse_datetime() {
        assert_eq!(
            parse_from_str(
                r#"
            d"2024-03-17 10:01:11+08:00"
            "#
            )
            .unwrap(),
            AsonNode::DateTime(DateTime::parse_from_rfc3339("2024-03-17 10:01:11+08:00").unwrap())
        );
    }

    #[test]
    fn test_parse_hexadecimal_byte_data() {
        assert_eq!(
            parse_from_str(
                r#"
            h"11 13 17 19"
            "#
            )
            .unwrap(),
            AsonNode::HexadecimalByteData(vec![0x11u8, 0x13, 0x17, 0x19])
        );
    }

    #[test]
    fn test_parse_object() {
        let expect_object1 = AsonNode::Object(vec![
            KeyValuePair {
                key: "id".to_owned(),
                value: Box::new(AsonNode::Number(Number::I32(123))),
            },
            KeyValuePair {
                key: "name".to_owned(),
                value: Box::new(AsonNode::String("foo".to_owned())),
            },
        ]);

        assert_eq!(
            parse_from_str(
                r#"
            {id:123,name:"foo"}
            "#
            )
            .unwrap(),
            expect_object1
        );

        // without comma (separate by space)
        assert_eq!(
            parse_from_str(
                r#"
            {id:123 name:"foo"}
            "#
            )
            .unwrap(),
            expect_object1
        );

        // with new-line
        assert_eq!(
            parse_from_str(
                r#"
            {
                id:123
                name:"foo"
            }
            "#
            )
            .unwrap(),
            expect_object1
        );

        // with comma
        assert_eq!(
            parse_from_str(
                r#"
            {
                id:123,
                name:"foo"
            }
            "#
            )
            .unwrap(),
            expect_object1
        );

        // with tailing comma
        assert_eq!(
            parse_from_str(
                r#"
            {
                id: 123,
                name: "foo",
            }
            "#
            )
            .unwrap(),
            expect_object1
        );

        // nested object
        assert_eq!(
            parse_from_str(
                r#"
            {
                id: 123
                addr: {
                    city: "ShenZhen"
                    street: Option::None
                }
            }
            "#
            )
            .unwrap(),
            AsonNode::Object(vec![
                KeyValuePair {
                    key: "id".to_owned(),
                    value: Box::new(AsonNode::Number(Number::I32(123))),
                },
                KeyValuePair {
                    key: "addr".to_owned(),
                    value: Box::new(AsonNode::Object(vec![
                        KeyValuePair {
                            key: "city".to_owned(),
                            value: Box::new(AsonNode::String("ShenZhen".to_owned())),
                        },
                        KeyValuePair {
                            key: "street".to_owned(),
                            value: Box::new(AsonNode::Enumeration(Enumeration::new(
                                "Option", "None"
                            ))),
                        },
                    ])),
                },
            ])
        );

        // err: invalid key (should not be enclosed with quotes)
        assert!(matches!(
            parse_from_str(r#"{"id": 123}"#),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 1,
                        line: 0,
                        column: 1
                    },
                    end_inclusive: Position {
                        index: 4,
                        line: 0,
                        column: 4
                    }
                }
            ))
        ));

        // err: invalid key (should be identifier)
        assert!(matches!(
            parse_from_str(r#"{123}"#),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 1,
                        line: 0,
                        column: 1
                    },
                    end_inclusive: Position {
                        index: 3,
                        line: 0,
                        column: 3
                    }
                }
            ))
        ));

        // err: missing ':'
        assert!(matches!(
            parse_from_str(r#"{id}"#),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 3,
                        line: 0,
                        column: 3
                    },
                    end_inclusive: Position {
                        index: 3,
                        line: 0,
                        column: 3
                    }
                }
            ))
        ));

        // err: missing value
        assert!(matches!(
            parse_from_str(r#"{id:}"#),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 4,
                        line: 0,
                        column: 4
                    },
                    end_inclusive: Position {
                        index: 4,
                        line: 0,
                        column: 4
                    }
                }
            ))
        ));

        // err: missing `:`, but EOF
        assert!(matches!(
            parse_from_str(r#"{id"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: missing a value, but EOF
        assert!(matches!(
            parse_from_str(r#"{id:"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: missing '}', but EOF
        assert!(matches!(
            parse_from_str(r#"{id:123"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));
    }

    #[test]
    fn test_parse_list() {
        let expect_list1 = AsonNode::List(vec![
            AsonNode::Number(Number::I32(123)),
            AsonNode::Number(Number::I32(456)),
            AsonNode::Number(Number::I32(789)),
        ]);

        assert_eq!(
            parse_from_str(
                r#"
            [123,456,789]
            "#
            )
            .unwrap(),
            expect_list1
        );

        assert_eq!(
            parse_from_str(
                r#"
            [123 456 789]
            "#
            )
            .unwrap(),
            expect_list1
        );

        assert_eq!(
            parse_from_str(
                r#"
            [
                123
                456
                789
            ]
            "#
            )
            .unwrap(),
            expect_list1
        );

        assert_eq!(
            parse_from_str(
                r#"
            [
                123,
                456,
                789
            ]
            "#
            )
            .unwrap(),
            expect_list1
        );

        assert_eq!(
            parse_from_str(
                r#"
            [
                123,
                456,
                789,
            ]
            "#
            )
            .unwrap(),
            expect_list1
        );

        // err: missing ']', but EOF
        assert!(matches!(
            parse_from_str(r#"[123"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: missing ']', but EOF
        assert!(matches!(
            parse_from_str(r#"[123,"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: missing ']', but EOF
        assert!(matches!(
            parse_from_str(r#"[123,456"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));
    }

    #[test]
    fn test_parse_named_list() {
        let expect_object1 = AsonNode::NamedList(vec![
            NamedListEntry {
                name: Box::new(AsonNode::String("foo".to_owned())),
                value: Box::new(AsonNode::Number(Number::I32(123))),
            },
            NamedListEntry {
                name: Box::new(AsonNode::String("bar".to_owned())),
                value: Box::new(AsonNode::Number(Number::I32(456))),
            },
        ]);

        assert_eq!(
            parse_from_str(
                r#"
            ["foo": 123, "bar": 456]
            "#
            )
            .unwrap(),
            expect_object1
        );

        assert_eq!(
            parse_from_str(
                r#"
            ["foo": 123 "bar": 456]
            "#
            )
            .unwrap(),
            expect_object1
        );

        assert_eq!(
            parse_from_str(
                r#"
            [
                "foo": 123
                "bar": 456
            ]
            "#
            )
            .unwrap(),
            expect_object1
        );

        assert_eq!(
            parse_from_str(
                r#"
            [
                "foo": 123,
                "bar": 456
            ]
            "#
            )
            .unwrap(),
            expect_object1
        );

        assert_eq!(
            parse_from_str(
                r#"
            [
                "foo": 123,
                "bar": 456,
            ]
            "#
            )
            .unwrap(),
            expect_object1
        );
    }

    #[test]
    fn test_parse_tuple() {
        let expect_tuple1 = AsonNode::Tuple(vec![
            AsonNode::Number(Number::I32(123)),
            AsonNode::String("foo".to_owned()),
            AsonNode::Boolean(true),
        ]);

        assert_eq!(
            parse_from_str(
                r#"
            (123,"foo",true)
            "#
            )
            .unwrap(),
            expect_tuple1
        );

        assert_eq!(
            parse_from_str(
                r#"
            (123 "foo" true)
            "#
            )
            .unwrap(),
            expect_tuple1
        );

        assert_eq!(
            parse_from_str(
                r#"
            (
                123
                "foo"
                true
            )
            "#
            )
            .unwrap(),
            expect_tuple1
        );

        assert_eq!(
            parse_from_str(
                r#"
            (
                123,
                "foo",
                true
            )
            "#
            )
            .unwrap(),
            expect_tuple1
        );

        assert_eq!(
            parse_from_str(
                r#"
            (
                123,
                "foo",
                true,
            )
            "#
            )
            .unwrap(),
            expect_tuple1
        );

        // err: empty tuple
        assert!(matches!(
            parse_from_str(r#"()"#),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 1,
                        line: 0,
                        column: 1
                    },
                    end_inclusive: Position {
                        index: 1,
                        line: 0,
                        column: 1
                    }
                }
            ))
        ));

        // err: missing ')', but EOF
        assert!(matches!(
            parse_from_str(r#"(123"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: missing ')', but EOF
        assert!(matches!(
            parse_from_str(r#"(123,"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: missing ')', but EOF
        assert!(matches!(
            parse_from_str(r#"(123,456"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));
    }

    #[test]
    fn test_parse_enumeration() {
        // empty value
        assert_eq!(
            parse_from_str(
                r#"
            Option::None
            "#
            )
            .unwrap(),
            AsonNode::Enumeration(Enumeration::new("Option", "None"))
        );

        // single value
        assert_eq!(
            parse_from_str(
                r#"
            Option::Some(123)
            "#
            )
            .unwrap(),
            AsonNode::Enumeration(Enumeration::with_value(
                "Option",
                "Some",
                AsonNode::Number(Number::I32(123))
            ))
        );

        // tuple-like value
        assert_eq!(
            parse_from_str(
                r#"
            Color::RGB(100,75,0)
            "#
            )
            .unwrap(),
            AsonNode::Enumeration(Enumeration::with_tuple_like(
                "Color",
                "RGB",
                vec![
                    AsonNode::Number(Number::I32(100)),
                    AsonNode::Number(Number::I32(75)),
                    AsonNode::Number(Number::I32(0)),
                ]
            ))
        );

        // tuple-like value without comma
        assert_eq!(
            parse_from_str(
                r#"
            Color::RGB(100 75 0)
            "#
            )
            .unwrap(),
            AsonNode::Enumeration(Enumeration::with_tuple_like(
                "Color",
                "RGB",
                vec![
                    AsonNode::Number(Number::I32(100)),
                    AsonNode::Number(Number::I32(75)),
                    AsonNode::Number(Number::I32(0)),
                ]
            ))
        );

        // object-like value
        assert_eq!(
            parse_from_str(
                r#"
            Shape::Rect{width:123, height:456}
            "#
            )
            .unwrap(),
            AsonNode::Enumeration(Enumeration::with_object_like(
                "Shape",
                "Rect",
                vec![
                    KeyValuePair::new("width", AsonNode::Number(Number::I32(123))),
                    KeyValuePair::new("height", AsonNode::Number(Number::I32(456))),
                ]
            ))
        );

        // object-like value without comma
        assert_eq!(
            parse_from_str(
                r#"
            Shape::Rect{width:123 height:456}
            "#
            )
            .unwrap(),
            AsonNode::Enumeration(Enumeration::with_object_like(
                "Shape",
                "Rect",
                vec![
                    KeyValuePair::new("width", AsonNode::Number(Number::I32(123))),
                    KeyValuePair::new("height", AsonNode::Number(Number::I32(456))),
                ]
            ))
        );

        // err: missing value
        assert!(matches!(
            parse_from_str(r#"Option::Some()"#),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 13,
                        line: 0,
                        column: 13
                    },
                    end_inclusive: Position {
                        index: 13,
                        line: 0,
                        column: 13
                    }
                }
            ))
        ));

        // err: object-like variant missing ':', but '}'
        assert!(matches!(
            parse_from_str(r#"Color::Rect{width}"#),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 17,
                        line: 0,
                        column: 17
                    },
                    end_inclusive: Position {
                        index: 17,
                        line: 0,
                        column: 17
                    }
                }
            ))
        ));

        // err: object-like variant missing value, but '}'
        assert!(matches!(
            parse_from_str(r#"Color::Rect{width:}"#),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 18,
                        line: 0,
                        column: 18
                    },
                    end_inclusive: Position {
                        index: 18,
                        line: 0,
                        column: 18
                    }
                }
            ))
        ));

        // err: missing ')', but EOF
        assert!(matches!(
            parse_from_str(r#"Option::Some(11"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: tuple-like variant missing ')', but EOF
        assert!(matches!(
            parse_from_str(r#"Color::RGB(11,13"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: object-like variant missing ':', but EOF
        assert!(matches!(
            parse_from_str(r#"Color::Rect{width"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: object-like variant missing value, but EOF
        assert!(matches!(
            parse_from_str(r#"Color::Rect{width:"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));

        // err: object-like variant missing '}', but EOF
        assert!(matches!(
            parse_from_str(r#"Color::Rect{width:11"#),
            Err(AsonError::UnexpectedEndOfDocument(_))
        ));
    }

    #[test]
    fn test_parse_document() {
        assert_eq!(
            parse_from_str(
                r#"
            {
                name: "foo"
                version: "0.1.0"
                dependencies: [
                    "registry.domain/user/random@1.0.1"
                    "registry.domain/user/regex@2.0.0"
                ]
            }
            "#
            )
            .unwrap(),
            AsonNode::Object(vec![
                KeyValuePair {
                    key: "name".to_owned(),
                    value: Box::new(AsonNode::String("foo".to_owned())),
                },
                KeyValuePair {
                    key: "version".to_owned(),
                    value: Box::new(AsonNode::String("0.1.0".to_owned())),
                },
                KeyValuePair {
                    key: "dependencies".to_owned(),
                    value: Box::new(AsonNode::List(vec![
                        AsonNode::String("registry.domain/user/random@1.0.1".to_owned()),
                        AsonNode::String("registry.domain/user/regex@2.0.0".to_owned()),
                    ])),
                },
            ])
        );

        assert_eq!(
            parse_from_str(
                r#"
            {
                id:123
                name:"hello"
                orders: [
                    (1, "foo", true)
                    (2, "bar", false)
                ]
                group: {
                    active: true
                    permissions:[
                        {number:11, title: "read"}
                        {number:13, title: "write"}
                    ]
                }
            }
            "#
            )
            .unwrap(),
            AsonNode::Object(vec![
                KeyValuePair {
                    key: "id".to_owned(),
                    value: Box::new(AsonNode::Number(Number::I32(123))),
                },
                KeyValuePair {
                    key: "name".to_owned(),
                    value: Box::new(AsonNode::String("hello".to_owned())),
                },
                KeyValuePair {
                    key: "orders".to_owned(),
                    value: Box::new(AsonNode::List(vec![
                        AsonNode::Tuple(vec![
                            AsonNode::Number(Number::I32(1)),
                            AsonNode::String("foo".to_owned()),
                            AsonNode::Boolean(true),
                        ]),
                        AsonNode::Tuple(vec![
                            AsonNode::Number(Number::I32(2)),
                            AsonNode::String("bar".to_owned()),
                            AsonNode::Boolean(false),
                        ]),
                    ])),
                },
                KeyValuePair {
                    key: "group".to_owned(),
                    value: Box::new(AsonNode::Object(vec![
                        KeyValuePair {
                            key: "active".to_owned(),
                            value: Box::new(AsonNode::Boolean(true)),
                        },
                        KeyValuePair {
                            key: "permissions".to_owned(),
                            value: Box::new(AsonNode::List(vec![
                                AsonNode::Object(vec![
                                    KeyValuePair {
                                        key: "number".to_owned(),
                                        value: Box::new(AsonNode::Number(Number::I32(11))),
                                    },
                                    KeyValuePair {
                                        key: "title".to_owned(),
                                        value: Box::new(AsonNode::String("read".to_owned())),
                                    },
                                ]),
                                AsonNode::Object(vec![
                                    KeyValuePair {
                                        key: "number".to_owned(),
                                        value: Box::new(AsonNode::Number(Number::I32(13))),
                                    },
                                    KeyValuePair {
                                        key: "title".to_owned(),
                                        value: Box::new(AsonNode::String("write".to_owned())),
                                    },
                                ]),
                            ])),
                        },
                    ])),
                },
            ])
        );

        // err: document contains multiple root nodes
        assert!(matches!(
            parse_from_str(r#"true false"#),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 5,
                        line: 0,
                        column: 5
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
}
