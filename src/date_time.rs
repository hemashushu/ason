// Copyright (c) 2026 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

//! Serde mapping for ASON DateTime literals.
//! Use `#[serde(with = "ason::date_time")]` on a chrono field, or `DateTime`
//! for a root value. Serialization requires a four-digit local year,
//! whole-second precision, and a whole-minute UTC offset.
//!
//! ```
//! let time = chrono::DateTime::parse_from_rfc3339("2024-03-16T12:00:00+08:00")
//!     .unwrap();
//! let value = ason::date_time::DateTime(time);
//! let text = ason::ser_to_string(&value)?;
//! assert_eq!(text, r#"d"2024-03-16T12:00:00+08:00""#);
//! assert_eq!(ason::de_from_str::<ason::date_time::DateTime>(&text)?, value);
//! # Ok::<(), ason::AsonError>(())
//! ```

use chrono::{DateTime as ChronoDateTime, Datelike, FixedOffset, SecondsFormat, TimeDelta};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Internal Serde newtype name used to select ASON's native DateTime mapping.
///
/// Serde has no DateTime primitive. Calling `serialize_str` directly would make
/// the RFC 3339 payload indistinguishable from an ordinary String, so this
/// adapter passes the marker as the `name` argument to
/// `serialize_newtype_struct` and `deserialize_newtype_struct` instead.
///
/// The ASON serializer recognizes this name, writes the `d` prefix, and then
/// serializes the string payload, producing a literal such as
/// `d"2024-03-16T00:00:00Z"`. The ASON deserializer recognizes the same name,
/// requires a DateTime token rather than a String token, and passes its
/// normalized RFC 3339 text to this adapter's newtype visitor.
///
/// This name is only an in-memory argument to Serde calls; it is not written
/// into the ASON document. ASONB uses its own `$asonb::DateTime` marker for its
/// binary adapter and likewise does not write that marker into its documents.
/// The markers are implementation conventions, not part of either wire format.
///
/// The `$ason::` prefix reduces collisions with ordinary newtype names, but is
/// not special to Serde or an unforgeable type identifier. Crate-only visibility
/// keeps the convention internal. Other formats need not recognize this name,
/// and ordinary ASON newtype structs remain unsupported.
pub(crate) const MARKER: &str = "$ason::DateTime";

/// A chrono DateTime serialized as an ASON DateTime rather than a String.
///
/// Use this owned wrapper for a root value or inside containers. For a chrono
/// field on a Serde struct, `#[serde(with = "ason::date_time")]` provides the
/// same mapping without changing the field's type. Serialization requires a
/// local year in `0000..=9999`, whole seconds, and a whole-minute UTC offset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTime(
    /// The wrapped chrono DateTime, retaining its fixed UTC offset.
    pub ChronoDateTime<FixedOffset>,
);

impl Serialize for DateTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserialize(deserializer).map(Self)
    }
}

/// Serializes a chrono DateTime using the native ASON DateTime mapping.
///
/// Intended for `#[serde(with = "ason::date_time")]` or `serialize_with` on a
/// field. The offset is retained in the emitted DateTime literal.
///
/// # Errors
///
/// Returns an error if the local year is outside `0000..=9999`, the value has
/// fractional-second precision, the offset is not a whole number of minutes,
/// or the serializer fails. An exactly represented leap second is accepted.
pub fn serialize<S: Serializer>(
    value: &ChronoDateTime<FixedOffset>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let local = value
        .naive_utc()
        .checked_add_signed(TimeDelta::seconds(i64::from(
            value.offset().local_minus_utc(),
        )));
    // if value.timestamp_subsec_nanos() % 1_000_000_000 != 0
    if !value.timestamp_subsec_nanos().is_multiple_of(1_000_000_000)
        || value.offset().local_minus_utc() % 60 != 0
        || !local.is_some_and(|local| (0..=9999).contains(&local.year()))
    {
        return Err(serde::ser::Error::custom(
            "ASON DateTime requires a four-digit year, whole seconds, and a whole-minute UTC offset.",
        ));
    }
    serializer.serialize_newtype_struct(MARKER, &value.to_rfc3339_opts(SecondsFormat::Secs, true))
}

/// Deserializes an ASON DateTime into a chrono DateTime with a fixed offset.
///
/// Intended for `#[serde(with = "ason::date_time")]` or `deserialize_with`
/// on a field. In ASON, this requires a DateTime literal rather than an
/// ordinary string.
///
/// # Errors
///
/// Returns an error if the input is not a valid DateTime or the underlying
/// deserializer fails.
pub fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<ChronoDateTime<FixedOffset>, D::Error> {
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = ChronoDateTime<FixedOffset>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an ASON DateTime")
        }

        fn visit_newtype_struct<D: Deserializer<'de>>(
            self,
            deserializer: D,
        ) -> Result<Self::Value, D::Error> {
            let text = String::deserialize(deserializer)?;
            ChronoDateTime::parse_from_rfc3339(&text).map_err(serde::de::Error::custom)
        }
    }

    deserializer.deserialize_newtype_struct(MARKER, Visitor)
}

#[cfg(test)]
mod tests {
    use crate::{de_from_str, ser_to_string};

    use super::*;

    #[test]
    fn test_date_time_root_round_trip() {
        for text in [
            "0000-01-01T00:00:00Z",
            "1969-12-31T23:59:59Z",
            "2024-03-16T16:30:50+08:00",
            "2024-03-16T16:30:50-05:30",
            "2024-03-16T16:30:50-23:59",
            "2024-03-16T16:30:50+23:59",
            "2016-12-31T23:59:60Z",
            "9999-12-31T23:59:59Z",
        ] {
            let value = DateTime(ChronoDateTime::parse_from_rfc3339(text).unwrap());
            let encoded = ser_to_string(&value).unwrap();
            assert_eq!(encoded, format!("d\"{text}\""));
            let decoded = de_from_str::<DateTime>(&encoded).unwrap();
            assert_eq!(decoded, value);
            assert_eq!(decoded.0.offset(), value.0.offset());
        }
    }

    #[test]
    fn test_date_time_local_to_utc() {
        for (offset, utc) in [
            ("+08:00", "2024-03-16T08:30:50Z"),
            ("-05:30", "2024-03-16T22:00:50Z"),
            ("+23:59", "2024-03-15T16:31:50Z"),
            ("-23:59", "2024-03-17T16:29:50Z"),
        ] {
            let text = format!("d\"2024-03-16T16:30:50{offset}\"");
            let decoded = de_from_str::<DateTime>(&text).unwrap();
            let expected = ChronoDateTime::parse_from_rfc3339(utc).unwrap();
            assert_eq!(decoded.0.naive_local().to_string(), "2024-03-16 16:30:50");
            assert_eq!(decoded.0.naive_utc(), expected.naive_utc());
            assert_eq!(decoded.0.timestamp(), expected.timestamp());
        }
    }

    #[test]
    fn test_date_time_omitted_parts() {
        for text in [
            r#"d"2024-03-16""#,
            r#"d"2024-03-16 00:00:00""#,
            r#"d"2024-03-16t00:00:00""#,
            r#"d"2024-03-16T00:00:00z""#,
            r#"d"2024-03-16T00:00:00+00:00""#,
        ] {
            let decoded = de_from_str::<DateTime>(text).unwrap();
            assert_eq!(
                ser_to_string(&decoded).unwrap(),
                r#"d"2024-03-16T00:00:00Z""#,
            );
        }
    }

    #[test]
    fn test_date_time_invalid_input() {
        for text in [
            "",
            "true",
            r#"h"00""#,
            r#""2024-03-16T00:00:00Z""#,
            r#"d"2024-02-30""#,
            r#"d"2024-03-16T00:00:00+24:00""#,
            r#"d"2024-03-16T00:00:00.123Z""#,
            r#"d"2024-03-16T00:00:00+08:00:01""#,
            r#"d"2024-03-16" true"#,
            "d\"2024-03-16",
        ] {
            assert!(de_from_str::<DateTime>(text).is_err(), "{text}");
        }
        assert!(de_from_str::<String>(r#"d"2024-03-16""#).is_err());
        assert_eq!(
            ser_to_string(&"2024-03-16T00:00:00Z").unwrap(),
            r#""2024-03-16T00:00:00Z""#,
        );
    }

    #[test]
    fn test_date_time_unrepresentable_values() {
        for text in ["2024-03-16T16:30:50.000000001Z", "2016-12-31T23:59:60.5Z"] {
            let value = DateTime(ChronoDateTime::parse_from_rfc3339(text).unwrap());
            assert!(ser_to_string(&value).is_err(), "{text}");
        }
        for year in [-1, 10000] {
            let local = chrono::NaiveDate::from_ymd_opt(year, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            let value = DateTime(local.and_utc().fixed_offset());
            assert!(ser_to_string(&value).is_err(), "{year}");
        }
        for offset in [-28_801, 28_801] {
            let value = DateTime(
                ChronoDateTime::from_timestamp(0, 0)
                    .unwrap()
                    .with_timezone(&FixedOffset::east_opt(offset).unwrap()),
            );
            assert!(ser_to_string(&value).is_err());
        }
        let value = DateTime(
            ChronoDateTime::<chrono::Utc>::MAX_UTC
                .with_timezone(&FixedOffset::east_opt(3600).unwrap()),
        );
        assert!(ser_to_string(&value).is_err());
    }
}
