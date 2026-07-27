use std::fmt;
use std::str::FromStr;

use chrono::{Local, NaiveDate};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::core::errors::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NoteDate(NaiveDate);

impl NoteDate {
    pub fn today_local() -> Self {
        Self(Local::now().date_naive())
    }
}

impl fmt::Display for NoteDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.format("%Y-%m-%d").fmt(formatter)
    }
}

impl FromStr for NoteDate {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let bytes = value.as_bytes();
        let canonical_shape = bytes.len() == 10
            && bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes
                .iter()
                .enumerate()
                .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit());
        if !canonical_shape {
            return Err(CoreError::InvalidDate {
                field: "date",
                value: value.to_string(),
            });
        }
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map(Self)
            .map_err(|_| CoreError::InvalidDate {
                field: "date",
                value: value.to_string(),
            })
    }
}

impl Serialize for NoteDate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for NoteDate {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone)]
pub struct Note {
    pub slug: String,
    pub title: String,
    pub date: NoteDate,
    pub tags: Vec<String>,
    pub updated_at: NoteDate,
    pub body: String,
}

#[cfg(test)]
mod tests {
    use super::NoteDate;
    use chrono::Local;

    #[test]
    fn note_date_accepts_real_iso_dates_and_orders_chronologically() {
        let leap: NoteDate = "2024-02-29".parse().unwrap();
        let later: NoteDate = "2024-03-01".parse().unwrap();

        assert!(leap < later);
        assert_eq!(leap.to_string(), "2024-02-29");
    }

    #[test]
    fn note_date_rejects_invalid_calendar_values_and_noncanonical_shapes() {
        for value in [
            "2023-02-29",
            "2026-00-01",
            "2026-13-01",
            "2026-04-31",
            "2026-7-01",
            "01-07-2026",
            "",
        ] {
            assert!(value.parse::<NoteDate>().is_err(), "value: {value:?}");
        }
    }

    #[test]
    fn note_date_serializes_as_a_plain_iso_string() {
        let date: NoteDate = "2026-07-27".parse().unwrap();
        let yaml = serde_yaml::to_string(&date).unwrap();

        assert_eq!(yaml, "2026-07-27\n");
        assert_eq!(serde_yaml::from_str::<NoteDate>(&yaml).unwrap(), date);
    }

    #[test]
    fn today_uses_the_local_calendar_day() {
        let before = Local::now().date_naive();
        let today = NoteDate::today_local();
        let after = Local::now().date_naive();

        assert!(today.0 == before || today.0 == after);
    }
}
