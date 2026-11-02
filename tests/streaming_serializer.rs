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
    assert_eq!(
        ason::de_from_str::<i32>("42").unwrap(),
        42
    );

    let stream: ason::ListStreamReader<i32, &[u8]> =
        ason::list_stream_reader("[11 13]".as_bytes()).unwrap();
    assert_eq!(stream.collect::<Result<Vec<_>, _>>().unwrap(), vec![11, 13]);

    let stream = ason::list_stream_reader::<i32, _>(std::io::Cursor::new("[11 13]")).unwrap();
    assert_eq!(stream.collect::<Result<Vec<_>, _>>().unwrap(), vec![11, 13]);

    assert_eq!(ason::de_from_reader::<i32, _>("42".as_bytes()).unwrap(), 42);
}
