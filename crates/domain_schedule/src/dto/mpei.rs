use chrono::{NaiveDate, NaiveTime};
use serde::{
    de::{self, Visitor},
    Deserialize,
};

#[derive(Debug, Deserialize)]
pub struct MpeiSearchResult {
    pub id: i64,
    /// Group name
    pub label: String,
    /// Faculty + description
    pub description: String,
    /// Enum: `group` | `person` | `room`
    #[serde(alias = "type")]
    pub r#type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MpeiClasses {
    /// Place
    pub auditorium: String,
    #[serde(deserialize_with = "deserialize_naive_time")]
    pub begin_lesson: NaiveTime,
    #[serde(deserialize_with = "deserialize_naive_time")]
    pub end_lesson: NaiveTime,
    #[serde(deserialize_with = "deserialize_naive_date")]
    pub date: NaiveDate,
    /// Name
    pub discipline: String,
    /// Type
    pub kind_of_work: String,
    /// Person
    pub lecturer: String,
    /// Group variations
    pub stream: Option<String>,
    pub group: Option<String>,
    pub sub_group: Option<String>,
}

struct NaiveDateVisitor;

impl<'de> Visitor<'de> for NaiveDateVisitor {
    type Value = NaiveDate;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a date formatted as 'yyyy.MM.dd'")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        NaiveDate::parse_from_str(v, "%Y.%m.%d").map_err(de::Error::custom)
    }
}

struct NaiveTimeVisitor;

impl<'de> Visitor<'de> for NaiveTimeVisitor {
    type Value = NaiveTime;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a time formatted as 'hh:mm'")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        NaiveTime::parse_from_str(v, "%H:%M").map_err(de::Error::custom)
    }
}

fn deserialize_naive_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: de::Deserializer<'de>,
{
    deserializer.deserialize_str(NaiveDateVisitor)
}

fn deserialize_naive_time<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
where
    D: de::Deserializer<'de>,
{
    deserializer.deserialize_str(NaiveTimeVisitor)
}
