// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::{
    cell::Cell,
    fmt::Debug,
    io::{self, BufWriter, Cursor, Read, Write},
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Object {
    id: i32,
    name: String,
}

#[track_caller]
fn assert_ok_round_trip<T>(values: &[T], document: &str)
where
    T: Serialize + DeserializeOwned + Debug + PartialEq,
{
    let mut output = Vec::new();
    let writer: ason::ResultStreamWriter<'_, _, T> = ason::result_stream_writer(&mut output);
    let mut stream = writer.start_ok().unwrap();
    stream.writes(&[]).unwrap();
    if let Some((first, rest)) = values.split_first() {
        stream.write(first).unwrap();
        stream.writes(rest).unwrap();
    }
    stream.writes(&[]).unwrap();
    stream.end_list().unwrap();

    assert_eq!(output, document.as_bytes());
    assert_eq!(
        ason::ser_to_string(&Ok::<_, String>(values)).unwrap(),
        document
    );
    let decoded = ason::de_from_reader::<Result<Vec<T>, String>, _>(output.as_slice()).unwrap();
    assert_eq!(decoded.unwrap(), values);

    let reader: ason::ResultStreamReader<T, String, _> =
        ason::result_stream_reader(Cursor::new(output)).unwrap();
    let ason::ResultStreamReader::Ok(mut reader) = reader else {
        panic!("expected a streamed List");
    };
    assert_eq!(
        reader.by_ref().collect::<Result<Vec<_>, _>>().unwrap(),
        values
    );
    assert!(reader.next().is_none());
    assert!(reader.next().is_none());
}

#[track_caller]
fn assert_err_round_trip<E>(error: &E, document: &str)
where
    E: Serialize + DeserializeOwned + Debug + PartialEq,
{
    let mut output = Vec::new();
    ason::result_stream_writer::<i32, _>(&mut output)
        .write_err(error)
        .unwrap();
    assert_eq!(output, document.as_bytes());
    assert_eq!(
        ason::ser_to_string(&Err::<Vec<i32>, _>(error)).unwrap(),
        document
    );

    let decoded = ason::de_from_reader::<Result<Vec<i32>, E>, _>(output.as_slice()).unwrap();
    assert_eq!(&decoded.unwrap_err(), error);
    match ason::result_stream_reader::<i32, E, _>(output.as_slice()).unwrap() {
        ason::ResultStreamReader::Err(actual) => assert_eq!(&actual, error),
        ason::ResultStreamReader::Ok(_) => panic!("expected a business error"),
    }
}

#[test]
fn test_result_ok_round_trips() {
    assert_ok_round_trip::<i32>(&[], "Ok([])");
    assert_ok_round_trip(&[11], "Ok([\n    11\n])");
    assert_ok_round_trip(&[11, 13, 17], "Ok([\n    11\n    13\n    17\n])");
    assert_ok_round_trip(
        &[vec![11, 13], vec![], vec![17]],
        "Ok([\n    [\n        11\n        13\n    ]\n    []\n    [\n        17\n    ]\n])",
    );
    assert_ok_round_trip(
        &[
            Object {
                id: 11,
                name: "foo".into(),
            },
            Object {
                id: 13,
                name: "bar".into(),
            },
        ],
        "Ok([\n    {\n        id: 11\n        name: \"foo\"\n    }\n    {\n        id: 13\n        name: \"bar\"\n    }\n])",
    );
}

#[test]
fn test_result_err_round_trips() {
    assert_err_round_trip(
        &Object {
            id: 7,
            name: "failed".into(),
        },
        "Err({\n    id: 7\n    name: \"failed\"\n})",
    );
    assert_err_round_trip(&vec![Some(11), None], "Err([\n    Some(11)\n    None\n])");
    assert_err_round_trip(&Vec::<i32>::new(), "Err([])");

    let mut output = Vec::new();
    ason::result_stream_writer::<i32, _>(&mut output)
        .write_err("failed\n\"reason\"")
        .unwrap();
    assert_eq!(output, b"Err(\"failed\n\\\"reason\\\"\")");
    assert!(matches!(
        ason::result_stream_reader::<i32, String, _>(output.as_slice()).unwrap(),
        ason::ResultStreamReader::Err(error) if error == "failed\n\"reason\""
    ));
}

#[test]
fn test_result_reader_qualified_and_shorthand_forms() {
    for name in ["", "Result::"] {
        for (payload, expected) in [
            ("[]", vec![]),
            ("[11, -13 // next\n +17,]", vec![11, -13, 17]),
        ] {
            let document = format!("/* before */ {name}Ok( /* list */ {payload} /* end */ )");
            let ason::ResultStreamReader::Ok(reader) =
                ason::result_stream_reader::<i32, String, _>(document.as_bytes()).unwrap()
            else {
                panic!("expected a streamed List");
            };
            assert_eq!(reader.collect::<Result<Vec<_>, _>>().unwrap(), expected);
        }
        let document = format!("/* before */ {name}Err( /* error */ \"failed\" /* end */ )");
        assert!(matches!(
            ason::result_stream_reader::<i32, String, _>(document.as_bytes()).unwrap(),
            ason::ResultStreamReader::Err(error) if error == "failed"
        ));
    }
}

#[test]
fn test_result_reader_rejects_invalid_headers_and_errors() {
    for document in [
        "",
        "[1]",
        "42",
        "Other::Ok([1])",
        "Result::Other([1])",
        "Result::Ok",
        "Ok[1]",
        "Ok()",
        "Result::Ok(1)",
        "Result::Ok((1))",
        "ok([1])",
        "OK([1])",
        "err(\"failed\")",
        "ERR(\"failed\")",
        "Err()",
        "Result::Err(1)",
        "Result::Err(\"failed\"",
        "Result::Err(\"failed\" 1)",
        "Err(\"failed\"]",
        "Err(\"unfinished",
        "&",
    ] {
        assert!(
            ason::result_stream_reader::<i32, String, _>(document.as_bytes()).is_err(),
            "{document}"
        );
    }
}

#[test]
fn test_result_reader_reports_deferred_errors_once() {
    for (document, first) in [
        ("Ok([", None),
        ("Ok([]", None),
        ("Ok([]}", None),
        ("Ok([true])", None),
        ("Result::Ok([1]", Some(1)),
        ("Result::Ok([1]}", Some(1)),
        ("Result::Ok([1] &)", Some(1)),
        ("Result::Ok([1", Some(1)),
        ("Result::Ok([1 \"wrong\"])", Some(1)),
        ("Ok([1 2_u8])", Some(1)),
        ("Ok([1] 2)", Some(1)),
        ("Ok([1})", Some(1)),
    ] {
        let ason::ResultStreamReader::Ok(mut reader) =
            ason::result_stream_reader::<i32, String, _>(document.as_bytes()).unwrap()
        else {
            panic!("expected a streamed List");
        };
        if let Some(first) = first {
            assert_eq!(reader.next().unwrap().unwrap(), first, "{document}");
        }
        assert!(reader.next().unwrap().is_err(), "{document}");
        assert!(reader.next().is_none(), "{document}");
        assert!(reader.next().is_none(), "{document}");
    }
}

#[test]
fn test_result_reader_can_stop_before_invalid_elements_or_delimiters() {
    for document in ["Ok([1 false])", "Ok([1]", "Ok([1"] {
        let ason::ResultStreamReader::Ok(reader) =
            ason::result_stream_reader::<i32, String, _>(document.as_bytes()).unwrap()
        else {
            panic!("expected a streamed List");
        };
        assert_eq!(
            reader.take(1).collect::<Result<Vec<_>, _>>().unwrap(),
            vec![1]
        );
    }
}

#[test]
fn test_result_reader_does_not_validate_trailing_content() {
    for trailing in ["42", "&", "\"unfinished"] {
        for name in ["", "Result::"] {
            let document = format!("{name}Ok([1]) {trailing}");
            let ason::ResultStreamReader::Ok(reader) =
                ason::result_stream_reader::<i32, String, _>(document.as_bytes()).unwrap()
            else {
                panic!("expected a streamed List");
            };
            assert_eq!(reader.collect::<Result<Vec<_>, _>>().unwrap(), vec![1]);

            let document = format!("{name}Err(\"failed\") {trailing}");
            assert!(matches!(
                ason::result_stream_reader::<i32, String, _>(document.as_bytes()).unwrap(),
                ason::ResultStreamReader::Err(error) if error == "failed"
            ));
        }
    }
}

struct FailingReader;

impl Read for FailingReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            Ok(0)
        } else {
            Err(io::Error::other("test read failure"))
        }
    }
}

#[test]
fn test_result_reader_propagates_io_errors() {
    for prefix in ["", "Ok(", "Err(", "Err(\"failed\" "] {
        let input = prefix.as_bytes().chain(FailingReader);
        let error = match ason::result_stream_reader::<i32, String, _>(input) {
            Ok(_) => panic!("expected a read error for {prefix}"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("test read failure"), "{error}");
    }

    for prefix in ["Ok([1 ", "Ok([1] "] {
        let input = prefix.as_bytes().chain(FailingReader);
        let ason::ResultStreamReader::Ok(mut reader) =
            ason::result_stream_reader::<i32, String, _>(input).unwrap()
        else {
            panic!("expected a streamed List");
        };
        assert_eq!(reader.next().unwrap().unwrap(), 1);
        let error = reader.next().unwrap().unwrap_err();
        assert!(error.to_string().contains("test read failure"), "{error}");
        assert!(reader.next().is_none());
        assert!(reader.next().is_none());
    }

    let input = b"Err(\"failed\")".as_slice().chain(FailingReader);
    assert!(matches!(
        ason::result_stream_reader::<i32, String, _>(input).unwrap(),
        ason::ResultStreamReader::Err(error) if error == "failed"
    ));
    let input = b"Ok([1])".as_slice().chain(FailingReader);
    let ason::ResultStreamReader::Ok(reader) =
        ason::result_stream_reader::<i32, String, _>(input).unwrap()
    else {
        panic!("expected a streamed List");
    };
    assert_eq!(reader.collect::<Result<Vec<_>, _>>().unwrap(), vec![1]);
}

#[test]
fn test_result_reader_reports_utf8_errors() {
    for input in [&b"\xff"[..], &b"Err(\"\xff\")"[..], &b"Err(\"\xe2\x82"[..]] {
        let error = match ason::result_stream_reader::<String, String, _>(input) {
            Ok(_) => panic!("expected a UTF-8 error"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("UTF-8"), "{error}");
    }
    for input in [
        &b"Ok([\"first\" \"\xff\"])"[..],
        &b"Ok([\"first\" \"\xe2\x82"[..],
    ] {
        let ason::ResultStreamReader::Ok(mut reader) =
            ason::result_stream_reader::<String, String, _>(input).unwrap()
        else {
            panic!("expected a streamed List");
        };
        assert_eq!(reader.next().unwrap().unwrap(), "first");
        let error = reader.next().unwrap().unwrap_err();
        assert!(error.to_string().contains("UTF-8"), "{error}");
        assert!(reader.next().is_none());
    }
}

#[test]
fn test_result_reader_handles_short_interrupted_utf8_reads() {
    struct ShortReader<'a> {
        bytes: &'a [u8],
        interrupt: bool,
    }

    impl Read for ShortReader<'_> {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if buffer.is_empty() {
                return Ok(0);
            }
            self.interrupt = !self.interrupt;
            if self.interrupt {
                return Err(io::ErrorKind::Interrupted.into());
            }
            self.bytes.read(&mut buffer[..1])
        }
    }

    let text = "\u{4e2d}\u{6587}\u{1f980}";
    for (document, expect_ok) in [
        (format!("Ok([\"{text}\" \"\"])"), true),
        (format!("Err(\"{text}\")"), false),
    ] {
        let input = ShortReader {
            bytes: document.as_bytes(),
            interrupt: false,
        };
        match (
            expect_ok,
            ason::result_stream_reader::<String, String, _>(input).unwrap(),
        ) {
            (true, ason::ResultStreamReader::Ok(reader)) => assert_eq!(
                reader.collect::<Result<Vec<_>, _>>().unwrap(),
                vec![text, ""]
            ),
            (false, ason::ResultStreamReader::Err(error)) => assert_eq!(error, text),
            _ => panic!("unexpected Result branch"),
        }
    }
}

#[test]
fn test_result_writer_propagates_io_errors_at_every_byte() {
    for (values, document) in [
        (vec![], "Ok([])"),
        (vec![11, 13], "Ok([\n    11\n    13\n])"),
    ] {
        for capacity in 0..=document.len() {
            let mut output = vec![0; capacity];
            let mut writer = Cursor::new(output.as_mut_slice());
            let result = (|| -> Result<(), ason::AsonError> {
                let mut stream = ason::result_stream_writer(&mut writer).start_ok()?;
                if let Some((first, rest)) = values.split_first() {
                    stream.write(first)?;
                    stream.writes(rest)?;
                }
                stream.end_list()
            })();
            assert_eq!(
                result.is_ok(),
                capacity == document.len(),
                "capacity {capacity}"
            );
            assert_eq!(
                output,
                document.as_bytes()[..capacity],
                "capacity {capacity}"
            );
        }
    }

    let document = "Err(\"failed\")";
    for capacity in 0..=document.len() {
        let mut output = vec![0; capacity];
        let mut writer = Cursor::new(output.as_mut_slice());
        let result = ason::result_stream_writer::<i32, _>(&mut writer).write_err("failed");
        assert_eq!(
            result.is_ok(),
            capacity == document.len(),
            "capacity {capacity}"
        );
        assert_eq!(
            output,
            document.as_bytes()[..capacity],
            "capacity {capacity}"
        );
    }
}

#[test]
fn test_result_writer_stops_batch_on_serialization_error() {
    struct Counted<'a> {
        value: i32,
        attempts: &'a Cell<usize>,
    }

    impl Serialize for Counted<'_> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            self.attempts.set(self.attempts.get() + 1);
            if self.value == 13 {
                return Err(serde::ser::Error::custom("test serialization failure"));
            }
            serializer.serialize_i32(self.value)
        }
    }

    let attempts = Cell::new(0);
    let values = [11, 13, 17].map(|value| Counted {
        value,
        attempts: &attempts,
    });
    let mut output = Vec::new();
    let mut stream = ason::result_stream_writer(&mut output).start_ok().unwrap();
    let error = stream.writes(&values).unwrap_err();
    assert!(
        error.to_string().contains("test serialization failure"),
        "{error}"
    );
    assert_eq!(attempts.get(), 2);
    assert_eq!(output, b"Ok([\n    11\n    ");

    output.clear();
    let error = ason::result_stream_writer::<i32, _>(&mut output)
        .write_err(&())
        .unwrap_err();
    assert!(error.to_string().contains("unit"), "{error}");
    assert_eq!(output, b"Err(");
}

#[test]
fn test_result_write_value_without_serde() {
    struct NonSerde;

    let values = vec![ason::Value::I32(1), ason::Value::String("two".into())];
    let mut output = Vec::new();
    let mut stream = ason::result_stream_writer::<NonSerde, _>(&mut output)
        .start_ok()
        .unwrap();
    for value in &values {
        stream.write_value(value).unwrap();
    }
    stream.end_list().unwrap();
    assert_eq!(output, b"Ok([\n    1\n    \"two\"\n])");
    assert_eq!(
        ason::de_value_from_reader(output.as_slice()).unwrap(),
        ason::Value::Enumeration {
            name: "Result".into(),
            variant: "Ok".into(),
            payload: ason::VariantValue::Newtype(Box::new(ason::Value::List(values))),
        }
    );

    output.clear();
    let mut stream = ason::result_stream_writer::<NonSerde, _>(&mut output)
        .start_ok()
        .unwrap();
    assert!(stream.write_value(&ason::Value::Tuple(vec![])).is_err());
    assert_eq!(output, b"Ok([\n    ");
}

#[test]
fn test_result_writer_does_not_finish_or_flush_implicitly() {
    let mut output = Vec::new();
    {
        let _writer = ason::result_stream_writer::<i32, _>(&mut output);
    }
    assert!(output.is_empty());

    for (finish, expected) in [(false, "Ok([\n    11"), (true, "Ok([\n    11\n])")] {
        let mut output = BufWriter::with_capacity(1024, Vec::new());
        {
            let mut stream = ason::result_stream_writer(&mut output).start_ok().unwrap();
            stream.write(&11).unwrap();
            if finish {
                stream.end_list().unwrap();
            }
        }
        assert!(output.get_ref().is_empty());
        output.flush().unwrap();
        assert_eq!(output.get_ref(), expected.as_bytes());
    }

    let mut output = BufWriter::with_capacity(1024, Vec::new());
    ason::result_stream_writer::<i32, _>(&mut output)
        .write_err("failed")
        .unwrap();
    assert!(output.get_ref().is_empty());
    output.flush().unwrap();
    assert_eq!(output.get_ref(), b"Err(\"failed\")");
}
