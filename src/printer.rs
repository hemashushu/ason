// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::io::Write;

use chrono::{DateTime, FixedOffset};

use crate::{
    ast::{AsonNode, KeyValuePair, NamedListEntry, Number, Variant, VariantValue},
    error::AsonError,
};

pub const DEFAULT_INDENT_CHARS: &str = "    ";

fn print_number(writer: &mut dyn Write, v: &Number) -> Result<(), std::io::Error> {
    match v {
        Number::I8(v) => {
            write!(writer, "{}_i8", v)
        }
        Number::U8(v) => {
            write!(writer, "{}_u8", v)
        }
        Number::I16(v) => {
            write!(writer, "{}_i16", v)
        }
        Number::U16(v) => {
            write!(writer, "{}_u16", v)
        }
        Number::I32(v) => {
            // default integer number type
            write!(writer, "{}", v)
        }
        Number::U32(v) => {
            write!(writer, "{}_u32", v)
        }
        Number::I64(v) => {
            write!(writer, "{}_i64", v)
        }
        Number::U64(v) => {
            write!(writer, "{}_u64", v)
        }
        Number::F32(v) => {
            if v.is_nan() {
                write!(writer, "NaN_f32")
            } else if v == &f32::INFINITY {
                write!(writer, "Inf_f32")
            } else if v == &f32::NEG_INFINITY {
                write!(writer, "-Inf_f32")
            } else {
                write!(writer, "{}_f32", v)
            }
        }
        Number::F64(v) => {
            // default floating-point number type
            if v.is_nan() {
                write!(writer, "NaN")
            } else if v == &f64::INFINITY {
                write!(writer, "Inf")
            } else if v == &f64::NEG_INFINITY {
                write!(writer, "-Inf")
            } else {
                // a decimal point needs to be appended if there is no decimal point
                // in the literal.
                let mut s = v.to_string();
                if !s.contains('.') {
                    s.push_str(".0");
                }
                write!(writer, "{}", s)
            }
        }
    }
}

fn print_boolean(writer: &mut dyn Write, v: &bool) -> Result<(), std::io::Error> {
    match v {
        true => write!(writer, "true"),
        false => write!(writer, "false"),
    }
}

fn print_char(writer: &mut dyn Write, v: &char) -> Result<(), std::io::Error> {
    // escape single char
    let s = match v {
        '\\' => "\\\\".to_owned(),
        '\'' => "\\'".to_owned(),
        '\t' => {
            // horizontal tabulation
            "\\t".to_owned()
        }
        '\r' => {
            // carriage return, jump to the beginning of the line (CR)
            "\\r".to_owned()
        }
        '\n' => {
            // new line/line feed (LF)
            "\\n".to_owned()
        }
        '\0' => {
            // null char
            "\\0".to_owned()
        }
        _ => v.to_string(),
    };

    write!(writer, "'{}'", s)
}

fn print_string(writer: &mut dyn Write, v: &str) -> Result<(), std::io::Error> {
    write!(
        writer,
        "\"{}\"",
        v.chars()
            .map(|c| match c {
                '\\' => "\\\\".to_owned(),
                '"' => "\\\"".to_owned(),
                '\t' => "\\t".to_owned(),

                // null char is allowed in the source code,
                // it is used to represent the null-terminated string.
                '\0' => "\\0".to_owned(),

                _ => c.to_string(),
            })
            .collect::<Vec<String>>()
            .join("")
    )
}

fn print_date(writer: &mut dyn Write, v: &DateTime<FixedOffset>) -> Result<(), std::io::Error> {
    write!(writer, "d\"{}\"", v.to_rfc3339())
}

fn print_variant(
    writer: &mut dyn Write,
    v: &Variant,
    indent_chars: &str,
    indent_level: usize,
) -> Result<(), std::io::Error> {
    let (type_name, member_name, value) = (&v.type_name, &v.member_name, &v.value);

    match value {
        VariantValue::Empty => write!(writer, "{}::{}", type_name, member_name),
        VariantValue::Value(v) => {
            write!(writer, "{}::{}(", type_name, member_name)?;
            print_node(writer, v, indent_chars, indent_level)?;
            write!(writer, ")")
        }
        VariantValue::Tuple(v) => {
            write!(writer, "{}::{}", type_name, member_name)?;
            print_tuple(writer, v, indent_chars, indent_level)
        }
        VariantValue::Object(kvps) => {
            write!(writer, "{}::{}", type_name, member_name)?;
            print_object(writer, kvps, indent_chars, indent_level)
        }
    }
}

/// format the byte array with fixed length hex:
///
/// e.g.
///
/// h"00 11 22 33  44 55 66 77
///   88 99 aa bb  cc dd ee ff"
///
fn print_hexadecimal_byte_data(
    writer: &mut dyn Write,
    data: &[u8],
    indent_chars: &str,
) -> Result<(), std::io::Error> {
    let line_sep = format!("\n{}", indent_chars);
    let content = data
        .chunks(8)
        .map(|chunk| {
            // line
            chunk
                .iter()
                .enumerate()
                .map(|(idx, byte)| {
                    // format the bytes as the following text:
                    // 00 11 22 33  44 55 66 77
                    // 00 11 22 33
                    // 00 11
                    //
                    // Rust std format!()
                    // https://doc.rust-lang.org/std/fmt/
                    if idx == 4 {
                        format!("  {:02x}", byte)
                    } else if idx == 0 {
                        format!("{:02x}", byte)
                    } else {
                        format!(" {:02x}", byte)
                    }
                })
                .collect::<Vec<String>>()
                .join("")
        })
        .collect::<Vec<String>>()
        .join(&line_sep);

    write!(writer, "h\"{}\"", content)
}

fn print_list(
    writer: &mut dyn Write,
    v: &[AsonNode],
    indent_chars: &str,
    indent_level: usize,
) -> Result<(), std::io::Error> {
    let leading_space = indent_chars.repeat(indent_level);
    let sub_level = indent_level + 1;
    let element_leading_space = indent_chars.repeat(sub_level);

    writeln!(writer, "[")?;
    for e in v {
        write!(writer, "{}", element_leading_space)?;
        print_node(writer, e, indent_chars, sub_level)?;
        writeln!(writer)?;
    }
    write!(writer, "{}]", leading_space)
}

fn print_tuple(
    writer: &mut dyn Write,
    v: &[AsonNode],
    indent_chars: &str,
    indent_level: usize,
) -> Result<(), std::io::Error> {
    write!(writer, "(")?;
    let mut is_first_element = true;

    for e in v {
        if is_first_element {
            is_first_element = false;
        } else {
            write!(writer, ", ")?;
        }
        print_node(writer, e, indent_chars, indent_level)?;
    }
    write!(writer, ")")
}

fn print_object(
    writer: &mut dyn Write,
    v: &[KeyValuePair],
    indent_chars: &str,
    indent_level: usize,
) -> Result<(), std::io::Error> {
    let leading_space = indent_chars.repeat(indent_level);
    let sub_level = indent_level + 1;
    let element_leading_space = indent_chars.repeat(sub_level);

    writeln!(writer, "{{")?;
    for e in v {
        write!(writer, "{}{}: ", element_leading_space, e.key)?;
        print_node(writer, &e.value, indent_chars, sub_level)?;
        writeln!(writer)?;
    }
    write!(writer, "{}}}", leading_space)
}

fn print_named_list(
    writer: &mut dyn Write,
    v: &[NamedListEntry],
    indent_chars: &str,
    indent_level: usize,
) -> Result<(), std::io::Error> {
    let leading_space = indent_chars.repeat(indent_level);
    let sub_level = indent_level + 1;
    let element_leading_space = indent_chars.repeat(sub_level);

    writeln!(writer, "[")?;
    for e in v {
        write!(writer, "{}", element_leading_space)?;
        print_node(writer, &e.name, indent_chars, sub_level)?;
        write!(writer, ": ")?;
        print_node(writer, &e.value, indent_chars, sub_level)?;
        writeln!(writer)?;
    }
    write!(writer, "{}]", leading_space)
}

fn print_node(
    writer: &mut dyn Write,
    node: &AsonNode,
    indent_chars: &str,
    indent_level: usize,
) -> Result<(), std::io::Error> {
    match node {
        AsonNode::Number(v) => print_number(writer, v),
        AsonNode::Boolean(v) => print_boolean(writer, v),
        AsonNode::Char(v) => print_char(writer, v),
        AsonNode::String(v) => print_string(writer, v),
        AsonNode::DateTime(v) => print_date(writer, v),
        AsonNode::Variant(v) => print_variant(writer, v, indent_chars, indent_level),
        AsonNode::ByteData(v) => print_hexadecimal_byte_data(writer, v, indent_chars),
        AsonNode::List(v) => print_list(writer, v, indent_chars, indent_level),
        AsonNode::Tuple(v) => print_tuple(writer, v, indent_chars, indent_level),
        AsonNode::Object(v) => print_object(writer, v, indent_chars, indent_level),
        AsonNode::NamedList(v) => print_named_list(writer, v, indent_chars, indent_level),
    }
}

pub fn print_to_writer(writer: &mut dyn Write, node: &AsonNode) -> Result<(), AsonError> {
    match print_node(writer, node, DEFAULT_INDENT_CHARS, 0) {
        Ok(_) => Ok(()),
        Err(e) => Err(AsonError::Message(e.to_string())),
    }
}

pub fn print_to_string(node: &AsonNode) -> String {
    let mut buf: Vec<u8> = vec![];
    print_to_writer(&mut buf, node).unwrap();
    String::from_utf8(buf).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{AsonNode, KeyValuePair, Variant},
        parser::parse_from_str,
    };

    use super::print_to_string;
    use pretty_assertions::assert_eq;

    fn format(s: &str) -> String {
        let node = parse_from_str(s).unwrap();
        print_to_string(&node)
    }

    impl AsonNode {
        fn new_string(s: &str) -> AsonNode {
            AsonNode::String(s.to_owned())
        }
    }

    #[test]
    fn test_print_node() {
        let node = AsonNode::Object(vec![
            KeyValuePair::new("name", AsonNode::new_string("foo")),
            KeyValuePair::new(
                "type",
                AsonNode::Variant(Variant::new("Type", "Application")),
            ),
            KeyValuePair::new("version", AsonNode::new_string("0.1.0")),
            KeyValuePair::new(
                "dependencies",
                AsonNode::List(vec![
                    AsonNode::Object(vec![
                        KeyValuePair::new("name", AsonNode::new_string("random")),
                        KeyValuePair::new(
                            "version",
                            AsonNode::Variant(Variant::new("Option", "None")),
                        ),
                    ]),
                    AsonNode::Object(vec![
                        KeyValuePair::new("name", AsonNode::new_string("regex")),
                        KeyValuePair::new(
                            "version",
                            AsonNode::Variant(Variant::with_value(
                                "Option",
                                "Some",
                                AsonNode::new_string("1.0.1"),
                            )),
                        ),
                    ]),
                ]),
            ),
        ]);

        let text = print_to_string(&node);

        assert_eq!(
            text,
            r#"{
    name: "foo"
    type: Type::Application
    version: "0.1.0"
    dependencies: [
        {
            name: "random"
            version: Option::None
        }
        {
            name: "regex"
            version: Option::Some("1.0.1")
        }
    ]
}"#
        );
    }

    #[test]
    fn test_print_primitive_value() {
        assert_eq!(
            format(
                r#"
            123
            "#
            ),
            "123"
        );

        assert_eq!(
            format(
                r#"
            1.23
            "#
            ),
            "1.23"
        );

        assert_eq!(
            format(
                r#"
            123f64
            "#
            ),
            "123.0"
        );

        assert_eq!(
            format(
                r#"
            123f32
            "#
            ),
            "123_f32"
        );

        assert_eq!(
            format(
                r#"
            true
            "#
            ),
            "true"
        );

        assert_eq!(
            format(
                r#"
            '🍒'
            "#
            ),
            "'🍒'"
        );

        assert_eq!(
            format(
                r#"
            '\n'
            "#
            ),
            "'\\n'"
        );

        assert_eq!(
            format(
                r#"
            "hello\"world"
            "#
            ),
            "\"hello\\\"world\""
        );
    }

    #[test]
    fn test_print_datetime() {
        assert_eq!(
            format(
                r#"
            d"2024-03-17 10:01:11+08:00"
            "#
            ),
            "d\"2024-03-17T10:01:11+08:00\""
        );
    }

    #[test]
    fn test_print_hexadecimal_byte_data() {
        assert_eq!(
            format(
                r#"
            h"11 13 17 19 23 29 31 37 41 43 47 53 59 61 67 71 73 79"
            "#
            ),
            "h\"11 13 17 19  23 29 31 37
    41 43 47 53  59 61 67 71
    73 79\""
        );
    }

    #[test]
    fn test_print_object() {
        assert_eq!(
            format(
                r#"
            {id:123,name:"foo"}
            "#
            ),
            r#"{
    id: 123
    name: "foo"
}"#
        );

        assert_eq!(
            format(
                r#"
            {id:123,name:{first:"foo", last:"bar"}}
            "#
            ),
            r#"{
    id: 123
    name: {
        first: "foo"
        last: "bar"
    }
}"#
        );

        assert_eq!(
            format(
                r#"
                    {id:123,name:Option::Some({first:"foo", last:"bar"}),result:Result::Ok(456)}
                    "#
            ),
            r#"{
    id: 123
    name: Option::Some({
        first: "foo"
        last: "bar"
    })
    result: Result::Ok(456)
}"#
        );
    }

    #[test]
    fn test_print_named_list() {
        assert_eq!(
            format(
                r#"
            [123: "foo", 456: "hello"]
            "#
            ),
            r#"[
    123: "foo"
    456: "hello"
]"#
        );
    }

    #[test]
    fn test_print_list() {
        assert_eq!(
            format(
                r#"
            [123,456,789]
            "#
            ),
            r#"[
    123
    456
    789
]"#
        );

        assert_eq!(
            format(
                r#"
            [{id:123, name:"abc"},{id:456, name:"def"},{id:789,name:"xyz"}]
            "#
            ),
            r#"[
    {
        id: 123
        name: "abc"
    }
    {
        id: 456
        name: "def"
    }
    {
        id: 789
        name: "xyz"
    }
]"#
        );
    }

    #[test]
    fn test_print_tuple() {
        assert_eq!(
            format(
                r#"
            (123,"foo",true)
            "#
            ),
            "(123, \"foo\", true)"
        );
    }

    #[test]
    fn test_print_variant() {
        assert_eq!(format(r#"Option::None"#), "Option::None");
        assert_eq!(format(r#"Option::Some(123)"#), "Option::Some(123)");
        assert_eq!(
            format(r#"Color::RGB(255,200,100)"#),
            "Color::RGB(255, 200, 100)"
        );
        assert_eq!(
            format(r#"Shape::Rect{width:11, height:13}"#),
            r#"Shape::Rect{
    width: 11
    height: 13
}"#
        );
    }
}
