// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::ops::Neg;

use crate::{
    error::AsonError,
    peekable_iter::PeekableIter,
    range::Range,
    token::{NumberToken, Token, TokenWithRange},
};

// /// Remove all comments from the token stream.
// pub struct RemoveCommentsIter<'a> {
//     upstream: &'a mut dyn Iterator<Item = Result<TokenWithRange, AsonError>>,
// }

// impl<'a> RemoveCommentsIter<'a> {
//     pub fn new(upstream: &'a mut dyn Iterator<Item = Result<TokenWithRange, AsonError>>) -> Self {
//         Self { upstream }
//     }
// }

// impl Iterator for RemoveCommentsIter<'_> {
//     type Item = Result<TokenWithRange, AsonError>;

//     fn next(&mut self) -> Option<Self::Item> {
//         // Remove all comments from the token stream.
//         loop {
//             match self.upstream.next() {
//                 Some(result) => {
//                     match &result {
//                         Ok(TokenWithRange {
//                             token: Token::Comment(_),
//                             ..
//                         }) => {
//                             // consume comments
//                         }
//                         _ => {
//                             return Some(result);
//                         }
//                     }
//                 }
//                 None => {
//                     return None;
//                 }
//             }
//         }
//     }
// }

// /// Merge multiple continuous newlines into one newline.
// pub struct MergeNewlinesIter<'a> {
//     upstream: &'a mut PeekableIter<'a, Result<TokenWithRange, AsonError>>,
// }

// impl<'a> MergeNewlinesIter<'a> {
//     pub fn new(upstream: &'a mut PeekableIter<'a, Result<TokenWithRange, AsonError>>) -> Self {
//         Self { upstream }
//     }
// }

// impl Iterator for MergeNewlinesIter<'_> {
//     type Item = Result<TokenWithRange, AsonError>;

//     fn next(&mut self) -> Option<Self::Item> {
//         // - combine multiple continuous newlines into one newline.
//         //   rules:
//         //     + multiple newlines => single newline
//         //     + comma + newline(s) => comma
//         //     + newline(s) + comma => comma
//         //     + newline(s) + comma + newline(s) => comma
//         //
//         //   because the comments have been removed, the following conclusions
//         //   can be inferred:
//         //     + comma + comment(s) + comma => comma + comma
//         //     + newline(s) + comment(s) + newline(s) => newline
//         //
//         match self.upstream.next() {
//             Some(result) => match &result {
//                 Ok(token_with_range) => {
//                     let TokenWithRange {
//                         token: current_token,
//                         range: current_range,
//                     } = token_with_range;

//                     let mut start_range = *current_range;
//                     let mut end_range = *current_range;

//                     match current_token {
//                         Token::NewLine => {
//                             // consume continuous newlines
//                             while let Some(Ok(TokenWithRange {
//                                 token: Token::NewLine,
//                                 range: next_range,
//                             })) = self.upstream.peek(0)
//                             {
//                                 end_range = *next_range;
//                                 self.upstream.next();
//                             }

//                             // found ','
//                             if let Some(Ok(TokenWithRange {
//                                 token: Token::Comma,
//                                 range: next_range,
//                             })) = self.upstream.peek(0)
//                             {
//                                 // consume comma
//                                 start_range = *next_range;
//                                 end_range = *next_range;
//                                 self.upstream.next();

//                                 // consume trailing continuous newlines
//                                 while let Some(Ok(TokenWithRange {
//                                     token: Token::NewLine,
//                                     range: _,
//                                 })) = self.upstream.peek(0)
//                                 {
//                                     self.upstream.next();
//                                 }

//                                 Some(Ok(TokenWithRange::new(
//                                     Token::Comma,
//                                     Range::merge(&start_range, &end_range),
//                                 )))
//                             } else {
//                                 Some(Ok(TokenWithRange::new(
//                                     Token::NewLine,
//                                     Range::merge(&start_range, &end_range),
//                                 )))
//                             }
//                         }
//                         Token::Comma => {
//                             // consume trailing continuous newlines
//                             while let Some(Ok(TokenWithRange {
//                                 token: Token::NewLine,
//                                 range: _,
//                             })) = self.upstream.peek(0)
//                             {
//                                 self.upstream.next();
//                             }

//                             Some(Ok(TokenWithRange::new(
//                                 Token::Comma,
//                                 Range::merge(&start_range, &end_range),
//                             )))
//                         }
//                         _ => Some(result),
//                     }
//                 }
//                 Err(_) => Some(result),
//             },
//             None => None,
//         }
//     }
// }

/// Check and normalize signed numbers in the token stream.
pub struct CheckSignedNumberIter<'a> {
    upstream: &'a mut PeekableIter<'a, Result<TokenWithRange, AsonError>>,
}

impl<'a> CheckSignedNumberIter<'a> {
    pub fn new(upstream: &'a mut PeekableIter<'a, Result<TokenWithRange, AsonError>>) -> Self {
        Self { upstream }
    }
}

impl Iterator for CheckSignedNumberIter<'_> {
    type Item = Result<TokenWithRange, AsonError>;

    fn next(&mut self) -> Option<Self::Item> {
        // - remove the '+' tokens in front of numbers (includes `+Inf`).
        // - apply the '-' tokens to numbers (includes `-Inf`).
        // - checks if the signed number is overflowed.
        //
        //   note that the lexer only checked the number width, it does not check the valid range of a signed integer
        //   because it does not know if there is a plus or minus sign in front of the number token.
        //
        //   for example, "128" is invalid for `i8`, but "-128" is a valid `i8`.
        //   thus the valid range of an integer can only be checked in the normalization
        //   phase after combining the plus or minus sign and the number of tokens.
        match self.upstream.next() {
            Some(result) => match &result {
                Ok(token_with_range) => {
                    let TokenWithRange {
                        token: current_token,
                        range: current_range,
                    } = token_with_range;

                    let start_range = *current_range;

                    match current_token {
                        Token::Plus => {
                            match self.upstream.peek(0) {
                                Some(Ok(TokenWithRange {
                                    token: Token::Number(num),
                                    range: next_range,
                                })) => {
                                    match num {
                                        NumberToken::F32(f) if f.is_nan() => {
                                            // combines two token ranges.
                                            Some(Err(AsonError::MessageWithRange(
                                                "The plus sign cannot be applied to NaN."
                                                    .to_owned(),
                                                Range::merge(&start_range, next_range),
                                            )))
                                        }
                                        NumberToken::F64(f) if f.is_nan() => {
                                            // combines two token ranges.
                                            Some(Err(AsonError::MessageWithRange(
                                                "The plus sign cannot be applied to NaN."
                                                    .to_owned(),
                                                Range::merge(&start_range, next_range),
                                            )))
                                        }
                                        NumberToken::I8(v) if *v > i8::MAX as u8 => {
                                            // check signed number overflow
                                            Some(Err(AsonError::MessageWithRange(
                                                format!(
                                                    "The signed i8 number {} is overflowed.",
                                                    v
                                                ),
                                                Range::merge(&start_range, next_range),
                                            )))
                                        }
                                        NumberToken::I16(v) if *v > i16::MAX as u16 => {
                                            // check signed number overflow
                                            Some(Err(AsonError::MessageWithRange(
                                                format!(
                                                    "The signed i16 number {} is overflowed.",
                                                    v
                                                ),
                                                Range::merge(&start_range, next_range),
                                            )))
                                        }
                                        NumberToken::I32(v) if *v > i32::MAX as u32 => {
                                            // check signed number overflow
                                            Some(Err(AsonError::MessageWithRange(
                                                format!(
                                                    "The signed i32 number {} is overflowed.",
                                                    v
                                                ),
                                                Range::merge(&start_range, next_range),
                                            )))
                                        }
                                        NumberToken::I64(v) if *v > i64::MAX as u64 => {
                                            // check signed number overflow
                                            Some(Err(AsonError::MessageWithRange(
                                                format!(
                                                    "The signed i64 number {} is overflowed.",
                                                    v
                                                ),
                                                Range::merge(&start_range, next_range),
                                            )))
                                        }
                                        _ => {
                                            // consumes the the plus sign (it's already done) and the
                                            // number token.
                                            let TokenWithRange {
                                                token: combined_token,
                                                range: next_range,
                                            } = self.upstream.next().unwrap().unwrap();

                                            // combines two token ranges and constructs new number token.
                                            Some(Ok(TokenWithRange {
                                                token: combined_token,
                                                range: Range::merge(&start_range, &next_range),
                                            }))
                                        }
                                    }
                                }
                                Some(Ok(TokenWithRange {
                                    token: _,
                                    range: current_range,
                                })) => {
                                    // combines two token ranges.
                                    Some(Err(AsonError::MessageWithRange(
                                        "The plus sign can only be applied to numbers.".to_owned(),
                                        Range::merge(&start_range, current_range),
                                    )))
                                }
                                Some(Err(e)) => Some(Err(e.clone())),
                                None => {
                                    // "...+EOF"
                                    Some(Err(AsonError::UnexpectedEndOfDocument(
                                        "The plus sign is not followed by a number.".to_owned(),
                                    )))
                                }
                            }
                        }
                        Token::Minus => {
                            match self.upstream.peek(0) {
                                Some(Ok(TokenWithRange {
                                    token: Token::Number(num),
                                    range: next_range,
                                })) => {
                                    match num {
                                        NumberToken::F32(v) => {
                                            if v.is_nan() {
                                                // combines two token ranges.
                                                Some(Err(AsonError::MessageWithRange(
                                                    "The minus sign cannot be applied to NaN."
                                                        .to_owned(),
                                                    Range::merge(&start_range, next_range),
                                                )))
                                            } else {
                                                // combines two token ranges and constructs new number token.
                                                let ret_val = Some(Ok(TokenWithRange {
                                                    token: Token::Number(NumberToken::F32(v.neg())),
                                                    range: Range::merge(
                                                        &start_range,
                                                        next_range,
                                                    ),
                                                }));

                                                // consume the minus sign (it's already done) and the
                                                // number token
                                                self.upstream.next();

                                                ret_val
                                            }
                                        }
                                        NumberToken::F64(v) => {
                                            if v.is_nan() {
                                                // combines two token ranges.
                                                Some(Err(AsonError::MessageWithRange(
                                                    "The minus sign cannot be applied to NaN."
                                                        .to_owned(),
                                                    Range::merge(&start_range, next_range),
                                                )))
                                            } else {
                                                // combines two token ranges and constructs new number token.
                                                let ret_val = Some(Ok(TokenWithRange {
                                                    token: Token::Number(NumberToken::F64(v.neg())),
                                                    range: Range::merge(
                                                        &start_range,
                                                        next_range,
                                                    ),
                                                }));

                                                // consume the minus sign (it's already done) and the
                                                // number token
                                                self.upstream.next();

                                                ret_val
                                            }
                                        }
                                        NumberToken::I8(v) => {
                                            let combined_range =
                                                Range::merge(&start_range, next_range);

                                            let parse_result =
                                                format!("-{}", v).parse::<i8>().map_err(|_| {
                                                    AsonError::MessageWithRange(
                                                        format!(
                                                            "Can not convert \"{}\" to negative i8",
                                                            v
                                                        ),
                                                        combined_range,
                                                    )
                                                });

                                            match parse_result {
                                                Ok(v) => {
                                                    let ret_val = Some(Ok(TokenWithRange::new(
                                                        Token::Number(NumberToken::I8(v as u8)),
                                                        combined_range,
                                                    )));

                                                    // consume the minus sign (already done) and the number literal token
                                                    self.next();

                                                    ret_val
                                                }
                                                Err(e) => Some(Err(e)),
                                            }
                                        }
                                        NumberToken::I16(v) => {
                                            let combined_range =
                                                Range::merge(&start_range, next_range);

                                            let parse_result =
                                                format!("-{}", v).parse::<i16>().map_err(|_| {
                                                    AsonError::MessageWithRange(
                                                        format!(
                                                            "Can not convert \"{}\" to negative i16.",
                                                            v
                                                        ),
                                                        combined_range,
                                                    )
                                                });

                                            match parse_result {
                                                Ok(v) => {
                                                    let ret_val = Some(Ok(TokenWithRange::new(
                                                        Token::Number(NumberToken::I16(v as u16)),
                                                        combined_range,
                                                    )));

                                                    // consume the minus sign (already done) and the number literal token
                                                    self.next();

                                                    ret_val
                                                }
                                                Err(e) => Some(Err(e)),
                                            }
                                        }
                                        NumberToken::I32(v) => {
                                            let combined_range =
                                                Range::merge(&start_range, next_range);

                                            let parse_result =
                                                format!("-{}", v).parse::<i32>().map_err(|_| {
                                                    AsonError::MessageWithRange(
                                                        format!(
                                                            "Can not convert \"{}\" to negative i32.",
                                                            v
                                                        ),
                                                        combined_range,
                                                    )
                                                });

                                            match parse_result {
                                                Ok(v) => {
                                                    let ret_val = Some(Ok(TokenWithRange::new(
                                                        Token::Number(NumberToken::I32(v as u32)),
                                                        combined_range,
                                                    )));

                                                    // consume the minus sign (already done) and the number literal token
                                                    self.next();

                                                    ret_val
                                                }
                                                Err(e) => Some(Err(e)),
                                            }
                                        }
                                        NumberToken::I64(v) => {
                                            let combined_range =
                                                Range::merge(&start_range, next_range);

                                            let parse_result =
                                                format!("-{}", v).parse::<i64>().map_err(|_| {
                                                    AsonError::MessageWithRange(
                                                        format!(
                                                            "Can not convert \"{}\" to negative i64.",
                                                            v
                                                        ),
                                                        combined_range,
                                                    )
                                                });

                                            match parse_result {
                                                Ok(v) => {
                                                    let ret_val = Some(Ok(TokenWithRange::new(
                                                        Token::Number(NumberToken::I64(v as u64)),
                                                        combined_range,
                                                    )));

                                                    // consume the minus sign (already done) and the number literal token
                                                    self.next();

                                                    ret_val
                                                }
                                                Err(e) => Some(Err(e)),
                                            }
                                        }
                                        NumberToken::U8(_)
                                        | NumberToken::U16(_)
                                        | NumberToken::U32(_)
                                        | NumberToken::U64(_) => {
                                            Some(Err(AsonError::MessageWithRange(
                                                "The minus sign cannot be applied to unsigned numbers."
                                                    .to_owned(),
                                                Range::merge(&start_range, next_range),
                                            )))
                                        }
                                    }
                                }
                                Some(Ok(TokenWithRange {
                                    token: _,
                                    range: next_range,
                                })) => {
                                    // combines two token ranges.
                                    Some(Err(AsonError::MessageWithRange(
                                        "The minus sign can only be applied to numbers.".to_owned(),
                                        Range::merge(&start_range, next_range),
                                    )))
                                }
                                Some(Err(e)) => Some(Err(e.clone())),
                                None => {
                                    // "...-EOF"
                                    Some(Err(AsonError::UnexpectedEndOfDocument(
                                        "The minus sign is not followed by a number.".to_owned(),
                                    )))
                                }
                            }
                        }
                        Token::Number(NumberToken::I8(v)) if *v > i8::MAX as u8 => {
                            // check signed number overflow
                            Some(Err(AsonError::MessageWithRange(
                                format!("The signed i8 number {} is overflowed.", v),
                                start_range,
                            )))
                        }
                        Token::Number(NumberToken::I16(v)) if *v > i16::MAX as u16 => {
                            // check signed number overflow
                            Some(Err(AsonError::MessageWithRange(
                                format!("The signed i16 number {} is overflowed.", v),
                                start_range,
                            )))
                        }
                        Token::Number(NumberToken::I32(v)) if *v > i32::MAX as u32 => {
                            // check signed number overflow
                            Some(Err(AsonError::MessageWithRange(
                                format!("The signed i32 number {} is overflowed.", v),
                                start_range,
                            )))
                        }
                        Token::Number(NumberToken::I64(v)) if *v > i64::MAX as u64 => {
                            // check signed number overflow
                            Some(Err(AsonError::MessageWithRange(
                                format!("The signed i64 number {} is overflowed.", v),
                                start_range,
                            )))
                        }
                        _ => Some(result),
                    }
                }
                Err(_) => Some(result),
            },
            None => None,
        }
    }
}

// /// Trim the leading and trailing newlines of the document.
// pub struct TrimDocumentIter<'a> {
//     upstream: &'a mut PeekableIter<'a, Result<TokenWithRange, AsonError>>,
// }

// impl<'a> TrimDocumentIter<'a> {
//     pub fn new(upstream: &'a mut PeekableIter<'a, Result<TokenWithRange, AsonError>>) -> Self {
//         // consume the leading '\n of document
//         if let Some(Ok(TokenWithRange {
//             token: Token::NewLine,
//             ..
//         })) = upstream.peek(0)
//         {
//             upstream.next();
//         }

//         Self { upstream }
//     }
// }

// impl Iterator for TrimDocumentIter<'_> {
//     type Item = Result<TokenWithRange, AsonError>;

//     fn next(&mut self) -> Option<Self::Item> {
//         match self.upstream.next() {
//             Some(r) => {
//                 match &r {
//                     Ok(tl) => {
//                         let TokenWithRange { token, .. } = tl;
//                         match token {
//                             Token::NewLine if self.upstream.peek(0).is_none() => {
//                                 // it is the last '\n' of document
//                                 None
//                             }
//                             _ => Some(r),
//                         }
//                     }
//                     Err(_) => Some(r),
//                 }
//             }
//             None => None,
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        char_with_position::CharsWithPositionIter,
        error::AsonError,
        lexer::{Lexer, PEEK_BUFFER_LENGTH_LEX},
        normalizer::CheckSignedNumberIter,
        peekable_iter::PeekableIter,
        position::Position,
        range::Range,
        token::{NumberToken, Token, TokenWithRange},
    };

    /// Helper function to lex tokens from a string.
    fn lex_from_str(s: &str) -> Result<Vec<TokenWithRange>, AsonError> {
        // Lex
        let mut chars = s.chars();
        let mut char_position_iter = CharsWithPositionIter::new(&mut chars);
        let mut peekable_char_position_iter =
            PeekableIter::new(&mut char_position_iter, PEEK_BUFFER_LENGTH_LEX);
        let mut lexer = Lexer::new(&mut peekable_char_position_iter);

        // // Remove comments
        // let mut removed_comments_iter = RemoveCommentsIter::new(&mut lexer);

        // // Merge newlines
        // let mut peekable_removed_comments_iter = PeekableIter::new(&mut removed_comments_iter, 1);
        // let mut merged_newlines_iter = MergeNewlinesIter::new(&mut peekable_removed_comments_iter);

        // Normalize signed numbers
        let mut peekable_lexer_iter = PeekableIter::new(&mut lexer, 1);
        // let mut peekable_merged_newlines_iter = PeekableIter::new(&mut merged_newlines_iter, 1);
        let normalizer_iter =
            CheckSignedNumberIter::new(&mut peekable_lexer_iter);

        // // Trim document
        // let mut peekable_checked_signed_number_iter =
        //     PeekableIter::new(&mut checked_signed_number_iter, 1);
        // let mut trimmed_document_iter =
        //     TrimDocumentIter::new(&mut peekable_checked_signed_number_iter);

        // Collect tokens
        //
        // do not use `iter.collect::<Vec<_>>()` to collect tokens,
        // because the `Lexer` throws exceptions via `next() -> Option<Result<...>>`.
        //
        // if we use `collect()`, once an error occurs,
        // the iterator wouldn't stop immediately, instead, it would continue to iterate until the end,
        let mut token_with_ranges = vec![];
        for result in normalizer_iter {
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
    fn test_remove_comments() {
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
                // Token::NewLine,
                Token::Number(NumberToken::I32(13)),
                // Token::NewLine,
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
    fn test_merge_newlines() {
        // test items:
        //
        // unchaged:
        // - comma => comma
        //
        // normalized:
        // - comma + blank(s) => comma
        // - blank(s) + comma => comma
        // - blank(s) + comma + blank(s) => comma
        //
        // inferred:
        // - comma + comment(s) + comma => comma + comma
        // - blank(s) + comment(s) + blank(s) => blank
        //
        // normalization:
        // - blanks => blank

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
                // Token::Comma,
                Token::Number(NumberToken::I32(2)),
                // Token::Comma,
                Token::Number(NumberToken::I32(3)),
                // Token::Comma,
                Token::Number(NumberToken::I32(4)),
                // Token::Comma,
                Token::Number(NumberToken::I32(5)),
                // Token::Comma,
                // Token::Comma,
                Token::Number(NumberToken::I32(6)),
                // Token::NewLine,
                Token::Number(NumberToken::I32(7)),
                // Token::NewLine,
                Token::Number(NumberToken::I32(8)),
                // Token::NewLine,
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
                // TokenWithRange::new(
                //     Token::NewLine,
                //     Range::new(&Position::new(2, 0, 2), &Position::new(7, 2, 2))
                // ),
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(13)),
                    Range::from_position_and_length(&Position::new(8, 3, 0), 2)
                ),
            ]
        );

        // comma + blanks -> comma
        assert_eq!(
            lex_from_str(",\n\n\n11").unwrap(),
            vec![
                // TokenWithRange::new(
                //     Token::Comma,
                //     Range::from_position_and_length(&Position::new(0, 0, 0), 1)
                // ),
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(11)),
                    Range::from_position_and_length(&Position::new(4, 3, 0), 2)
                ),
            ]
        );

        // blanks + comma -> comma
        assert_eq!(
            lex_from_str("11\n\n\n,").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(11)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 2)
                ),
                // TokenWithRange::new(
                //     Token::Comma,
                //     Range::from_position_and_length(&Position::new(5, 3, 0), 1)
                // ),
            ]
        );

        // blanks + comma + blanks -> comma
        assert_eq!(
            lex_from_str("11\n\n,\n\n13").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(11)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 2)
                ),
                // TokenWithRange::new(
                //     Token::Comma,
                //     Range::from_position_and_length(&Position::new(4, 2, 0), 1)
                // ),
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(13)),
                    Range::from_position_and_length(&Position::new(7, 4, 0), 2)
                ),
            ]
        );

        // comma + comment + comma -> comma + comma
        assert_eq!(
            lex_from_str(",//abc\n,").unwrap(),
            vec![
                // TokenWithRange::new(
                //     Token::Comma,
                //     Range::from_position_and_length(&Position::new(0, 0, 0), 1)
                // ),
                // TokenWithRange::new(
                //     Token::Comma,
                //     Range::from_position_and_length(&Position::new(7, 1, 0), 1)
                // ),
            ]
        );

        // blanks + comment + blanks -> blank
        assert_eq!(
            lex_from_str("11\n\n//abc\n\n13").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(11)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 2)
                ),
                // TokenWithRange::new(
                //     Token::NewLine,
                //     Range::new(&Position::new(2, 0, 2), &Position::new(10, 3, 0))
                // ),
                TokenWithRange::new(
                    Token::Number(NumberToken::I32(13)),
                    Range::from_position_and_length(&Position::new(11, 4, 0), 2)
                ),
            ]
        );
    }

    #[test]
    fn test_normalize_plus_and_minus_decimal_numbers() {
        // implicit type, default `i32`
        {
            assert_eq!(
                lex_from_str_without_location("+11").unwrap(),
                vec![Token::Number(NumberToken::I32(11))]
            );

            assert_eq!(
                lex_from_str_without_location("-13").unwrap(),
                vec![Token::Number(NumberToken::I32(-13_i32 as u32))]
            );

            // err: positive overflow
            // i32 max: 2_147_483_647
            assert!(matches!(
                lex_from_str("+2_147_483_648"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 13,
                            line: 0,
                            column: 13,
                        }
                    }
                ))
            ));

            // err: negative overflow
            // i32 min: -2_147_483_648
            assert!(matches!(
                lex_from_str("-2_147_483_649"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 13,
                            line: 0,
                            column: 13,
                        }
                    }
                ))
            ));
        }

        // i8/u8
        {
            assert_eq!(
                lex_from_str_without_location("+127_i8").unwrap(),
                vec![Token::Number(NumberToken::I8(127))]
            );

            assert_eq!(
                lex_from_str_without_location("-128_i8").unwrap(),
                vec![Token::Number(NumberToken::I8(-128_i8 as u8))]
            );

            // err: positive overflow
            // i8 max: 127
            assert!(matches!(
                lex_from_str("+128_i8"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 6,
                            line: 0,
                            column: 6,
                        }
                    }
                ))
            ));

            // err: negative overflow
            // i8 min: -128
            assert!(matches!(
                lex_from_str("-129_i8"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 6,
                            line: 0,
                            column: 6,
                        }
                    }
                ))
            ));

            // err: unsigned number with minus sign
            assert!(matches!(
                lex_from_str("-1_u8"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 4,
                            line: 0,
                            column: 4,
                        }
                    }
                ))
            ));
        }

        // i16/u16
        {
            assert_eq!(
                lex_from_str_without_location("+32767_i16").unwrap(),
                vec![Token::Number(NumberToken::I16(32767))]
            );

            assert_eq!(
                lex_from_str_without_location("-32768_i16").unwrap(),
                vec![Token::Number(NumberToken::I16(-32768_i16 as u16))]
            );

            // err: positive overflow
            // i16 max: 32767
            assert!(matches!(
                lex_from_str("+32768_i16"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 9,
                            line: 0,
                            column: 9,
                        }
                    }
                ))
            ));

            // err: negative overflow
            // i16 min: -32768
            assert!(matches!(
                lex_from_str("-32769_i16"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 9,
                            line: 0,
                            column: 9,
                        }
                    }
                ))
            ));

            // err: unsigned number with minus sign
            assert!(matches!(
                lex_from_str("-1_u16"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 5,
                            line: 0,
                            column: 5,
                        }
                    }
                ))
            ));
        }

        // i32/u32
        {
            assert_eq!(
                lex_from_str_without_location("+2_147_483_647_i32").unwrap(),
                vec![Token::Number(NumberToken::I32(2_147_483_647i32 as u32))]
            );

            assert_eq!(
                lex_from_str_without_location("-2_147_483_648_i32").unwrap(),
                vec![Token::Number(NumberToken::I32(-2_147_483_648i32 as u32))]
            );

            // err: positive overflow
            // i32 max: 2_147_483_647
            assert!(matches!(
                lex_from_str("+2_147_483_648_i32"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 17,
                            line: 0,
                            column: 17,
                        }
                    }
                ))
            ));

            // err: negative overflow
            // i32 min: -2_147_483_648
            assert!(matches!(
                lex_from_str("-2_147_483_649_i32"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 17,
                            line: 0,
                            column: 17,
                        }
                    }
                ))
            ));

            // err: unsigned number with minus sign
            assert!(matches!(
                lex_from_str("-1_u32"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 5,
                            line: 0,
                            column: 5,
                        }
                    }
                ))
            ));
        }

        // i64/u64
        {
            assert_eq!(
                lex_from_str_without_location("+9_223_372_036_854_775_807_i64").unwrap(),
                vec![Token::Number(NumberToken::I64(
                    9_223_372_036_854_775_807i64 as u64
                )),]
            );

            assert_eq!(
                lex_from_str_without_location("-9_223_372_036_854_775_808_i64").unwrap(),
                vec![Token::Number(NumberToken::I64(
                    -9_223_372_036_854_775_808i64 as u64
                )),]
            );

            // err: positive overflow
            // i64 max: 9_223_372_036_854_775_807
            assert!(matches!(
                lex_from_str("+9_223_372_036_854_775_808_i64"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 29,
                            line: 0,
                            column: 29,
                        }
                    }
                ))
            ));

            // err: negative overflow
            // i64 min: -9_223_372_036_854_775_808
            assert!(matches!(
                lex_from_str("-9_223_372_036_854_775_809_i64"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 29,
                            line: 0,
                            column: 29,
                        }
                    }
                ))
            ));

            // err: unsigned number with minus sign
            assert!(matches!(
                lex_from_str("-1_u64"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 5,
                            line: 0,
                            column: 5,
                        }
                    }
                ))
            ));
        }

        // range

        {
            assert_eq!(
                lex_from_str("+11").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::I32(11)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 3)
                ),]
            );

            assert_eq!(
                lex_from_str("-13").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::I32(-13_i32 as u32)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 3)
                ),]
            );

            assert_eq!(
                lex_from_str("+11,-13").unwrap(),
                vec![
                    TokenWithRange::new(
                        Token::Number(NumberToken::I32(11)),
                        Range::from_position_and_length(&Position::new(0, 0, 0), 3)
                    ),
                    // TokenWithRange::new(
                    //     Token::Comma,
                    //     Range::from_position_and_length(&Position::new(3, 0, 3), 1)
                    // ),
                    TokenWithRange::new(
                        Token::Number(NumberToken::I32(-13_i32 as u32)),
                        Range::from_position_and_length(&Position::new(4, 0, 4), 3)
                    ),
                ]
            );
        }

        // +EOF
        assert!(matches!(
            lex_from_str("abc,+"),
            Err(AsonError::UnexpectedEndOfDocument(_,))
        ));

        // -EOF
        assert!(matches!(
            lex_from_str("xyz,-"),
            Err(AsonError::UnexpectedEndOfDocument(_,))
        ));

        // err: plus sign is followed by non-numbers
        assert!(matches!(
            lex_from_str("+true"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 4,
                        line: 0,
                        column: 4
                    }
                }
            ))
        ));

        // err: minus sign is followed by non-numbers
        assert!(matches!(
            lex_from_str("-true"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 4,
                        line: 0,
                        column: 4
                    }
                }
            ))
        ));
    }

    #[test]
    fn test_normalize_plus_and_minus_floating_point_numbers() {
        // general
        assert_eq!(
            lex_from_str("+3.402_823_5e+38").unwrap(),
            vec![TokenWithRange::new(
                Token::Number(NumberToken::F64(3.402_823_5e38f64)),
                Range::from_position_and_length(&Position::new(0, 0, 0), 16)
            )]
        );

        assert_eq!(
            lex_from_str("-3.402_823_5e+38").unwrap(),
            vec![TokenWithRange::new(
                Token::Number(NumberToken::F64(-3.402_823_5e38f64)),
                Range::from_position_and_length(&Position::new(0, 0, 0), 16)
            )]
        );

        // 0.0, +0.0, -0.0
        {
            assert_eq!(
                lex_from_str_without_location("0.0").unwrap(),
                vec![Token::Number(NumberToken::F64(0f64))]
            );

            assert_eq!(
                lex_from_str("+0.0").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::F64(0f64)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 4)
                )]
            );

            // +0 == -0
            assert_eq!(
                lex_from_str("-0.0").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::F64(0f64)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 4)
                )]
            );
        }

        // NaN
        {
            let t = lex_from_str_without_location("NaN").unwrap();
            assert!(matches!(t[0], Token::Number(NumberToken::F64(v)) if v.is_nan()));
        }

        // Inf
        {
            assert_eq!(
                lex_from_str_without_location("Inf").unwrap(),
                vec![Token::Number(NumberToken::F64(f64::INFINITY))]
            );

            assert_eq!(
                lex_from_str("+Inf").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::F64(f64::INFINITY)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 4)
                )]
            );

            assert_eq!(
                lex_from_str("-Inf").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::F64(f64::NEG_INFINITY)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 4)
                )]
            );
        }

        // err: +NaN, plus sign preceding NaN is invalid
        assert!(matches!(
            lex_from_str("+NaN"),
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
                        column: 3
                    }
                }
            ))
        ));

        // err: -NaN, minus sign preceding NaN is invalid
        assert!(matches!(
            lex_from_str("-NaN"),
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
                        column: 3
                    }
                }
            ))
        ));

        // explicit type `f32` (single precision)
        {
            assert_eq!(
                lex_from_str("+1.602_176_6e-19_f32").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::F32(1.602_176_6e-19f32)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 20)
                )]
            );

            assert_eq!(
                lex_from_str("-1.602_176_6e-19_f32").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::F32(-1.602_176_6e-19f32)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 20)
                )]
            );

            assert_eq!(
                lex_from_str_without_location("0_f32").unwrap(),
                vec![Token::Number(NumberToken::F32(0f32))]
            );

            assert_eq!(
                lex_from_str("+0_f32").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::F32(0f32)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 6)
                )]
            );

            // +0 == -0
            assert_eq!(
                lex_from_str("-0_f32").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::F32(0f32)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 6)
                )]
            );

            let t = lex_from_str_without_location("NaN_f32").unwrap();
            assert!(matches!(t[0], Token::Number(NumberToken::F32(v)) if v.is_nan()));

            assert_eq!(
                lex_from_str_without_location("Inf_f32").unwrap(),
                vec![Token::Number(NumberToken::F32(f32::INFINITY))]
            );

            assert_eq!(
                lex_from_str("+Inf_f32").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::F32(f32::INFINITY)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 8)
                )]
            );

            assert_eq!(
                lex_from_str("-Inf_f32").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::F32(f32::NEG_INFINITY)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 8)
                )]
            );

            // err: +NaN_f32, plus sign preceeding NaN is invalid
            assert!(matches!(
                lex_from_str("+NaN_f32"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 7,
                            line: 0,
                            column: 7
                        }
                    }
                ))
            ));

            // err: -NaN_f32, minus sign preceeding NaN is invalid
            assert!(matches!(
                lex_from_str("-NaN_f32"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
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

        // explicit type `f64` (double precision)
        {
            assert_eq!(
                lex_from_str("+1.797_693_134_862_315_7e+308_f64").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::F64(1.797_693_134_862_315_7e308_f64)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 33)
                )]
            );

            assert_eq!(
                lex_from_str("-1.797_693_134_862_315_7e+308_f64").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::F64(-1.797_693_134_862_315_7e308_f64)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 33)
                )]
            );

            assert_eq!(
                lex_from_str_without_location("0_f64").unwrap(),
                vec![Token::Number(NumberToken::F64(0f64))]
            );

            assert_eq!(
                lex_from_str("+0_f64").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::F64(0f64)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 6)
                )]
            );

            // +0 == -0
            assert_eq!(
                lex_from_str("-0_f64").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::F64(0f64)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 6)
                )]
            );

            let t = lex_from_str_without_location("NaN_f64").unwrap();
            assert!(matches!(t[0], Token::Number(NumberToken::F64(v)) if v.is_nan()));

            assert_eq!(
                lex_from_str_without_location("Inf_f64").unwrap(),
                vec![Token::Number(NumberToken::F64(f64::INFINITY))]
            );

            assert_eq!(
                lex_from_str("+Inf_f64").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::F64(f64::INFINITY)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 8)
                )]
            );

            assert_eq!(
                lex_from_str("-Inf_f64").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::F64(f64::NEG_INFINITY)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 8)
                )]
            );

            // err: +NaN_f64, plus sign preceding NaN is invalid
            assert!(matches!(
                lex_from_str("+NaN_f64"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 7,
                            line: 0,
                            column: 7
                        }
                    }
                ))
            ));

            // err: -NaN, minus sign preceding NaN is invalid
            assert!(matches!(
                lex_from_str("-NaN_f64"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
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
    }

    #[test]
    fn test_normalize_plus_and_minus_hexadecimal_numbers() {
        // implicit type, default `i32`
        {
            assert_eq!(
                lex_from_str_without_location("+0x11").unwrap(),
                vec![Token::Number(NumberToken::I32(0x11))]
            );

            assert_eq!(
                lex_from_str_without_location("-0x13").unwrap(),
                vec![Token::Number(NumberToken::I32(-0x13_i32 as u32))]
            );

            // err: positive overflow
            // i32 max: 0x7fff_ffff
            assert!(matches!(
                lex_from_str("+0x8000_0000"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 11,
                            line: 0,
                            column: 11,
                        }
                    }
                ))
            ));

            // err: negative overflow
            // i32 min: -0x8000_0000
            assert!(matches!(
                lex_from_str("-0x8000_0001"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 11,
                            line: 0,
                            column: 11,
                        }
                    }
                ))
            ));
        }

        // i8/u8
        {
            assert_eq!(
                lex_from_str_without_location("+0x7f_i8").unwrap(),
                vec![Token::Number(NumberToken::I8(0x7f_i8 as u8))]
            );

            assert_eq!(
                lex_from_str_without_location("-0x80_i8").unwrap(),
                vec![Token::Number(NumberToken::I8(-0x80_i8 as u8))]
            );

            // err: positive overflow
            // i8 max: 0x7f
            assert!(matches!(
                lex_from_str("+0x80_i8"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 7,
                            line: 0,
                            column: 7,
                        }
                    }
                ))
            ));

            // err: negative overflow
            // i8 min: -0x80
            assert!(matches!(
                lex_from_str("-0x81_i8"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 7,
                            line: 0,
                            column: 7,
                        }
                    }
                ))
            ));

            // err: minus sign preceding unsigned number
            assert!(matches!(
                lex_from_str("-0x1_u8"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 6,
                            line: 0,
                            column: 6,
                        }
                    }
                ))
            ));
        }

        // i16/u16
        {
            assert_eq!(
                lex_from_str_without_location("+0x7fff_i16").unwrap(),
                vec![Token::Number(NumberToken::I16(0x7fff_i16 as u16))]
            );

            assert_eq!(
                lex_from_str_without_location("-0x8000_i16").unwrap(),
                vec![Token::Number(NumberToken::I16(-0x8000_i16 as u16))]
            );

            // err: positive overflow
            // i16 max: 0x7fff
            assert!(matches!(
                lex_from_str("+0x8000_i16"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 10,
                            line: 0,
                            column: 10,
                        }
                    }
                ))
            ));

            // err: negative overflow
            // i16 min: -0x8000
            assert!(matches!(
                lex_from_str("-0x8001_i16"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 10,
                            line: 0,
                            column: 10,
                        }
                    }
                ))
            ));

            // err: minus sign preceding unsigned number
            assert!(matches!(
                lex_from_str("-0x1_u16"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 7,
                            line: 0,
                            column: 7,
                        }
                    }
                ))
            ));
        }

        // i32/u32
        {
            assert_eq!(
                lex_from_str_without_location("+0x7fff_ffff_i32").unwrap(),
                vec![Token::Number(NumberToken::I32(0x7fff_ffff_i32 as u32))]
            );

            assert_eq!(
                lex_from_str_without_location("-0x8000_0000_i32").unwrap(),
                vec![Token::Number(NumberToken::I32(-0x8000_0000_i32 as u32))]
            );

            // err: positive overflow
            // i32 max: 0x7fff_ffff
            assert!(matches!(
                lex_from_str("+0x8000_0000_i32"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 15,
                            line: 0,
                            column: 15,
                        }
                    }
                ))
            ));

            // err: negative overflow
            // i32 min: -0x8000_0000
            assert!(matches!(
                lex_from_str("-0x8000_0001_i32"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 15,
                            line: 0,
                            column: 15,
                        }
                    }
                ))
            ));

            // err: minus sign preceding unsigned number
            assert!(matches!(
                lex_from_str("-0x1_u32"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 7,
                            line: 0,
                            column: 7,
                        }
                    }
                ))
            ));
        }

        // i64/u64
        {
            assert_eq!(
                lex_from_str_without_location("+0x7fff_ffff_ffff_ffff_i64").unwrap(),
                vec![Token::Number(NumberToken::I64(
                    0x7fff_ffff_ffff_ffff_i64 as u64
                ))]
            );

            assert_eq!(
                lex_from_str_without_location("-0x8000_0000_0000_0000_i64").unwrap(),
                vec![Token::Number(NumberToken::I64(
                    -0x8000_0000_0000_0000_i64 as u64
                ))]
            );

            // err: positive overflow
            // i64 max: 0x7fff_ffff_ffff_ffff
            assert!(matches!(
                lex_from_str("+0x8000_0000_0000_0000_i64"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 25,
                            line: 0,
                            column: 25,
                        }
                    }
                ))
            ));

            // err: negative overflow
            // i64 min: -0x8000_0000_0000_0000
            assert!(matches!(
                lex_from_str("-0x8000_0000_0000_0001_i64"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 25,
                            line: 0,
                            column: 25,
                        }
                    }
                ))
            ));

            // err: minus sign preceding unsigned number
            assert!(matches!(
                lex_from_str("-0x1_u64"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
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

        // range

        {
            assert_eq!(
                lex_from_str("+0x11").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::I32(0x11)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 5)
                ),]
            );

            assert_eq!(
                lex_from_str("-0x13").unwrap(),
                vec![TokenWithRange::new(
                    Token::Number(NumberToken::I32(-0x13_i32 as u32)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 5)
                ),]
            );

            assert_eq!(
                lex_from_str("+0x11,-0x13").unwrap(),
                vec![
                    TokenWithRange::new(
                        Token::Number(NumberToken::I32(0x11)),
                        Range::from_position_and_length(&Position::new(0, 0, 0), 5)
                    ),
                    // TokenWithRange::new(
                    //     Token::Comma,
                    //     Range::from_position_and_length(&Position::new(5, 0, 5), 1)
                    // ),
                    TokenWithRange::new(
                        Token::Number(NumberToken::I32(-0x13_i32 as u32)),
                        Range::from_position_and_length(&Position::new(6, 0, 6), 5)
                    ),
                ]
            );
        }
    }

    #[test]
    fn test_normalize_plus_and_minus_hexadecimal_floating_point_numbers() {
        // 3.1415927f32
        assert_eq!(
            lex_from_str_without_location("+0x1.921fb6p1f32").unwrap(),
            vec![Token::Number(NumberToken::F32(std::f32::consts::PI))]
        );

        // -2.718281828459045f64
        assert_eq!(
            lex_from_str_without_location("-0x1.5bf0a8b145769p+1_f64").unwrap(),
            vec![Token::Number(NumberToken::F64(-std::f64::consts::E))]
        );

        // range

        assert_eq!(
            lex_from_str("+0x1.921fb6p1f32,-0x1.5bf0a8b145769p+1_f64").unwrap(),
            vec![
                TokenWithRange::new(
                    Token::Number(NumberToken::F32(std::f32::consts::PI)),
                    Range::from_position_and_length(&Position::new(0, 0, 0), 16)
                ),
                // TokenWithRange::new(
                //     Token::Comma,
                //     Range::from_position_and_length(&Position::new(16, 0, 16), 1)
                // ),
                TokenWithRange::new(
                    Token::Number(NumberToken::F64(-std::f64::consts::E)),
                    Range::from_position_and_length(&Position::new(17, 0, 17), 25)
                ),
            ]
        );
    }

    #[test]
    fn test_normalize_plus_and_minus_binary_numbers() {
        // implicit type, default `i32`
        {
            assert_eq!(
                lex_from_str_without_location("+0b101").unwrap(),
                vec![Token::Number(NumberToken::I32(0b101_i32 as u32))]
            );

            assert_eq!(
                lex_from_str_without_location("-0b010").unwrap(),
                vec![Token::Number(NumberToken::I32(-0b010_i32 as u32))]
            );

            // err: positive overflow
            // i32 max: 0b0111_1111_1111_1111_1111_1111_1111_1111
            assert!(matches!(
                lex_from_str("+0b1000_0000_0000_0000__0000_0000_0000_0000"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 42,
                            line: 0,
                            column: 42
                        }
                    }
                ))
            ));

            // err: negative overflow
            // i32 min: -0b1000_0000_0000_0000_0000_0000_0000_0000
            assert!(matches!(
                lex_from_str("-0b1000_0000_0000_0000__0000_0000_0000_0001"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 42,
                            line: 0,
                            column: 42
                        }
                    }
                ))
            ));
        }

        // i8/u8
        {
            assert_eq!(
                lex_from_str_without_location("0b0111_1111_i8").unwrap(),
                vec![Token::Number(NumberToken::I8(0x7f_i8 as u8))]
            );

            assert_eq!(
                lex_from_str_without_location("-0b1000_0000_i8").unwrap(),
                vec![Token::Number(NumberToken::I8(-0x80_i8 as u8))]
            );

            // err: positive overflow
            // i8 max: 0b0111_1111
            assert!(matches!(
                lex_from_str("+0b1000_0000_i8"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 14,
                            line: 0,
                            column: 14
                        }
                    }
                ))
            ));

            // err: negative overflow
            // i8 min: -0b1000_0000
            assert!(matches!(
                lex_from_str("-0b1000_0001_i8"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 14,
                            line: 0,
                            column: 14
                        }
                    }
                ))
            ));

            // err: minus sign preceding unsigned number
            assert!(matches!(
                lex_from_str("-0b1_u8"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 6,
                            line: 0,
                            column: 6
                        }
                    }
                ))
            ));
        }

        // i16/u16
        {
            assert_eq!(
                lex_from_str_without_location("+0b0111_1111_1111_1111_i16").unwrap(),
                vec![Token::Number(NumberToken::I16(0x7fff_i16 as u16))]
            );

            assert_eq!(
                lex_from_str_without_location("-0b1000_0000_0000_0000_i16").unwrap(),
                vec![Token::Number(NumberToken::I16(-0x8000_i16 as u16))]
            );

            // err: positive overflow
            // i16 max: 0b0111_1111_1111_1111
            assert!(matches!(
                lex_from_str("+0b1000_0000_0000_0000_i16"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 25,
                            line: 0,
                            column: 25
                        }
                    }
                ))
            ));

            // err: negative overflow
            // i16 min: -0b1000_0000_0000_0000
            assert!(matches!(
                lex_from_str("-0b1000_0000_0000_0001_i16"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 25,
                            line: 0,
                            column: 25
                        }
                    }
                ))
            ));

            // err: minus sign preceding unsigned number
            assert!(matches!(
                lex_from_str("-0b1_u16"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
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

        // i32/u32
        {
            assert_eq!(
                lex_from_str_without_location("+0b0111_1111_1111_1111__1111_1111_1111_1111_i32")
                    .unwrap(),
                vec![Token::Number(NumberToken::I32(0x7fff_ffff_i32 as u32))]
            );

            assert_eq!(
                lex_from_str_without_location("-0b1000_0000_0000_0000__0000_0000_0000_0000_i32")
                    .unwrap(),
                vec![Token::Number(NumberToken::I32(-0x8000_0000_i32 as u32))]
            );

            // err: positive overflow
            // i32 max: 0b0111_1111_1111_1111_1111_1111_1111_1111
            assert!(matches!(
                lex_from_str("+0b1000_0000_0000_0000__0000_0000_0000_0000_i32"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 46,
                            line: 0,
                            column: 46
                        }
                    }
                ))
            ));

            // err: negative overflow
            // i32 min: -0b1000_0000_0000_0000_0000_0000_0000_0000
            assert!(matches!(
                lex_from_str("-0b1000_0000_0000_0000__0000_0000_0000_0001_i32"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 46,
                            line: 0,
                            column: 46
                        }
                    }
                ))
            ));

            // err: minus sign preceding unsigned number
            assert!(matches!(
                lex_from_str("-0b1_u32"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
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

        // i64/u64
        {
            assert_eq!(
                lex_from_str_without_location("0b0111_1111_1111_1111__1111_1111_1111_1111__1111_1111_1111_1111__1111_1111_1111_1111_i64").unwrap(),
                vec![Token::Number(NumberToken::I64(0x7fff_ffff_ffff_ffff_i64 as u64))]
            );

            assert_eq!(
                lex_from_str_without_location("-0b1000_0000_0000_0000__0000_0000_0000_0000__0000_0000_0000_0000__0000_0000_0000_0000_i64").unwrap(),
                vec![Token::Number(NumberToken::I64(-0x8000_0000_0000_0000_i64 as u64))]
            );

            // err: positive overflow
            assert!(matches!(
                lex_from_str(
                    "+0b1000_0000_0000_0000__0000_0000_0000_0000__0000_0000_0000_0000__0000_0000_0000_0000_i64"
                ),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 88,
                            line: 0,
                            column: 88
                        }
                    }
                ))
            ));

            // err: negative overflow
            assert!(matches!(
                lex_from_str(
                    "-0b1000_0000_0000_0000__0000_0000_0000_0000__0000_0000_0000_0000__0000_0000_0000_0001_i64"
                ),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 88,
                            line: 0,
                            column: 88
                        }
                    }
                ))
            ));

            // err: minus sign preceding unsigned number
            assert!(matches!(
                lex_from_str("-0b1_u64"),
                Err(AsonError::MessageWithRange(
                    _,
                    Range {
                        start: Position {
                            index: 0,
                            line: 0,
                            column: 0,
                        },
                        end_included: Position {
                            index: 7,
                            line: 0,
                            column: 7
                        }
                    }
                ))
            ));

            // range

            {
                assert_eq!(
                    lex_from_str("+0b101").unwrap(),
                    vec![TokenWithRange::new(
                        Token::Number(NumberToken::I32(0b101_i32 as u32)),
                        Range::from_position_and_length(&Position::new(0, 0, 0), 6)
                    )]
                );

                assert_eq!(
                    lex_from_str("-0b010").unwrap(),
                    vec![TokenWithRange::new(
                        Token::Number(NumberToken::I32(-0b010_i32 as u32)),
                        Range::from_position_and_length(&Position::new(0, 0, 0), 6)
                    )]
                );

                assert_eq!(
                    lex_from_str("+0b101,-0b010").unwrap(),
                    vec![
                        TokenWithRange::new(
                            Token::Number(NumberToken::I32(0b101_i32 as u32)),
                            Range::from_position_and_length(&Position::new(0, 0, 0), 6)
                        ),
                        // TokenWithRange::new(
                        //     Token::Comma,
                        //     Range::from_position_and_length(&Position::new(6, 0, 6), 1)
                        // ),
                        TokenWithRange::new(
                            Token::Number(NumberToken::I32(-0b010_i32 as u32)),
                            Range::from_position_and_length(&Position::new(7, 0, 7), 6)
                        )
                    ]
                );
            }
        }
    }

    #[test]
    fn test_normalize_plus_and_minus_octal_numbers() {
        // todo
    }

    #[test]
    fn test_check_signed_decimal_integer_overflow() {
        // i32 max: 2_147_483_647
        assert!(matches!(
            lex_from_str("2_147_483_648"),
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
                        column: 12
                    }
                }
            ))
        ));

        // i8 max: 127
        assert!(matches!(
            lex_from_str("128_i8"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 5,
                        line: 0,
                        column: 5
                    }
                }
            ))
        ));

        // i16 max: 32_767
        assert!(matches!(
            lex_from_str("32768_i16"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 8,
                        line: 0,
                        column: 8
                    }
                }
            ))
        ));

        // i32 max: 2_147_483_647
        assert!(matches!(
            lex_from_str("2_147_483_648_i32"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 16,
                        line: 0,
                        column: 16
                    }
                }
            ))
        ));

        // i64 max: 9_223_372_036_854_775_807
        assert!(matches!(
            lex_from_str("9_223_372_036_854_775_808_i64"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 28,
                        line: 0,
                        column: 28
                    }
                }
            ))
        ));
    }

    #[test]
    fn test_check_signed_hexadecimal_integer_overflow() {
        // i32 max: 0x7fff_ffff
        assert!(matches!(
            lex_from_str("0x8000_0000"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 10,
                        line: 0,
                        column: 10
                    }
                }
            ))
        ));

        // i8 max: 0x7f
        assert!(matches!(
            lex_from_str("0x80_i8"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 6,
                        line: 0,
                        column: 6
                    }
                }
            ))
        ));

        // i16 max: 0x7fff
        assert!(matches!(
            lex_from_str("0x8000_i16"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 9,
                        line: 0,
                        column: 9
                    }
                }
            ))
        ));

        // i32 max: 0x7fff_ffff
        assert!(matches!(
            lex_from_str("0x8000_0000_i32"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 14,
                        line: 0,
                        column: 14
                    }
                }
            ))
        ));

        // i64 max: 0x7fff_ffff_ffff_ffff
        assert!(matches!(
            lex_from_str("0x8000_0000_0000_0000_i64"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 24,
                        line: 0,
                        column: 24
                    }
                }
            ))
        ));
    }

    #[test]
    fn test_check_signed_binary_integer_overflow() {
        // i32 max: 0b0111_1111_1111_1111_1111_1111_1111_1111
        assert!(matches!(
            lex_from_str("0b1000_0000_0000_0000__0000_0000_0000_0000"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 41,
                        line: 0,
                        column: 41
                    }
                }
            ))
        ));

        // i8 max: 0b0111_1111
        assert!(matches!(
            lex_from_str("0b1000_0000_i8"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 13,
                        line: 0,
                        column: 13
                    }
                }
            ))
        ));

        // i16 max: 0b0111_1111_1111_1111
        assert!(matches!(
            lex_from_str("0b1000_0000_0000_0000_i16"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 24,
                        line: 0,
                        column: 24
                    }
                }
            ))
        ));

        // i32 max: 0b0111_1111_1111_1111_1111_1111_1111_1111
        assert!(matches!(
            lex_from_str("0b1000_0000_0000_0000__0000_0000_0000_0000_i32"),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 45,
                        line: 0,
                        column: 45
                    }
                }
            ))
        ));

        assert!(matches!(
            lex_from_str(
                "0b1000_0000_0000_0000__0000_0000_0000_0000__0000_0000_0000_0000__0000_0000_0000_0000_i64"
            ),
            Err(AsonError::MessageWithRange(
                _,
                Range {
                    start: Position {
                        index: 0,
                        line: 0,
                        column: 0,
                    },
                    end_included: Position {
                        index: 87,
                        line: 0,
                        column: 87
                    }
                }
            ))
        ));
    }

    #[test]
    fn test_check_signed_octal_integer_overflow() {
        // todo
    }

    #[test]
    fn test_trim_document() {
        assert_eq!(
            lex_from_str_without_location(
                r#"

                11

                13

                "#
            )
            .unwrap(),
            vec![
                Token::Number(NumberToken::I32(11)),
                // Token::NewLine,
                Token::Number(NumberToken::I32(13)),
            ]
        );
    }
}
