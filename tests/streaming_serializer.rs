// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Object {
    id: i32,
    name: String,
}

#[test]
fn test_stream_serialize_and_deserialize() {
    let o1 = Object {
        id: 11,
        name: "foo".to_owned(),
    };

    let o2 = Object {
        id: 13,
        name: "bar".to_owned(),
    };

    let mut buf = vec![];
    let mut ser = ason::list_stream_writer(&mut buf);

    ser.start_list().unwrap();
    ser.write(&o1).unwrap();
    ser.write(&o2).unwrap();
    ser.end_list().unwrap();

    // deserialize
    let mut cursor = std::io::Cursor::new(buf);
    let mut de = ason::list_stream_reader(&mut cursor).unwrap();

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
}

#[test]
fn test_batch_stream_serialize_and_deserialize() {
    let values = vec![
        Object {
            id: 11,
            name: "foo".to_owned(),
        },
        Object {
            id: 13,
            name: "bar".to_owned(),
        },
        Object {
            id: 17,
            name: "baz".to_owned(),
        },
    ];
    let mut buf = Vec::new();
    let mut stream = ason::list_stream_writer(&mut buf);
    stream.start_list().unwrap();
    stream.writes(&[]).unwrap();
    stream.write(&values[0]).unwrap();
    stream.writes(&values[1..]).unwrap();
    stream.writes(&[]).unwrap();
    stream.end_list().unwrap();

    assert_eq!(buf, ason::ser_to_string(&values).unwrap().as_bytes());
    let decoded = ason::list_stream_reader::<Object, _>(buf.as_slice())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(decoded, values);
}

#[test]
fn test_concrete_reader_types() {
    assert_eq!(ason::de_from_str::<i32>("42").unwrap(), 42);

    let stream: ason::ListStreamReader<i32, &[u8]> =
        ason::list_stream_reader("[11 13]".as_bytes()).unwrap();
    assert_eq!(stream.collect::<Result<Vec<_>, _>>().unwrap(), vec![11, 13]);

    let stream = ason::list_stream_reader::<i32, _>(std::io::Cursor::new("[11 13]")).unwrap();
    assert_eq!(stream.collect::<Result<Vec<_>, _>>().unwrap(), vec![11, 13]);

    assert_eq!(ason::de_from_reader::<i32, _>("42".as_bytes()).unwrap(), 42);
}

#[test]
fn test_result_stream_reader() {
    for values in [vec![], vec![11, 13]] {
        let document = ason::ser_to_string(&Ok::<_, String>(&values)).unwrap();
        let stream: ason::ResultStreamReader<i32, String, &[u8]> =
            ason::result_stream_reader(document.as_bytes()).unwrap();
        let ason::ResultStreamReader::Ok(stream) = stream else {
            panic!("expected a streamed List");
        };
        assert_eq!(stream.collect::<Result<Vec<_>, _>>().unwrap(), values);
    }

    let error = Object {
        id: 7,
        name: "failed".to_owned(),
    };
    let document = ason::ser_to_string(&Err::<Vec<i32>, _>(&error)).unwrap();
    let stream = ason::result_stream_reader::<i32, Object, _>(document.as_bytes()).unwrap();
    match stream {
        ason::ResultStreamReader::Err(actual) => assert_eq!(actual, error),
        ason::ResultStreamReader::Ok(_) => panic!("expected a business error"),
    }
}

#[test]
fn test_result_stream_reader_enum_shorthand() {
    for (document, expected) in [("Ok([])", vec![]), ("Ok([11, 13])", vec![11, 13])] {
        let stream = ason::result_stream_reader::<i32, String, _>(document.as_bytes()).unwrap();
        let ason::ResultStreamReader::Ok(stream) = stream else {
            panic!("expected a streamed List");
        };
        assert_eq!(stream.collect::<Result<Vec<_>, _>>().unwrap(), expected);
    }

    let stream =
        ason::result_stream_reader::<i32, String, _>(b"Err(\"failed\")".as_slice()).unwrap();
    match stream {
        ason::ResultStreamReader::Err(error) => assert_eq!(error, "failed"),
        ason::ResultStreamReader::Ok(_) => panic!("expected a business error"),
    }

    for invalid in ["ok([1])", "OK([1])", "err(1)", "ERR(1)", "Ok(1)", "Err()"] {
        assert!(
            ason::result_stream_reader::<i32, i32, _>(invalid.as_bytes()).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn test_result_stream_writer() {
    for values in [vec![], vec![vec![11, 13], vec![], vec![17]]] {
        let mut buffer = Vec::new();
        let mut stream = ason::result_stream_writer(&mut buffer).start_ok().unwrap();
        stream.writes(&[]).unwrap();
        for value in &values {
            stream.write(value).unwrap();
        }
        stream.end_list().unwrap();
        assert!(buffer.starts_with(b"Ok(["));
        assert_eq!(
            buffer,
            ason::ser_to_string(&Ok::<_, String>(&values))
                .unwrap()
                .as_bytes()
        );
        assert_eq!(
            ason::de_from_reader::<Result<Vec<Vec<i32>>, String>, _>(buffer.as_slice()).unwrap(),
            Ok(values.clone())
        );
        let ason::ResultStreamReader::Ok(stream) =
            ason::result_stream_reader::<Vec<i32>, String, _>(buffer.as_slice()).unwrap()
        else {
            panic!("expected a streamed List");
        };
        assert_eq!(stream.collect::<Result<Vec<_>, _>>().unwrap(), values);
    }

    let values = [
        Object {
            id: 11,
            name: "foo".to_owned(),
        },
        Object {
            id: 13,
            name: "bar".to_owned(),
        },
    ];
    let mut buffer = Vec::new();
    let mut stream = ason::result_stream_writer(&mut buffer).start_ok().unwrap();
    stream.write(&values[0]).unwrap();
    stream.writes(&values[1..]).unwrap();
    stream.end_list().unwrap();
    assert_eq!(
        buffer,
        ason::ser_to_string(&Ok::<_, String>(&values[..]))
            .unwrap()
            .as_bytes()
    );

    let mut buffer = Vec::new();
    ason::result_stream_writer::<i32, _>(&mut buffer)
        .write_err(&values[0])
        .unwrap();
    assert_eq!(
        buffer,
        ason::ser_to_string(&Err::<Vec<i32>, _>(&values[0]))
            .unwrap()
            .as_bytes()
    );
    assert_eq!(
        ason::de_from_reader::<Result<Vec<i32>, Object>, _>(buffer.as_slice()).unwrap(),
        Err(Object {
            id: 11,
            name: "foo".to_owned()
        })
    );

    let mut buffer = Vec::new();
    ason::result_stream_writer::<i32, _>(&mut buffer)
        .write_err("failed")
        .unwrap();
    assert_eq!(buffer, b"Err(\"failed\")");
}

#[test]
fn test_result_stream_reader_rejects_invalid_documents() {
    for document in [
        "[1]",
        "Other::Ok([1])",
        "Result::Other([1])",
        "Result::Ok",
        "Result::Ok(1)",
        "Result::Ok((1))",
        "Result::Err(1)",
        "Result::Err(\"failed\"",
        "Result::Err(\"failed\" 1)",
    ] {
        assert!(
            ason::result_stream_reader::<i32, String, _>(document.as_bytes()).is_err(),
            "{document}"
        );
    }

    for document in [
        "Result::Ok([1]",
        "Result::Ok([1]}",
        "Result::Ok([1] &)",
        "Result::Ok([1",
        "Result::Ok([1 \"wrong\"])",
    ] {
        let ason::ResultStreamReader::Ok(mut stream) =
            ason::result_stream_reader::<i32, String, _>(document.as_bytes()).unwrap()
        else {
            panic!("expected a streamed List");
        };
        assert_eq!(stream.next().unwrap().unwrap(), 1);
        assert!(stream.next().unwrap().is_err(), "{document}");
        assert!(stream.next().is_none());
    }
}

#[test]
fn test_result_stream_reader_stops_without_checking_trailing_content() {
    let ason::ResultStreamReader::Ok(stream) =
        ason::result_stream_reader::<i32, String, _>(b"Result::Ok([1]) 42".as_slice()).unwrap()
    else {
        panic!("expected a streamed List");
    };
    assert_eq!(stream.collect::<Result<Vec<_>, _>>().unwrap(), vec![1]);
    assert!(matches!(
        ason::result_stream_reader::<i32, String, _>(b"Result::Err(\"failed\") 42".as_slice()).unwrap(),
        ason::ResultStreamReader::Err(error) if error == "failed"
    ));
}

#[test]
fn test_result_stream_writer_io_errors() {
    let expected = ason::ser_to_string(&Ok::<_, String>(vec![11, 13])).unwrap();
    for capacity in 0..expected.len() {
        let mut buffer = vec![0; capacity];
        let mut writer = std::io::Cursor::new(buffer.as_mut_slice());
        let result = (|| -> Result<(), ason::AsonError> {
            let mut stream = ason::result_stream_writer(&mut writer).start_ok()?;
            stream.writes(&[11, 13])?;
            stream.end_list()
        })();
        assert!(result.is_err(), "capacity {capacity}");
        assert_eq!(buffer, expected.as_bytes()[..capacity]);
    }

    let expected = "Err(\"failed\")";
    for capacity in 0..expected.len() {
        let mut buffer = vec![0; capacity];
        let mut writer = std::io::Cursor::new(buffer.as_mut_slice());
        assert!(
            ason::result_stream_writer::<i32, _>(&mut writer)
                .write_err("failed")
                .is_err()
        );
        assert_eq!(buffer, expected.as_bytes()[..capacity]);
    }
}
