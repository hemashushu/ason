// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

//! Use `#[serde(with = "ason::array")]` to map a fixed-length array to an ASON List.
//! Byte arrays also map to Lists; use `serde_bytes` to select Byte Data instead.
//! Only the annotated array's outer container is adapted; nested arrays retain
//! their normal Serde representation.
//!
//! ```
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, PartialEq, Serialize, Deserialize)]
//! struct Samples {
//!     #[serde(with = "ason::array")]
//!     values: [i32; 2],
//! }
//!
//! let samples = Samples { values: [1, 2] };
//! let text = ason::ser_to_string(&samples)?;
//! assert!(text.contains("values: ["));
//! assert_eq!(ason::de_from_str::<Samples>(&text)?, samples);
//! # Ok::<(), ason::AsonError>(())
//! ```

use std::marker::PhantomData;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// Serializes a fixed-length array as a Serde sequence.
///
/// With ASON this produces a List, including for `[u8; N]`. Intended for
/// `#[serde(with = "ason::array")]` or `serialize_with` on an array field.
/// Nested element types keep their own Serde mappings.
///
/// # Errors
///
/// Propagates an error from the serializer or an element's serialization.
pub fn serialize<T, S, const N: usize>(value: &[T; N], serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    value.as_slice().serialize(serializer)
}

/// Deserializes a Serde sequence into a fixed-length array.
///
/// With ASON the input must be a List containing exactly `N` elements.
/// Intended for `#[serde(with = "ason::array")]` or `deserialize_with` on an
/// array field. A zero-length array accepts an empty List.
///
/// # Errors
///
/// Returns an error if the input is not a sequence, has a different length,
/// or contains an element that cannot be deserialized as `T`.
pub fn deserialize<'de, T, D, const N: usize>(deserializer: D) -> Result<[T; N], D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    struct Visitor<T, const N: usize>(PhantomData<T>);

    impl<'de, T: Deserialize<'de>, const N: usize> de::Visitor<'de> for Visitor<T, N> {
        type Value = [T; N];

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a List with exactly {N} elements")
        }

        fn visit_seq<A: de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            let mut values = Vec::new();
            for index in 0..N {
                let value = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(index, &self))?;
                values.push(value);
            }
            if sequence.next_element::<de::IgnoredAny>()?.is_some() {
                return Err(de::Error::invalid_length(N.saturating_add(1), &self));
            }
            values
                .try_into()
                .map_err(|values: Vec<T>| de::Error::invalid_length(values.len(), &self))
        }
    }

    deserializer.deserialize_seq(Visitor::<T, N>(PhantomData))
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::{de_from_str, ser_to_string};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
    struct Record<T, const N: usize> {
        #[serde(with = "crate::array")]
        values: [T; N],
    }

    fn assert_round_trip<T, const N: usize>(values: [T; N])
    where
        T: std::fmt::Debug + PartialEq + Serialize + serde::de::DeserializeOwned,
    {
        let record = Record { values };
        let text = ser_to_string(&record).unwrap();
        assert_eq!(de_from_str::<Record<T, N>>(&text).unwrap(), record);
    }

    #[test]
    fn test_array_field_round_trip() {
        let record = Record {
            values: [11, 13, 17, 19],
        };
        let text = ser_to_string(&record).unwrap();
        assert_eq!(
            text,
            "{\n    values: [\n        11\n        13\n        17\n        19\n    ]\n}"
        );
        assert_eq!(de_from_str::<Record<i32, 4>>(&text).unwrap(), record);
        for text in [
            "{values: [11, 13, 17, 19,]}",
            "{values: [11 13 17 19]}",
            "{values: [11\n13\n17\n19]}",
        ] {
            assert_eq!(de_from_str::<Record<i32, 4>>(text).unwrap(), record);
        }
    }

    #[test]
    fn test_empty_and_large_arrays() {
        let record = Record::<i32, 0> { values: [] };
        assert_eq!(ser_to_string(&record).unwrap(), "{\n    values: []\n}");
        assert_round_trip::<i32, 0>([]);
        assert_round_trip([42u16; 64]);
        assert_round_trip([true]);
        assert_round_trip([0u8, 255]);
    }

    #[test]
    fn test_non_copy_and_nested_elements() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct Item {
            name: String,
        }

        assert_round_trip(["foo".to_owned(), "bar".to_owned()]);
        assert_round_trip([
            Item {
                name: "foo".to_owned(),
            },
            Item {
                name: "bar".to_owned(),
            },
        ]);
        assert_round_trip([Record { values: [11, 13] }, Record { values: [17, 19] }]);

        let record = Record {
            values: [[11, 13], [17, 19]],
        };
        let text = ser_to_string(&record).unwrap();
        assert!(text.contains("(11 13)"));
        assert_eq!(de_from_str::<Record<[i32; 2], 2>>(&text).unwrap(), record);
    }

    #[test]
    fn test_array_length_mismatch() {
        for text in [
            "{values: []}",
            "{values: [11 13 17]}",
            "{values: [11 13 17 19 23]}",
            "{values: [11 13 17 19 {extra: [23]}]}",
        ] {
            let error = de_from_str::<Record<i32, 4>>(text).unwrap_err();
            assert!(error.to_string().contains("exactly 4 elements"), "{error}");
        }
        let error = de_from_str::<Record<i32, 0>>("{values: [true]}").unwrap_err();
        assert!(error.to_string().contains("exactly 0 elements"), "{error}");
    }

    #[test]
    fn test_array_invalid_input() {
        for text in [
            "{values: (11 13 17 19)}",
            "{values: h\"00 01 02 03\"}",
            "{values: [11 13 true 19]}",
            "{values: [11 13",
            "{values: [11 13 17 19",
            "{values: [11 13 17 19]} true",
            "{values: [11 13 17 19)}",
        ] {
            assert!(de_from_str::<Record<i32, 4>>(text).is_err(), "{text}");
        }
    }
}
