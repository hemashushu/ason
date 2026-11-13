// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::io::{self, Cursor, Read};

use ason::{Value, VariantValue};

struct OneByteReader<R>(R);

impl<R: Read> Read for OneByteReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let length = buffer.len().min(1);
        self.0.read(&mut buffer[..length])
    }
}

struct FailingReader {
    bytes: &'static [u8],
}

impl Read for FailingReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }

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
fn test_round_trip_nested_value_through_public_apis() {
    let document = r#"{
        name: "Ada 文"
        score: -12_i16
        ratio: 1.5_f32
        values: [255_u8 "two" None Some(true)]
        map: ["first": 1 "second": 2]
        pair: (1 "two")
        bytes: h"00 ff"
        time: d"2024-03-16T16:30:50+08:00"
        status: State::Ready
        color: Color::Rgb(255_u8 127_u8 63_u8)
        shape: Shape::Rect{width: 2 height: 3}
        result: Result::Err("failed")
    }"#;
    let expected = Value::Object(vec![
        ("name".into(), Value::String("Ada 文".into())),
        ("score".into(), Value::I16(-12)),
        ("ratio".into(), Value::F32(1.5)),
        (
            "values".into(),
            Value::List(vec![
                Value::U8(255),
                Value::String("two".into()),
                Value::Option(None),
                Value::Option(Some(Box::new(Value::Bool(true)))),
            ]),
        ),
        (
            "map".into(),
            Value::Map(vec![
                (Value::String("first".into()), Value::I32(1)),
                (Value::String("second".into()), Value::I32(2)),
            ]),
        ),
        (
            "pair".into(),
            Value::Tuple(vec![Value::I32(1), Value::String("two".into())]),
        ),
        ("bytes".into(), Value::Bytes(vec![0, 0xff])),
        (
            "time".into(),
            Value::DateTime(
                chrono::DateTime::parse_from_rfc3339("2024-03-16T16:30:50+08:00").unwrap(),
            ),
        ),
        (
            "status".into(),
            Value::Enumeration {
                name: "State".into(),
                variant: "Ready".into(),
                payload: VariantValue::Unit,
            },
        ),
        (
            "color".into(),
            Value::Enumeration {
                name: "Color".into(),
                variant: "Rgb".into(),
                payload: VariantValue::Tuple(vec![Value::U8(255), Value::U8(127), Value::U8(63)]),
            },
        ),
        (
            "shape".into(),
            Value::Enumeration {
                name: "Shape".into(),
                variant: "Rect".into(),
                payload: VariantValue::Struct(vec![
                    ("width".into(), Value::I32(2)),
                    ("height".into(), Value::I32(3)),
                ]),
            },
        ),
        (
            "result".into(),
            Value::Enumeration {
                name: "Result".into(),
                variant: "Err".into(),
                payload: VariantValue::Newtype(Box::new(Value::String("failed".into()))),
            },
        ),
    ]);

    assert_eq!(ason::de_value_from_str(document).unwrap(), expected);
    let decoded =
        ason::de_value_from_reader(OneByteReader(Cursor::new(document.as_bytes()))).unwrap();
    assert_eq!(decoded, expected);
    let Value::Object(fields) = &decoded else {
        panic!("expected an object");
    };
    let Value::DateTime(time) = &fields.iter().find(|(name, _)| name == "time").unwrap().1 else {
        panic!("expected a DateTime");
    };
    assert_eq!(time.offset().local_minus_utc(), 8 * 60 * 60);

    let serialized = ason::ser_value_to_string(&expected).unwrap();
    let mut output = Vec::new();
    ason::ser_value_to_writer(&expected, &mut output).unwrap();
    assert_eq!(output, serialized.as_bytes());
    assert_eq!(ason::de_value_from_str(&serialized).unwrap(), expected);
    assert_eq!(
        ason::de_value_from_reader(output.as_slice()).unwrap(),
        expected
    );
}

#[test]
fn test_serialize_value_to_string_and_writer() {
    let value = Value::Object(vec![
        ("name".into(), Value::String("Ada".into())),
        ("score".into(), Value::I16(-12)),
        (
            "enabled".into(),
            Value::Option(Some(Box::new(Value::Bool(true)))),
        ),
    ]);
    let expected = "{\n    name: \"Ada\"\n    score: -12_i16\n    enabled: Some(true)\n}";

    assert_eq!(ason::ser_value_to_string(&value).unwrap(), expected);

    let mut output = b"prefix ".to_vec();
    ason::ser_value_to_writer(&value, &mut output).unwrap();
    assert_eq!(output, format!("prefix {expected}").as_bytes());
}

#[test]
fn test_empty_map_is_parsed_as_an_empty_list() {
    let serialized = ason::ser_value_to_string(&Value::Map(vec![])).unwrap();
    assert_eq!(serialized, "[]");
    assert_eq!(
        ason::de_value_from_str(&serialized).unwrap(),
        Value::List(vec![])
    );
    assert_eq!(
        ason::de_value_from_reader(serialized.as_bytes()).unwrap(),
        Value::List(vec![])
    );
}

#[test]
fn test_value_deserialization_rejects_invalid_documents() {
    for input in [
        "[1",
        "1 true",
        "{name: 1 name: 2}",
        "Some(1 2)",
        "Result::Err",
        "()",
    ] {
        assert!(ason::de_value_from_str(input).is_err(), "{input}");
        assert!(
            ason::de_value_from_reader(input.as_bytes()).is_err(),
            "{input}"
        );
    }
}

#[test]
fn test_value_reader_reports_utf8_and_io_errors() {
    for input in [&b"\"\xff\""[..], &b"\"\xe2\x82"[..]] {
        let error = ason::de_value_from_reader(input).unwrap_err();
        assert!(error.to_string().contains("UTF-8"), "{error}");
    }

    let error = ason::de_value_from_reader(FailingReader { bytes: b"42" }).unwrap_err();
    assert!(error.to_string().contains("test read failure"), "{error}");
}

#[test]
fn test_value_serialization_reports_errors() {
    for value in [
        Value::Tuple(vec![]),
        Value::Enumeration {
            name: "Result".into(),
            variant: "Ok".into(),
            payload: VariantValue::Unit,
        },
    ] {
        assert!(ason::ser_value_to_string(&value).is_err(), "{value:?}");
        let mut output = Vec::new();
        assert!(
            ason::ser_value_to_writer(&value, &mut output).is_err(),
            "{value:?}"
        );
    }

    let mut output = [0; 2];
    let mut writer = Cursor::new(output.as_mut_slice());
    assert!(ason::ser_value_to_writer(&Value::String("hello".into()), &mut writer).is_err());
    assert_eq!(&output, b"\"h");
}

#[test]
fn test_write_value_in_list_streams() {
    struct NonSerde;

    let values = vec![Value::I32(1), Value::String("two".into())];
    let mut output = Vec::new();
    {
        let mut stream = ason::list_stream_writer::<NonSerde, _>(&mut output);
        stream.start_list().unwrap();
        for value in &values {
            stream.write_value(value).unwrap();
        }
        stream.end_list().unwrap();
    }
    assert_eq!(output, b"[\n    1\n    \"two\"\n]");
    assert_eq!(
        ason::de_value_from_reader(output.as_slice()).unwrap(),
        Value::List(values)
    );

    let mut partial = Vec::new();
    {
        let mut stream = ason::list_stream_writer::<NonSerde, _>(&mut partial);
        stream.start_list().unwrap();
        assert!(stream.write_value(&Value::Tuple(vec![])).is_err());
    }
    assert_eq!(partial, b"[\n    ");
}
