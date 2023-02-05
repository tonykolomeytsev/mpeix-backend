use std::{fmt::Display, str::FromStr};

use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub id: String,
    pub name: String,
    pub r#type: ScheduleType,
    pub weeks: Vec<Week>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScheduleType {
    Group,
    Person,
    Room,
}

pub struct ParseScheduleTypeError(String);

impl Display for ScheduleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Group => write!(f, "group"),
            Self::Person => write!(f, "person"),
            Self::Room => write!(f, "room"),
        }
    }
}

impl FromStr for ScheduleType {
    type Err = ParseScheduleTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "group" => Ok(Self::Group),
            "person" => Ok(Self::Person),
            "room" => Ok(Self::Room),
            _ => Err(ParseScheduleTypeError(s.to_owned())),
        }
    }
}

impl Display for ParseScheduleTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown schedule type: {}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Week {
    pub week_of_year: u8,
    pub week_of_semester: i8,
    pub first_day_of_week: NaiveDate,
    pub days: Vec<Day>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Day {
    pub day_of_week: u8,
    pub date: NaiveDate,
    pub classes: Vec<Classes>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Classes {
    pub name: String,
    pub r#type: ClassesType,
    pub raw_type: String,
    pub place: String,
    pub groups: String,
    pub person: String,
    pub time: ClassesTime,
    pub number: i8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClassesType {
    Undefined,
    Lecture,
    Practice,
    Lab,
    Course,
    Consultation,
    Exam,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClassesTime {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduleSearchResult {
    pub name: String,
    pub description: String,
    pub id: String,
    pub r#type: ScheduleType,
}
