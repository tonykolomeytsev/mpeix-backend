use chrono::{NaiveDate, NaiveTime};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub id: String,
    pub name: String,
    #[serde(alias = "type")]
    pub r#type: ScheduleType,
    pub weeks: Vec<Week>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScheduleType {
    Group,
    Person,
    Room,
}

impl ScheduleType {
    pub fn to_mpei(&self) -> String {
        match &self {
            Self::Group => "group".to_owned(),
            Self::Person => "peron".to_owned(),
            Self::Room => "room".to_owned(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Week {
    pub week_of_year: u8,
    pub week_of_semester: i8,
    pub first_day_of_week: NaiveDate,
    pub days: Vec<Day>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Day {
    pub day_of_week: u8,
    pub date: NaiveDate,
    pub classes: Vec<Classes>,
}

#[derive(Debug, Serialize, Clone)]
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

#[derive(Debug, Serialize, Clone)]
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

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClassesTime {
    pub start: NaiveTime,
    pub end: NaiveTime,
}
