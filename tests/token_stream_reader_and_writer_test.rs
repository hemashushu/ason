// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use ason::token::{NumberToken, Token};

#[test]
fn test_token_stream_read() {
    let text = r#"{
    id: 123
}"#;

    // Create a token stream reader from the ASON document string.
    let mut reader = ason::token_stream_reader::stream_from_str(text);

    // Function `reader.next()` returns `Option<Result<Token, AsonError>>`,
    // where `Option::None` indicates the end of the stream,
    // `Some(Err)` indicates a lexing error, and `Some(Ok)` contains the next token.

    // The first token should be the opening curly brace `{`.
    let first_result = reader.next().unwrap();
    assert!(first_result.is_ok());

    let first_token = first_result.unwrap();
    assert_eq!(first_token, Token::OpeningBrace);

    // The next token should be the identifier `id`.
    assert_eq!(
        reader.next().unwrap().unwrap(),
        Token::Identifier("id".to_string())
    );

    // The next token should be the colon `:`.
    assert_eq!(reader.next().unwrap().unwrap(), Token::Colon);

    // The next token should be the number `123`, which is of type `i32` by default.
    assert_eq!(
        reader.next().unwrap().unwrap(),
        Token::Number(NumberToken::I32(123))
    );

    // The last token should be the closing curly brace `}`.
    assert_eq!(reader.next().unwrap().unwrap(), Token::ClosingBrace);

    // There should be no more tokens.
    assert!(reader.next().is_none());
}

#[test]
fn test_token_stream_write() -> std::io::Result<()> {
    let text = r#"{
    id: 123
}"#;

    // Create an output stream (can be a file, network stream, or in-memory buffer).
    // Here we use `Vec<u8>` for testing.
    let mut output = Vec::new();

    // Create a token stream writer and write some tokens to the output stream.
    let mut writer = ason::token_stream_writer::TokenStreamWriter::new(&mut output);

    writer.print_opening_brace()?;
    writer.print_token(&Token::Identifier("id".to_owned()))?;
    writer.print_colon()?;
    writer.print_space()?;
    writer.print_token(&Token::Number(NumberToken::I32(123)))?;
    writer.print_closing_brace()?;

    // Verify the output
    let document = String::from_utf8(output).unwrap();
    assert_eq!(document, text);

    Ok(())
}
