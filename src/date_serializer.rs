use chrono::{DateTime, FixedOffset};
use serde::{de::Error, Deserialize, Deserializer};

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error> {
    let time: String = Deserialize::deserialize(deserializer)?;
    Ok(DateTime::parse_from_rfc3339(&time).map_err(D::Error::custom)?)
}