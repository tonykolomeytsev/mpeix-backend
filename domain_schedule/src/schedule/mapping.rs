use std::collections::HashMap;

use anyhow::bail;
use chrono::{Datelike, NaiveDate, NaiveTime};
use common_errors::errors::CommonError;
use domain_schedule_models::dto::v1::{
    Classes, ClassesTime, ClassesType, Day, Schedule, ScheduleType, Week,
};

use crate::{
    dto::mpei::MpeiClasses,
    time::{NaiveDateExt, WeekOfSemester},
};

pub fn map_schedule_models(
    name: String,
    week_start: NaiveDate,
    schedule_id: i64,
    r#type: ScheduleType,
    mpei_classes: Vec<MpeiClasses>,
) -> anyhow::Result<Schedule> {
    let mut map_of_days = HashMap::<NaiveDate, Vec<Classes>>::new();
    for ref cls in mpei_classes {
        let time = ClassesTime {
            start: cls.begin_lesson,
            end: cls.end_lesson,
        };
        let mpeix_cls = Classes {
            name: cls.discipline.to_owned(),
            r#type: get_classes_type(&cls.kind_of_work),
            raw_type: cls.kind_of_work.to_owned(),
            place: cls.auditorium.to_owned(),
            groups: match (&cls.stream, &cls.group) {
                (Some(stream), _) => stream.to_owned(),
                (None, Some(group)) => group.to_owned(),
                (_, _) => String::new(),
            },
            person: check_is_not_empty(&cls.lecturer),
            number: get_number(&time),
            time,
        };
        if !map_of_days.contains_key(&cls.date) {
            map_of_days.insert(cls.date.to_owned(), vec![]);
        }
        if let Some(vec) = map_of_days.get_mut(&cls.date) {
            vec.push(mpeix_cls)
        };
    }
    let mut days = Vec::<Day>::new();
    for (day_of_week, classes) in map_of_days {
        days.push(Day {
            day_of_week: day_of_week.weekday().number_from_monday() as u8,
            date: day_of_week,
            classes,
        });
    }
    days.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(Schedule {
        id: schedule_id.to_string(),
        name,
        r#type,
        weeks: vec![Week {
            week_of_semester: match week_start.week_of_semester() {
                Some(WeekOfSemester::Studying(num)) => num as i8,
                Some(WeekOfSemester::NonStudying) => -1,
                None => bail!(CommonError::internal(format!(
                    "Cannot calculate week of semester for offset {}",
                    week_start,
                ))),
            },
            week_of_year: week_start.week_of_year(),
            first_day_of_week: week_start.to_owned(),
            days,
        }],
    })
}

fn get_classes_type(raw_type: &str) -> ClassesType {
    let raw_type = raw_type.to_lowercase();
    if raw_type.contains("лек") {
        ClassesType::Lecture
    } else if raw_type.contains("лаб") {
        ClassesType::Lab
    } else if raw_type.contains("прак") {
        ClassesType::Practice
    } else if raw_type.contains("курс") || raw_type.contains("кп") {
        ClassesType::Course
    } else {
        ClassesType::Undefined
    }
}

fn check_is_not_empty(lecturer: &str) -> String {
    if lecturer.to_lowercase().contains("вакансия") {
        return String::new();
    }
    lecturer.trim().to_owned()
}

fn get_number(time: &ClassesTime) -> i8 {
    if time.start == NaiveTime::from_hms_opt(9, 20, 0).unwrap() {
        1
    } else if time.start == NaiveTime::from_hms_opt(11, 10, 0).unwrap() {
        2
    } else if time.start == NaiveTime::from_hms_opt(13, 45, 0).unwrap() {
        3
    } else if time.start == NaiveTime::from_hms_opt(15, 35, 0).unwrap() {
        4
    } else if time.start == NaiveTime::from_hms_opt(17, 20, 0).unwrap() {
        5
    } else if time.start == NaiveTime::from_hms_opt(18, 55, 0).unwrap() {
        6
    } else if time.start == NaiveTime::from_hms_opt(20, 30, 0).unwrap() {
        7
    } else {
        -1
    }
}
