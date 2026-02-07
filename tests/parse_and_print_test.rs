// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use pretty_assertions::assert_eq;
use std::path::PathBuf;

use ason::{parse_from_str, print_to_string};

/// Helpers to get the path to the examples directory.
fn get_examples_file_directory() -> PathBuf {
    // `env::current_dir()` returns the current Rust project's root folder
    //
    // If there are multiple crates in the workspace.
    // - In the `cargo test` context, the `current_dir()` returns the current crate path.
    // - In the VSCode editor `Run Test` context, the `current_dir()` returns the current crate path.
    // - In the VSCode editor `Debug` context, the `current_dir()` always returns the workspace root folder.
    let mut dir = std::env::current_dir().unwrap();
    dir.push("examples");
    dir
}

/// Helpers to read an example file content to a string.
fn read_example_file_to_string(filename: &str) -> String {
    let mut dir = get_examples_file_directory();
    dir.push(filename);
    std::fs::read_to_string(&dir).unwrap()
}

#[test]
fn test_example_file_primitive() {
    let text = read_example_file_to_string("01-primitive.ason");
    let root_node = parse_from_str(&text).unwrap();
    let document = print_to_string(&root_node);

    assert_eq!(
        &document,
        r#"{
    integer: 123
    integer_negative: -123
    byte: 11_i8
    short: 13_i16
    int: 17
    long: 19_i64
    unsigned_byte: 23_u8
    unsigned_short: 29_u16
    unsigned_int: 31_u32
    unsigned_long: 37_u64
    floating_point: 3.14
    floating_point_with_exponent: 602200000000000000000000.0
    floating_point_with_negative_exponent: 0.000000000066738
    single_precision: 3.14_f32
    double_precision: 6.626
    hexadecimal_integer: 48879
    hexadecimal_integer_negative: -48879
    hexadecimal_byte: 127_i8
    hexadecimal_short: 32767_i16
    hexadecimal_int: 2147483647
    hexadecimal_long: 9223372036854775807_i64
    hexadecimal_unsigned_byte: 255_u8
    hexadecimal_unsigned_short: 65535_u16
    hexadecimal_unsigned_int: 4294967295_u32
    hexadecimal_unsigned_long: 18446744073709551615_u64
    hexadecimal_floating_point: 10.0
    hexadecimal_single_precison: 3.1415927_f32
    hexadecimal_double_precison: 2.718281828459045
    binary_integer: 9
    binary_integer_negative: -9
    binary_byte: 127_i8
    binary_short: 32767_i16
    binary_int: 2147483647
    binary_long: 9223372036854775807_i64
    binary_unsigned_byte: 255_u8
    binary_unsigned_short: 65535_u16
    binary_unsigned_int: 4294967295_u32
    binary_unsigned_long: 18446744073709551615_u64
    octal_integer: 493
    octal_integer_negative: -493
    octal_byte: 127_i8
    octal_short: 32767_i16
    octal_int: 2147483647
    octal_long: 9223372036854775807_i64
    octal_unsigned_byte: 255_u8
    octal_unsigned_short: 65535_u16
    octal_unsigned_int: 4294967295_u32
    octal_unsigned_long: 18446744073709551615_u64
    boolean_true: true
    boolean_false: false
    datatime: d"2023-02-23T10:23:45+00:00"
    datatime_with_timezone: d"2023-02-23T10:23:45+08:00"
    datatime_rfc3339: d"2023-02-23T10:23:45+08:00"
    datatime_rfc3339_zero_timezone: d"2023-02-23T10:23:45+00:00"
    char: 'c'
    char_unicode: '文'
    char_emoji: '🍋'
    char_escaped: '\n'
    char_escaped_zero: '\0'
    char_escaped_unicode: '河'
    string: "hello world"
    string_unicode: "中文🍀emoji👋🏻"
    multiline_string: "one
        two
        three"
    multiline_string_with_new_line_escaped: "onetwothree"
    string_with_escaped_chars: "double quote:\"
        single quote:'
        slash:\\
        tab:\t
        line feed:
"
    string_with_escaped_unicode: "河马"
    raw_string: "hello"
    raw_string_with_hash: "hello \"programming\" world"
    auto_trimmed_string: "heading 1
    heading 2
        heading 3"
    new_line: "value1"
    new_line_variant: "value2"
    space: "value3"
    line_comment: 101
    line_comment_in_tail: 103
    block_comment: 107
    multiline_block_comment: 109
    document_comment: 113
    inline_comma_1: 211
    inline_comma_2: 223
    inline_comma_3: 227
    space_separated_1: 307
    space_separated_2: 311
    space_separated_3: 313
    tail_comma_1: 401
    tail_comma_2: 409
    tail_comma_3: 419
}"#
    )
}

#[test]
fn test_example_file_list() {
    let text = read_example_file_to_string("02-list.ason");
    let root_node = parse_from_str(&text).unwrap();
    let document = print_to_string(&root_node);

    assert_eq!(
        document,
        r#"{
    number_array: [
        1
        2
        3
    ]
    string_array: [
        "one"
        "two"
        "three"
    ]
    array_with_trailing_comma: [
        1
        2
        3
        4
    ]
    array_with_space_separator: [
        "one"
        "two"
        "three"
        "four"
    ]
    mulitline_array: [
        1
        2
        3
    ]
    mulitline_array_with_commas: [
        1
        2
        3
    ]
    mulitline_array_with_trailing_comma: [
        1
        2
        3
    ]
}"#
    );
}

#[test]
fn test_example_file_tuple() {
    let text = read_example_file_to_string("03-tuple.ason");
    let root_node = parse_from_str(&text).unwrap();
    let document = print_to_string(&root_node);

    assert_eq!(
        document,
        r#"{
    tuple: (1, "foo", true)
    tuple_with_trailing_comma: (1, "foo", true)
    tuple_with_space_separator: (1, "foo", true)
    mulitline_tuple: (1, "foo", true)
    mulitline_tuple_with_commas: (1, "foo", true)
    mulitline_tuple_with_trailing_comma: (1, "foo", true)
}"#
    );
}

#[test]
fn test_example_file_object() {
    let text = read_example_file_to_string("04-object.ason");
    let root_node = parse_from_str(&text).unwrap();
    let document = print_to_string(&root_node);

    assert_eq!(
        document,
        r#"{
    id: 123
    name: "hello"
    orders: [
        (1, "foo", true)
        (2, "bar", false)
    ]
    group: {
        active: true
        permissions: [
            {
                number: 11
                title: "read"
            }
            {
                number: 13
                title: "write"
            }
        ]
    }
}"#
    );
}

#[test]
fn test_example_file_named_list() {
    let text = read_example_file_to_string("05-named-list.ason");
    let root_node = parse_from_str(&text).unwrap();
    let document = print_to_string(&root_node);

    assert_eq!(
        document,
        r#"{
    modules: [
        "foo": {
            version: "1.0"
            repo: "default"
        }
        "bar": {
            version: "2.0"
        }
    ]
    orders: [
        123: [
            1
            2
            3
        ]
        456: [
            4
            5
            6
        ]
    ]
}"#
    );
}

#[test]
fn test_example_file_variant() {
    let text = read_example_file_to_string("06-variant.ason");
    let root_node = parse_from_str(&text).unwrap();
    let document = print_to_string(&root_node);

    assert_eq!(
        document,
        r#"{
    variant_without_value: Option::None
    variant_single_value: Option::Some(123)
    variant_tuple_like: Color::RGB(255, 127, 63)
    variant_object_like: Shape::Rect{
        width: 200
        height: 100
    }
    variant_single_value: Option::Some((11, 13))
    variant_single_value: Option::Some([
        17
        19
        23
        29
    ])
    variant_single_value: Option::Some({
        id: 123
        name: "foo"
    })
}"#
    );
}
