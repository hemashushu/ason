// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use ason::utf8_char_iterator::UTF8CharIterator;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Object {
    id: i32,
    name: String,
}

fn main() {
    // Build and run this program with this command:
    //
    // ```sh
    // cargo build --examples
    // ./target/debug/examples/stream-ser | ./target/debug/examples/stream-de
    // ```

    let mut input = std::io::stdin().lock();
    let mut char_iter = UTF8CharIterator::new(&mut input);
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

    println!("Deserialization successful!");
}
