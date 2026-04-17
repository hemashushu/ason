// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use ason::utf8_char_iterator::UTF8CharIterator;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Package {
    name: String,
    version: String,
    dependencies: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Object {
    id: i32,
    name: String,
}

#[test]
fn test_serialize_and_deserialize() {
    let text = r#"{
    name: "foo"
    version: "0.1.0"
    dependencies: [
        "random"
        "regex"
    ]
}"#;

    // Deserialize the ASON document string into a `Package` struct.
    let package: Package = ason::de::de_from_str(text).unwrap();

    // Verify the deserialized `Package` struct.
    assert_eq!(
        package,
        Package {
            name: String::from("foo"),
            version: String::from("0.1.0"),
            dependencies: vec![String::from("random"), String::from("regex")],
        }
    );

    // Serialize the `Package` struct back into an ASON document string.
    let serialized_text = ason::ser::ser_to_string(&package).unwrap();
    assert_eq!(serialized_text, text);
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
    let mut ser = ason::ser::list_to_writer(&mut buf);

    ser.start_list().unwrap();
    ser.serialize_element(&o1).unwrap();
    ser.serialize_element(&o2).unwrap();
    ser.end_list().unwrap();

    // deserialize
    let mut cursor = std::io::Cursor::new(buf);
    let mut char_iter = UTF8CharIterator::new(&mut cursor);
    let mut de = ason::de::list_from_char_iterator(&mut char_iter).unwrap();

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
