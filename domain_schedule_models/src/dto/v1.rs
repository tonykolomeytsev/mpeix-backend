use std::fmt::Display;

use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub id: String,
    pub name: String,
    #[serde(alias = "type")]
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

impl Display for ScheduleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Group => write!(f, "group"),
            Self::Person => write!(f, "person"),
            Self::Room => write!(f, "room"),
        }
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
