// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

//! Build and run this program with this command:
//!
//! ```sh
//! cargo build --examples
//! cd target/debug/examples
//! ./stream-writer | ./stream-reader
//! ```

use std::io::Write;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Object {
    id: i32,
    name: String,
}

fn main() {
    let mut input = std::io::stdin().lock();
    let mut char_iter = ason::UTF8CharIterator::new(&mut input);
    let mut stream_reader = ason::list_stream_reader(&mut char_iter).unwrap();

    // Deserialize the first element.
    let o1: Object = stream_reader.next().unwrap().unwrap();
    assert_eq!(
        o1,
        Object {
            id: 11,
            name: "foo".to_owned()
        }
    );

    // Deserialize the second element.
    let o2: Object = stream_reader.next().unwrap().unwrap();
    assert_eq!(
        o2,
        Object {
            id: 13,
            name: "bar".to_owned()
        }
    );

    // Read the remaining elements until the end of the list.
    while !stream_reader.next().is_none() {
        // ...
    }

    let mut stderr = std::io::stderr().lock();
    writeln!(&mut stderr, "Streaming deserialization successful!").unwrap();
}
