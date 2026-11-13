// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::{
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
fn assert_list_round_trip<T>(values: &[T], document: &str)
where
    T: Serialize + DeserializeOwned + Debug + PartialEq,
{
    let mut output = Vec::new();
    let mut stream = ason::list_stream_writer(&mut output);
    stream.start_list().unwrap();
    stream.writes(&[]).unwrap();
    if let Some((first, rest)) = values.split_first() {
        stream.write(first).unwrap();
        stream.writes(rest).unwrap();
    }
    stream.writes(&[]).unwrap();
    stream.end_list().unwrap();

    assert_eq!(output, document.as_bytes());
    assert_eq!(ason::ser_to_string(&values).unwrap(), document);
    assert_eq!(
        ason::de_from_reader::<Vec<T>, _>(output.as_slice()).unwrap(),
        values
    );
    let mut input = Cursor::new(output);
    let reader: ason::ListStreamReader<T, _> = ason::list_stream_reader(&mut input).unwrap();
    assert_eq!(reader.collect::<Result<Vec<_>, _>>().unwrap(), values);
}

#[test]
fn test_list_stream_round_trips() {
    assert_list_round_trip::<i32>(&[], "[]");
    assert_list_round_trip(&[11], "[\n    11\n]");
    assert_list_round_trip(&[11, 13, 17], "[\n    11\n    13\n    17\n]");
    assert_list_round_trip(
        &[vec![], vec![11, 13], vec![], vec![17]],
        "[\n    []\n    [\n        11\n        13\n    ]\n    []\n    [\n        17\n    ]\n]",
    );
    assert_list_round_trip(
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
        "[\n    {\n        id: 11\n        name: \"foo\"\n    }\n    {\n        id: 13\n        name: \"bar\"\n    }\n]",
    );
}

#[test]
fn test_list_reader_accepts_comments_and_separators() {
    for (document, expected) in [
        ("/* before */ [, /* empty */ ,]", vec![]),
        ("[11, -13 // next\n +17,] /* after */", vec![11, -13, 17]),
    ] {
        let reader = ason::list_stream_reader::<i32, _>(document.as_bytes()).unwrap();
        assert_eq!(reader.collect::<Result<Vec<_>, _>>().unwrap(), expected);
    }

    let reader = ason::list_stream_reader::<Option<(i32, String)>, _>(
        br#"[None Some((1 "one")) None]"#.as_slice(),
    )
    .unwrap();
    assert_eq!(
        reader.collect::<Result<Vec<_>, _>>().unwrap(),
        vec![None, Some((1, "one".into())), None]
    );
}

#[test]
fn test_list_reader_rejects_invalid_openings() {
    for document in ["", "42", "{}", "(1)", "Ok([1])", "\"list\"", "&"] {
        assert!(
            ason::list_stream_reader::<i32, _>(document.as_bytes()).is_err(),
            "{document}"
        );
    }
}

#[test]
fn test_list_reader_reports_deferred_errors_once() {
    for (document, first) in [
        ("[", None),
        ("[true]", None),
        ("[1", Some(1)),
        ("[1}", Some(1)),
        ("[1 true]", Some(1)),
        ("[1 2_u8]", Some(1)),
        ("[1 &]", Some(1)),
        ("[1 (2)]", Some(1)),
        ("[1 [2]]", Some(1)),
        ("[1 \"unfinished", Some(1)),
    ] {
        let mut reader = ason::list_stream_reader::<i32, _>(document.as_bytes()).unwrap();
        if let Some(first) = first {
            assert_eq!(reader.next().unwrap().unwrap(), first, "{document}");
        }
        assert!(reader.next().unwrap().is_err(), "{document}");
        assert!(reader.next().is_none(), "{document}");
        assert!(reader.next().is_none(), "{document}");
    }
}

#[test]
fn test_list_reader_can_stop_before_invalid_elements() {
    for document in ["[1 true]", "[1"] {
        let reader = ason::list_stream_reader::<i32, _>(document.as_bytes()).unwrap();
        assert_eq!(
            reader.take(1).collect::<Result<Vec<_>, _>>().unwrap(),
            vec![1]
        );
    }
}

#[test]
fn test_list_reader_does_not_validate_trailing_content() {
    for document in ["[] 42", "[1] true", "[1] &", "[1] \"unfinished"] {
        let mut reader = ason::list_stream_reader::<i32, _>(document.as_bytes()).unwrap();
        let expected = if document.starts_with("[]") {
            vec![]
        } else {
            vec![1]
        };
        assert_eq!(
            reader.by_ref().collect::<Result<Vec<_>, _>>().unwrap(),
            expected
        );
        assert!(reader.next().is_none());
        assert!(reader.next().is_none());
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
fn test_list_reader_io_and_utf8_errors() {
    let error = match ason::list_stream_reader::<i32, _>(FailingReader) {
        Ok(_) => panic!("expected an opening read error"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("test read failure"), "{error}");
    assert!(ason::list_stream_reader::<i32, _>(b"\xff".as_slice()).is_err());

    let input = b"[1 ".as_slice().chain(FailingReader);
    let mut reader = ason::list_stream_reader::<i32, _>(input).unwrap();
    assert_eq!(reader.next().unwrap().unwrap(), 1);
    let error = reader.next().unwrap().unwrap_err();
    assert!(error.to_string().contains("test read failure"), "{error}");
    assert!(reader.next().is_none());
    assert!(reader.next().is_none());

    for input in [&b"[1 \"\xff\"]"[..], &b"[1 \"\xe2\x82"[..]] {
        let mut reader = ason::list_stream_reader::<i32, _>(input).unwrap();
        assert_eq!(reader.next().unwrap().unwrap(), 1);
        let error = reader.next().unwrap().unwrap_err();
        assert!(error.to_string().contains("UTF-8"), "{error}");
        assert!(reader.next().is_none());
    }

    let input = b"[1]".as_slice().chain(FailingReader);
    let reader = ason::list_stream_reader::<i32, _>(input).unwrap();
    assert_eq!(reader.collect::<Result<Vec<_>, _>>().unwrap(), vec![1]);
}

#[test]
fn test_list_writer_propagates_io_errors_at_every_byte() {
    for (values, document) in [(vec![], "[]"), (vec![11, 13], "[\n    11\n    13\n]")] {
        for capacity in 0..=document.len() {
            let mut output = vec![0; capacity];
            let mut writer = Cursor::new(output.as_mut_slice());
            let result = (|| -> Result<(), ason::AsonError> {
                let mut stream = ason::list_stream_writer(&mut writer);
                stream.start_list()?;
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
}

#[test]
fn test_list_writer_propagates_serialization_errors() {
    let mut output = Vec::new();
    let mut stream = ason::list_stream_writer(&mut output);
    stream.start_list().unwrap();
    let error = stream.write(&()).unwrap_err();
    assert!(error.to_string().contains("unit"), "{error}");
    assert_eq!(output, b"[\n    ");
}

#[test]
fn test_list_writer_does_not_finish_or_flush_implicitly() {
    let mut output = Vec::new();
    {
        let _stream = ason::list_stream_writer::<i32, _>(&mut output);
    }
    assert!(output.is_empty());

    for (finish, expected) in [(false, "[\n    11"), (true, "[\n    11\n]")] {
        let mut output = BufWriter::with_capacity(1024, Vec::new());
        {
            let mut stream = ason::list_stream_writer(&mut output);
            stream.start_list().unwrap();
            stream.write(&11).unwrap();
            if finish {
                stream.end_list().unwrap();
            }
        }
        assert!(output.get_ref().is_empty());
        output.flush().unwrap();
        assert_eq!(output.get_ref(), expected.as_bytes());
    }
}
