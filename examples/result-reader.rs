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
//! ./result-writer | ./result-reader
//! ./result-writer --error | ./result-reader
//! ```

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Object {
    id: i32,
    name: String,
}

fn main() {
    let mut input = std::io::stdin().lock();
    match ason::result_stream_reader::<Object, String, _>(&mut input).unwrap() {
        ason::ResultStreamReader::Ok(mut stream_reader) => {
            // Deserialize the first element.
            let o1: Object = stream_reader.next().unwrap().unwrap();
            assert_eq!(
                o1,
                Object {
                    id: 11,
                    name: "Alice".to_string()
                }
            );

            // Deserialize the second element.
            let o2: Object = stream_reader.next().unwrap().unwrap();
            assert_eq!(
                o2,
                Object {
                    id: 13,
                    name: "Bob".to_string()
                }
            );

            // Deserialize the remaining elements until the end of the list.
            let mut objs = vec![];
            for result in stream_reader {
                objs.push(result.unwrap());
            }

            assert_eq!(
                objs,
                vec![
                    Object {
                        id: 17,
                        name: "Charlie".to_string()
                    },
                    Object {
                        id: 19,
                        name: "David".to_string()
                    }
                ]
            );

            println!("Success: received \"Result::Ok\"");
        }
        ason::ResultStreamReader::Err(error) => {
            assert_eq!(error, "Eve");
            println!("Success: received \"Result::Err\"");
        }
    };
}
