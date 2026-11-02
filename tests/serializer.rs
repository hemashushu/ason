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
    let package: Package = ason::de_from_str(text).unwrap();

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
    let serialized_text = ason::ser_to_string(&package).unwrap();
    assert_eq!(serialized_text, text);
}

#[test]
fn test_byte_data_and_date_time_adapters() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Record {
        #[serde(with = "ason::date_time")]
        time: chrono::DateTime<chrono::FixedOffset>,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
        bytes: serde_bytes::ByteBuf,
        dates: Vec<ason::date_time::DateTime>,
        optional_time: Option<ason::date_time::DateTime>,
    }

    let time = chrono::DateTime::parse_from_rfc3339("2024-03-16T16:30:50+08:00").unwrap();
    for data in [vec![], vec![0x00, 0xff], (0..=255).collect()] {
        let record = Record {
            time,
            bytes: serde_bytes::ByteBuf::from(data.clone()),
            data,
            dates: vec![ason::date_time::DateTime(time)],
            optional_time: Some(ason::date_time::DateTime(time)),
        };
        let text = ason::ser_to_string(&record).unwrap();
        assert!(text.contains("time: d\"2024-03-16T16:30:50+08:00\""));
        assert!(text.contains("data: h\""));
        assert!(text.contains("bytes: h\""));
        let decoded: Record = ason::de_from_str(&text).unwrap();
        assert_eq!(decoded, record);
        assert_eq!(decoded.time.offset(), record.time.offset());

        let mut output = Vec::new();
        ason::ser_to_writer(&record, &mut output).unwrap();
        assert_eq!(
            ason::de_from_reader::<Record, _>(&output[..]).unwrap(),
            record
        );

        output.clear();
        let mut writer = ason::list_stream_writer::<Record, _>(&mut output);
        writer.start_list().unwrap();
        writer.write(&record).unwrap();
        writer.write(&record).unwrap();
        writer.end_list().unwrap();
        let stream_text = String::from_utf8(output).unwrap();
        let mut reader = ason::list_stream_reader::<Record, _>(stream_text.as_bytes()).unwrap();
        assert_eq!(reader.next().unwrap().unwrap(), record);
        assert_eq!(reader.next().unwrap().unwrap(), record);
        assert!(reader.next().is_none());
    }

    assert_eq!(
        ason::ser_to_string(&vec![0u8, 255]).unwrap(),
        "[\n    0_u8\n    255_u8\n]",
    );
    assert_eq!(
        ason::de_from_str::<Vec<u8>>("[0_u8, 255_u8]").unwrap(),
        vec![0, 255],
    );
}

#[test]
fn test_array_mapping_modes() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Record {
        default_array: [i32; 4],
        default_bytes: [u8; 3],
        #[serde(with = "serde_bytes")]
        byte_data: [u8; 3],
        #[serde(with = "ason::array")]
        byte_list: [u8; 3],
        #[serde(with = "ason::array")]
        integer_list: [i32; 4],
    }

    let record = Record {
        default_array: [11, 13, 17, 19],
        default_bytes: *b"abc",
        byte_data: *b"abc",
        byte_list: *b"abc",
        integer_list: [11, 13, 17, 19],
    };
    let text = ason::ser_to_string(&record).unwrap();
    assert!(text.contains("default_array: (11 13 17 19)"));
    assert!(text.contains("default_bytes: (97_u8 98_u8 99_u8)"));
    assert!(text.contains("byte_data: h\"61 62 63\""));
    assert!(text.contains("byte_list: [\n        97_u8\n        98_u8\n        99_u8\n    ]"));
    assert!(
        text.contains("integer_list: [\n        11\n        13\n        17\n        19\n    ]")
    );
    assert_eq!(ason::de_from_str::<Record>(&text).unwrap(), record);
    assert_eq!(
        ason::ser_to_string(&(11, 13, 17, 19)).unwrap(),
        "(11 13 17 19)"
    );

    let mut output = Vec::new();
    ason::ser_to_writer(&record, &mut output).unwrap();
    assert_eq!(
        ason::de_from_reader::<Record, _>(&output[..]).unwrap(),
        record
    );

    output.clear();
    let mut writer = ason::list_stream_writer::<Record, _>(&mut output);
    writer.start_list().unwrap();
    writer.write(&record).unwrap();
    writer.end_list().unwrap();
    let stream_text = String::from_utf8(output).unwrap();
    let mut reader = ason::list_stream_reader::<Record, _>(stream_text.as_bytes()).unwrap();
    assert_eq!(reader.next().unwrap().unwrap(), record);
    assert!(reader.next().is_none());

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct EmptyBytes {
        #[serde(with = "serde_bytes")]
        data: [u8; 0],
    }

    let empty = EmptyBytes { data: [] };
    assert_eq!(
        ason::ser_to_string(&empty).unwrap(),
        "{\n    data: h\"\"\n}"
    );
    assert_eq!(
        ason::de_from_str::<EmptyBytes>("{data: h\"\"}").unwrap(),
        empty
    );
    assert!(ason::de_from_str::<EmptyBytes>("{data: h\"00\"}").is_err());
    assert!(ason::de_from_str::<Record>(&text.replace("61 62 63", "61 62")).is_err());
}
