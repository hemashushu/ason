// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use ason::UTF8CharIterator;
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
    let mut char_iter = UTF8CharIterator::new(&mut cursor);
    let mut de = ason::list_stream_reader(&mut char_iter).unwrap();

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
    let chars = UTF8CharIterator::new(buf.as_slice());
    let decoded = ason::list_stream_reader::<Object, _>(chars)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(decoded, values);
}

#[test]
fn test_concrete_character_iterators() {
    assert_eq!(
        ason::de_from_char_iterator::<i32, _>("42".chars()).unwrap(),
        42
    );

    let mut chars = "42".chars();
    assert_eq!(
        ason::de_from_char_iterator::<i32, _>(&mut chars).unwrap(),
        42
    );

    let stream: ason::ListStreamReader<i32, std::str::Chars<'_>> =
        ason::list_stream_reader("[11 13]".chars()).unwrap();
    assert_eq!(stream.collect::<Result<Vec<_>, _>>().unwrap(), vec![11, 13]);

    let mut chars = "[11 13]".chars();
    let stream = ason::list_stream_reader::<i32, _>(&mut chars).unwrap();
    assert_eq!(stream.collect::<Result<Vec<_>, _>>().unwrap(), vec![11, 13]);

    let chars = UTF8CharIterator::new("[11 13]".as_bytes());
    let stream = ason::list_stream_reader::<i32, _>(chars).unwrap();
    assert_eq!(stream.collect::<Result<Vec<_>, _>>().unwrap(), vec![11, 13]);

    assert_eq!(ason::de_from_reader::<i32, _>("42".as_bytes()).unwrap(), 42);
}
