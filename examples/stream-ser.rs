// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::io::Write;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Object {
    id: i32,
    name: String,
}

fn main() {
    let o1 = Object {
        id: 11,
        name: "foo".to_owned(),
    };

    let o2 = Object {
        id: 13,
        name: "bar".to_owned(),
    };

    let mut output = std::io::stdout().lock();
    let mut ser = ason::ser::list_to_writer(&mut output);

    ser.start_list().unwrap();
    ser.serialize_element(&o1).unwrap();
    ser.serialize_element(&o2).unwrap();
    ser.end_list().unwrap();

    output.flush().unwrap();
}
