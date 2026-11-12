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

use std::io::Write;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Object {
    id: i32,
    name: String,
}

fn main() {
    let mut output = std::io::stdout().lock();
    if std::env::args().any(|arg| arg == "--error") {
        ason::result_stream_writer::<Object, _>(&mut output)
            .write_err("Eve")
            .unwrap();

        output.flush().unwrap();
    } else {
        let o1 = Object {
            id: 11,
            name: "Alice".to_string(),
        };

        let o2 = Object {
            id: 13,
            name: "Bob".to_string(),
        };

        let o3 = Object {
            id: 17,
            name: "Charlie".to_string(),
        };

        let o4 = Object {
            id: 19,
            name: "David".to_string(),
        };

        let mut stream_writer = ason::result_stream_writer(&mut output).start_ok().unwrap();

        // Serialize the first element.
        stream_writer.write(&o1).unwrap();

        // Serialize the second element.
        stream_writer.write(&o2).unwrap();

        // Serialize the third and fourth element.
        stream_writer.writes(&[o3, o4]).unwrap();

        // End the list and the Result explicitly.
        stream_writer.end_list().unwrap();

        output.flush().unwrap();
    }
}
