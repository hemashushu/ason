// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::{
    collections::BTreeMap,
    fmt::Debug,
    io::{self, Cursor, Read},
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[track_caller]
fn assert_serde_round_trip<T>(value: &T, document: &str)
where
    T: Serialize + DeserializeOwned + Debug + PartialEq,
{
    assert_eq!(ason::ser_to_string(value).unwrap(), document);

    let mut output = Vec::new();
    ason::ser_to_writer(value, &mut output).unwrap();
    assert_eq!(output, document.as_bytes());

    assert_eq!(&ason::de_from_str::<T>(document).unwrap(), value);
    assert_eq!(
        &ason::de_from_reader::<T, _>(document.as_bytes()).unwrap(),
        value
    );
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Package {
    name: String,
    version: String,
    dependencies: Vec<String>,
}

#[test]
fn test_struct_round_trip() {
    let document = r#"{
    name: "foo"
    version: "0.1.0"
    dependencies: [
        "random"
        "regex"
    ]
}"#;

    let package = Package {
        name: String::from("foo"),
        version: String::from("0.1.0"),
        dependencies: vec![String::from("random"), String::from("regex")],
    };
    assert_serde_round_trip(&package, document);
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
    for (data, byte_prefix) in [
        (vec![], "h\"\""),
        (vec![0x00, 0xff], "h\"00 ff\""),
        ((0..=255).collect(), "h\"00 01 02 03  04 05 06 07\n"),
    ] {
        let record = Record {
            time,
            bytes: serde_bytes::ByteBuf::from(data.clone()),
            data,
            dates: vec![ason::date_time::DateTime(time)],
            optional_time: Some(ason::date_time::DateTime(time)),
        };
        let text = ason::ser_to_string(&record).unwrap();
        assert!(text.contains("time: d\"2024-03-16T16:30:50+08:00\""));
        assert!(text.contains(&format!("data: {byte_prefix}")), "{text}");
        assert!(text.contains(&format!("bytes: {byte_prefix}")), "{text}");
        let decoded: Record = ason::de_from_str(&text).unwrap();
        assert_eq!(decoded, record);
        assert_eq!(decoded.time.offset(), record.time.offset());
        assert_eq!(decoded.dates[0].0.offset(), time.offset());
        assert_eq!(
            decoded.optional_time.as_ref().unwrap().0.offset(),
            time.offset()
        );

        let mut output = Vec::new();
        ason::ser_to_writer(&record, &mut output).unwrap();
        assert_eq!(output, text.as_bytes());
        let decoded = ason::de_from_reader::<Record, _>(&output[..]).unwrap();
        assert_eq!(decoded, record);
        assert_eq!(decoded.time.offset(), time.offset());

        output.clear();
        let mut writer = ason::list_stream_writer::<Record, _>(&mut output);
        writer.start_list().unwrap();
        writer.write(&record).unwrap();
        writer.write(&record).unwrap();
        writer.end_list().unwrap();
        let mut reader = ason::list_stream_reader::<Record, _>(output.as_slice()).unwrap();
        assert_eq!(reader.next().unwrap().unwrap(), record);
        assert_eq!(reader.next().unwrap().unwrap(), record);
        assert!(reader.next().is_none());
    }

    let fractional = chrono::DateTime::parse_from_rfc3339("2024-03-16T16:30:50.5+08:00").unwrap();
    let invalid = Record {
        time: fractional,
        data: vec![],
        bytes: serde_bytes::ByteBuf::from(vec![]),
        dates: vec![],
        optional_time: None,
    };
    assert!(ason::ser_to_string(&invalid).is_err());
    assert!(ason::ser_to_writer(&invalid, &mut Vec::new()).is_err());
    let input = r#"{time: d"2024-03-16T16:30:50.5+08:00" data: h"" bytes: h"" dates: [] optional_time: None}"#;
    let valid_input = input.replace("50.5", "50");
    assert_eq!(
        ason::de_from_str::<Record>(&valid_input).unwrap().time,
        time
    );
    assert!(ason::de_from_str::<Record>(input).is_err());
    assert!(ason::de_from_reader::<Record, _>(input.as_bytes()).is_err());

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
    let document = r#"{
    default_array: (11 13 17 19)
    default_bytes: (97_u8 98_u8 99_u8)
    byte_data: h"61 62 63"
    byte_list: [
        97_u8
        98_u8
        99_u8
    ]
    integer_list: [
        11
        13
        17
        19
    ]
}"#;
    assert_serde_round_trip(&record, document);

    let mut output = Vec::new();
    let mut writer = ason::list_stream_writer::<Record, _>(&mut output);
    writer.start_list().unwrap();
    writer.write(&record).unwrap();
    writer.end_list().unwrap();
    let mut reader = ason::list_stream_reader::<Record, _>(output.as_slice()).unwrap();
    assert_eq!(reader.next().unwrap().unwrap(), record);
    assert!(reader.next().is_none());

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct EmptyBytes {
        #[serde(with = "serde_bytes")]
        data: [u8; 0],
    }

    let empty = EmptyBytes { data: [] };
    assert_serde_round_trip(&empty, "{\n    data: h\"\"\n}");
    assert!(ason::de_from_str::<EmptyBytes>("{data: h\"00\"}").is_err());
    assert!(ason::de_from_str::<Record>(&document.replace("61 62 63", "61 62")).is_err());
}

#[test]
fn test_serde_field_attributes() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Settings {
        #[serde(rename = "displayName")]
        name: String,
        #[serde(default)]
        retries: i32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    }

    let input = r#"{displayName: "Ada" ignored: [1 (2 "x") {nested: Some(Color::Rgb(3 4 5))}]}"#;
    let settings = Settings {
        name: "Ada".into(),
        retries: 0,
        note: None,
    };
    assert_eq!(ason::de_from_str::<Settings>(input).unwrap(), settings);
    assert_eq!(
        ason::de_from_reader::<Settings, _>(input.as_bytes()).unwrap(),
        settings
    );
    assert_serde_round_trip(&settings, "{\n    displayName: \"Ada\"\n    retries: 0\n}");

    let configured = Settings {
        retries: 2,
        note: Some("ready".into()),
        ..settings
    };
    assert_serde_round_trip(
        &configured,
        "{\n    displayName: \"Ada\"\n    retries: 2\n    note: Some(\"ready\")\n}",
    );
    assert!(ason::de_from_str::<Settings>(r#"{displayName: "Ada" ignored: [1)}"#).is_err());

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Strict {
        name: String,
    }

    assert_eq!(
        ason::de_from_str::<Strict>(r#"{name: "Ada"}"#)
            .unwrap()
            .name,
        "Ada"
    );
    let unknown = r#"{name: "Ada" extra: 1}"#;
    assert!(ason::de_from_str::<Strict>(unknown).is_err());
    assert!(ason::de_from_reader::<Strict, _>(unknown.as_bytes()).is_err());
}

#[test]
fn test_serde_enum_variants_and_renames() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(rename = "Action")]
    enum Event {
        #[serde(rename = "Start")]
        Ready,
        Note(String),
        Move(i32, i32),
        Resize {
            width: u8,
            height: u8,
        },
    }

    for (value, document) in [
        (Event::Ready, "Action::Start"),
        (Event::Note("hello".into()), r#"Action::Note("hello")"#),
        (Event::Move(-1, 2), "Action::Move(-1 2)"),
        (
            Event::Resize {
                width: 3,
                height: 4,
            },
            "Action::Resize{\n    width: 3_u8\n    height: 4_u8\n}",
        ),
    ] {
        assert_serde_round_trip(&value, document);
    }

    for input in [
        "Event::Start",
        "Action::Unknown",
        "Action::Note(1)",
        "Action::Move(1)",
        "Action::Resize{width: 3_u8}",
    ] {
        assert!(ason::de_from_str::<Event>(input).is_err(), "{input}");
        assert!(
            ason::de_from_reader::<Event, _>(input.as_bytes()).is_err(),
            "{input}"
        );
    }
}

#[test]
fn test_serde_map_and_optional_values() {
    let values: BTreeMap<String, Option<i32>> =
        BTreeMap::from([("alpha".into(), Some(1)), ("beta".into(), None)]);
    assert_serde_round_trip(&values, "[\n    \"alpha\": Some(1)\n    \"beta\": None\n]");
    assert_serde_round_trip(&BTreeMap::<String, Option<i32>>::new(), "[]");
}

#[test]
fn test_typed_deserialization_rejects_invalid_documents() {
    for input in [
        r#"{name: "foo" version: "1"}"#,
        r#"{name: "foo" version: 1 dependencies: []}"#,
        r#"{name: "foo" name: "bar" version: "1" dependencies: []}"#,
        r#"{name: "foo" version: "1" dependencies: []} true"#,
        r#"{name: "foo" version: "1" dependencies: ["pkg"]"#,
    ] {
        assert!(ason::de_from_str::<Package>(input).is_err(), "{input}");
        assert!(
            ason::de_from_reader::<Package, _>(input.as_bytes()).is_err(),
            "{input}"
        );
    }

    assert_eq!(ason::de_from_str::<u8>("18_u8").unwrap(), 18);
    assert!(ason::de_from_str::<u8>("18").is_err());
    assert!(ason::de_from_reader::<u8, _>("18".as_bytes()).is_err());
}

#[test]
fn test_array_adapter_rejects_wrong_lengths_and_types() {
    #[derive(Debug, Deserialize)]
    struct ArrayRecord {
        #[serde(with = "ason::array")]
        values: [i32; 3],
    }

    assert_eq!(
        ason::de_from_str::<ArrayRecord>("{values: [1 2 3]}")
            .unwrap()
            .values,
        [1, 2, 3]
    );
    for input in ["{values: [1 2]}", "{values: [1 2 3 4]}"] {
        let error = ason::de_from_str::<ArrayRecord>(input).unwrap_err();
        assert!(error.to_string().contains("exactly 3 elements"), "{error}");
        let error = ason::de_from_reader::<ArrayRecord, _>(input.as_bytes()).unwrap_err();
        assert!(error.to_string().contains("exactly 3 elements"), "{error}");
    }
    for input in ["{values: (1 2 3)}", "{values: [1 2 true]}"] {
        assert!(ason::de_from_str::<ArrayRecord>(input).is_err(), "{input}");
        assert!(
            ason::de_from_reader::<ArrayRecord, _>(input.as_bytes()).is_err(),
            "{input}"
        );
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct EmptyList {
        #[serde(with = "ason::array")]
        values: [i32; 0],
    }

    assert_serde_round_trip(&EmptyList { values: [] }, "{\n    values: []\n}");
    assert!(ason::de_from_str::<EmptyList>("{values: [1]}").is_err());
}

#[test]
fn test_reader_and_writer_errors() {
    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if buffer.is_empty() {
                Ok(0)
            } else {
                Err(io::Error::other("test read failure"))
            }
        }
    }

    let reader = b"42".as_slice().chain(FailingReader);
    let error = ason::de_from_reader::<i32, _>(reader).unwrap_err();
    assert!(error.to_string().contains("test read failure"), "{error}");

    for input in [&b"\"\xff\""[..], &b"\"\xe2\x82"[..]] {
        let error = ason::de_from_reader::<String, _>(input).unwrap_err();
        assert!(error.to_string().contains("UTF-8"), "{error}");
    }

    let mut output = [0; 2];
    let mut writer = Cursor::new(output.as_mut_slice());
    assert!(ason::ser_to_writer(&"hello", &mut writer).is_err());
    assert_eq!(&output, b"\"h");
}

#[test]
fn test_unsupported_serde_representations() {
    assert!(ason::ser_to_string(&()).is_err());
    assert!(ason::ser_to_writer(&(), &mut Vec::new()).is_err());
    assert!(ason::de_from_str::<()>("()").is_err());
    assert!(ason::de_from_reader::<(), _>("()".as_bytes()).is_err());

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum Scalar {
        Integer(i32),
        Boolean(bool),
    }

    let error = match ason::de_from_str::<Scalar>("42") {
        Ok(Scalar::Integer(value)) => panic!("unexpected integer {value}"),
        Ok(Scalar::Boolean(value)) => panic!("unexpected boolean {value}"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("deserialize_any"), "{error}");
}
