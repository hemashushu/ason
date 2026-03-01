// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Package {
    name: String,
    version: String,
    dependencies: Vec<String>,
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
