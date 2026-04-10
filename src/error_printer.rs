// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use crate::error::AsonError;

// A struct representing a range of a snippet in the source text.
//
// ```diagram
//                 /-- snippet offset in source text (snippet is a substring of source text)
//                 |
//                 |            |-- snippet length
//                 v            v
// prefix -->   ...sni[ppet]_text...  <-- suffix
//                     ^^^^
//                     |  |-- highlight length
//                     |
//                     \----- highlight offset in snippet text
// ```
struct SnippetAndIndication {
    prefix_ellipsis: bool, // Whether the snippet should start with an ellipsis.
    suffix_ellipsis: bool, // Whether the snippet should end with an ellipsis.
    snippet_offset_in_source: usize, // The starting index of the snippet in the source text.
    snippet_length: usize, // The length of the snippet in the source text.
    highlight_offset_in_snippet: usize, // The offset of the highlight within the snippet.
    highlight_length: usize, // The length of the highlight within the snippet.
}

/// Calculates the snippet and highlight indication based on the position and length
/// of the highlighted text in the source.
fn calculate_snippet_and_indication(
    highlight_text_start_in_source: usize,
    highlight_text_length: usize,
    source_text_length: usize,
) -> SnippetAndIndication {
    // ```diagram
    //
    //     |-- leading text       |-- trailing text
    //     v                      v
    // |--------------|xxxxx|-----------------| <-- snippet text
    //                   ^
    //                   |-- highlight text
    // ```
    const LEADING_LENGTH: usize = 15;
    const SNIPPET_LENGTH: usize = 40;

    let (prefix_ellipsis, snippet_offset_in_source, highlight_offset_in_snippet) =
        if source_text_length < SNIPPET_LENGTH || highlight_text_start_in_source < LEADING_LENGTH {
            (false, 0, highlight_text_start_in_source)
        } else if highlight_text_start_in_source + SNIPPET_LENGTH > source_text_length {
            (
                true,
                source_text_length - SNIPPET_LENGTH,
                highlight_text_start_in_source - (source_text_length - SNIPPET_LENGTH),
            )
        } else {
            (
                true,
                highlight_text_start_in_source - LEADING_LENGTH,
                LEADING_LENGTH,
            )
        };

    let (suffix_ellipsis, snippet_length) =
        if snippet_offset_in_source + SNIPPET_LENGTH >= source_text_length {
            (false, source_text_length - snippet_offset_in_source)
        } else {
            (true, SNIPPET_LENGTH)
        };

    let highlight_length = if highlight_offset_in_snippet + highlight_text_length > snippet_length {
        snippet_length - highlight_offset_in_snippet
    } else {
        highlight_text_length
    };

    SnippetAndIndication {
        prefix_ellipsis,
        suffix_ellipsis,
        snippet_offset_in_source,
        snippet_length,
        highlight_offset_in_snippet,
        highlight_length,
    }
}

/// Generates a formatted error message with a snippet and highlight indication
/// based on the source text and the error message.
fn generate_error_text_paragraph(
    source_chars: &mut dyn Iterator<Item = char>,
    snippet_and_indication: &SnippetAndIndication,
    error_message: &str,
) -> (
    /* snippet line */ String,
    /* indication line */ String,
) {
    // Build the first line: the snippet
    let mut snippet = String::new();
    snippet.push_str("| ");

    if snippet_and_indication.prefix_ellipsis {
        snippet.push_str("...");
    }

    let selection_chars = source_chars
        .skip(snippet_and_indication.snippet_offset_in_source)
        .take(snippet_and_indication.snippet_length);
    let selection_string = selection_chars
        .map(|c| match c {
            '\n' => ' ',
            '\t' => ' ',
            _ => c,
        })
        .collect::<String>();
    snippet.push_str(&selection_string);

    if snippet_and_indication.suffix_ellipsis {
        snippet.push_str("...");
    }

    // Build the second line: the highlight indication
    let mut indication = String::new();
    indication.push_str("| ");

    if snippet_and_indication.prefix_ellipsis {
        indication.push_str("   ");
    }

    indication.push_str(&" ".repeat(snippet_and_indication.highlight_offset_in_snippet));
    indication.push('^');
    if snippet_and_indication.highlight_length > 0 {
        indication.push_str(&"^".repeat(snippet_and_indication.highlight_length - 1));
    }

    indication.push_str("___");
    indication.push(' ');
    indication.push_str(error_message);

    (snippet, indication)
}

impl AsonError {
    pub fn with_source(&self, source_text: &str) -> String {
        let source_text_length = source_text.chars().count();
        let mut chars = source_text.chars();

        match self {
            AsonError::Message(msg) => msg.to_owned(),
            AsonError::UnexpectedEndOfDocument(msg) => {
                let title = "Unexpected to reach the end of document.";
                let snippet_and_indication =
                    calculate_snippet_and_indication(source_text_length, 0, source_text_length);
                let (snippet_line, indication_line) =
                    generate_error_text_paragraph(&mut chars, &snippet_and_indication, msg);
                format!("{}\n{}\n{}", title, snippet_line, indication_line)
            }
            AsonError::MessageWithPosition(msg, position) => {
                let title = format!(
                    "Error at line: {}, column: {}",
                    position.line + 1,
                    position.column + 1
                );

                let snippet_range =
                    calculate_snippet_and_indication(position.index, 0, source_text_length);
                let (snippet_line, indication_line) =
                    generate_error_text_paragraph(&mut chars, &snippet_range, msg);
                format!("{}\n{}\n{}", title, snippet_line, indication_line)
            }
            AsonError::MessageWithRange(msg, range) => {
                let title = format!(
                    "Error at line: {}, column: {}",
                    range.start.line + 1,
                    range.start.column + 1,
                );

                let snippet_range = calculate_snippet_and_indication(
                    range.start.index,
                    range.end_inclusive.index - range.start.index + 1,
                    source_text_length,
                );
                let (snippet_line, indication_line) =
                    generate_error_text_paragraph(&mut chars, &snippet_range, msg);
                format!("{}\n{}\n{}", title, snippet_line, indication_line)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{error::AsonError, position::Position, range::Range};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_error_with_source_and_message() {
        let source1 = "0123456789"; // 10 chars
        let source2 = "012345678_b12345678_c12345678_d12345678_e123456789"; // 50 chars
        let msg = "abcde";

        assert_eq!(AsonError::Message(msg.to_owned()).with_source(source1), msg);
        assert_eq!(AsonError::Message(msg.to_owned()).with_source(source2), msg);
    }

    #[test]
    fn test_error_with_source_and_unexpected_end_of_document() {
        let source1 = "0123456789"; // 10 chars
        let source2 = "012345678_b12345678_c12345678_d12345678_e123456789"; // 50 chars
        let msg = "abcde";

        assert_eq!(
            AsonError::UnexpectedEndOfDocument(msg.to_owned()).with_source(source1),
            r#"Unexpected to reach the end of document.
| 0123456789
|           ^___ abcde"#
        );

        assert_eq!(
            AsonError::UnexpectedEndOfDocument(msg.to_owned()).with_source(source2),
            r#"Unexpected to reach the end of document.
| ...b12345678_c12345678_d12345678_e123456789
|                                            ^___ abcde"#
        );
    }

    #[test]
    fn test_error_with_source_and_position() {
        let source1 = "0123456789"; // 10 chars
        let source2 = "012345678_b12345678_c12345678_d12345678_e123456789"; // 50 chars
        let msg = "abcde";

        // position at the first character

        assert_eq!(
            AsonError::MessageWithPosition(msg.to_owned(), Position::new(0, 11, 13))
                .with_source(source1),
            r#"Error at line: 12, column: 14
| 0123456789
| ^___ abcde"#
        );

        assert_eq!(
            AsonError::MessageWithPosition(msg.to_owned(), Position::new(0, 11, 13))
                .with_source(source2),
            r#"Error at line: 12, column: 14
| 012345678_b12345678_c12345678_d12345678_...
| ^___ abcde"#
        );

        // position at the head

        assert_eq!(
            AsonError::MessageWithPosition(msg.to_owned(), Position::new(2, 11, 13))
                .with_source(source1),
            r#"Error at line: 12, column: 14
| 0123456789
|   ^___ abcde"#
        );

        assert_eq!(
            AsonError::MessageWithPosition(msg.to_owned(), Position::new(15, 11, 13))
                .with_source(source2),
            r#"Error at line: 12, column: 14
| ...b12345678_c12345678_d12345678_e123456789
|         ^___ abcde"#
        );

        // position at the body

        assert_eq!(
            AsonError::MessageWithPosition(msg.to_owned(), Position::new(5, 11, 13))
                .with_source(source1),
            r#"Error at line: 12, column: 14
| 0123456789
|      ^___ abcde"#
        );

        assert_eq!(
            AsonError::MessageWithPosition(msg.to_owned(), Position::new(25, 11, 13))
                .with_source(source2),
            r#"Error at line: 12, column: 14
| ...b12345678_c12345678_d12345678_e123456789
|                   ^___ abcde"#
        );

        // position at the tail

        assert_eq!(
            AsonError::MessageWithPosition(msg.to_owned(), Position::new(8, 11, 13))
                .with_source(source1),
            r#"Error at line: 12, column: 14
| 0123456789
|         ^___ abcde"#
        );

        assert_eq!(
            AsonError::MessageWithPosition(msg.to_owned(), Position::new(45, 11, 13))
                .with_source(source2),
            r#"Error at line: 12, column: 14
| ...b12345678_c12345678_d12345678_e123456789
|                                       ^___ abcde"#
        );

        // position at the last character

        assert_eq!(
            AsonError::MessageWithPosition(msg.to_owned(), Position::new(10, 11, 13))
                .with_source(source1),
            r#"Error at line: 12, column: 14
| 0123456789
|           ^___ abcde"#
        );

        assert_eq!(
            AsonError::MessageWithPosition(msg.to_owned(), Position::new(50, 11, 13))
                .with_source(source2),
            r#"Error at line: 12, column: 14
| ...b12345678_c12345678_d12345678_e123456789
|                                            ^___ abcde"#
        );
    }

    #[test]
    fn test_error_with_source_and_range() {
        let source1 = "0123456789"; // 10 chars
        let source2 = "012345678_b12345678_c12345678_d12345678_e123456789"; // 50 chars
        let msg = "abcde";

        // range at the first character

        assert_eq!(
            AsonError::MessageWithRange(msg.to_owned(), Range::from_detail(0, 17, 19, 4))
                .with_source(source1),
            r#"Error at line: 18, column: 20
| 0123456789
| ^^^^___ abcde"#
        );

        assert_eq!(
            AsonError::MessageWithRange(msg.to_owned(), Range::from_detail(0, 17, 19, 8))
                .with_source(source2),
            r#"Error at line: 18, column: 20
| 012345678_b12345678_c12345678_d12345678_...
| ^^^^^^^^___ abcde"#
        );

        // range at the head

        assert_eq!(
            AsonError::MessageWithRange(msg.to_owned(), Range::from_detail(2, 17, 19, 4))
                .with_source(source1),
            r#"Error at line: 18, column: 20
| 0123456789
|   ^^^^___ abcde"#
        );

        assert_eq!(
            AsonError::MessageWithRange(msg.to_owned(), Range::from_detail(15, 17, 19, 8))
                .with_source(source2),
            r#"Error at line: 18, column: 20
| ...b12345678_c12345678_d12345678_e123456789
|         ^^^^^^^^___ abcde"#
        );

        // range at the body

        assert_eq!(
            AsonError::MessageWithRange(msg.to_owned(), Range::from_detail(5, 17, 19, 4))
                .with_source(source1),
            r#"Error at line: 18, column: 20
| 0123456789
|      ^^^^___ abcde"#
        );

        assert_eq!(
            AsonError::MessageWithRange(msg.to_owned(), Range::from_detail(25, 17, 19, 8))
                .with_source(source2),
            r#"Error at line: 18, column: 20
| ...b12345678_c12345678_d12345678_e123456789
|                   ^^^^^^^^___ abcde"#
        );

        // range at the tail

        assert_eq!(
            AsonError::MessageWithRange(msg.to_owned(), Range::from_detail(8, 17, 19, 4))
                .with_source(source1),
            r#"Error at line: 18, column: 20
| 0123456789
|         ^^___ abcde"#
        );

        assert_eq!(
            AsonError::MessageWithRange(msg.to_owned(), Range::from_detail(45, 17, 19, 8))
                .with_source(source2),
            r#"Error at line: 18, column: 20
| ...b12345678_c12345678_d12345678_e123456789
|                                       ^^^^^___ abcde"#
        );

        // range at the last character

        assert_eq!(
            AsonError::MessageWithRange(msg.to_owned(), Range::from_detail(10, 17, 19, 4))
                .with_source(source1),
            r#"Error at line: 18, column: 20
| 0123456789
|           ^___ abcde"#
        );

        assert_eq!(
            AsonError::MessageWithRange(msg.to_owned(), Range::from_detail(50, 17, 19, 8))
                .with_source(source2),
            r#"Error at line: 18, column: 20
| ...b12345678_c12345678_d12345678_e123456789
|                                            ^___ abcde"#
        );
    }
}
