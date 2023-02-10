use std::collections::HashMap;

use chrono::{Datelike, NaiveDate, Timelike};
use domain_schedule_models::dto::v1::{
    Classes, ClassesTime, ClassesType, Day, Schedule, ScheduleType, Week,
};

use crate::{
    dto::{mpei::MpeiClasses, mpeix::ScheduleName},
    time::{NaiveDateExt, WeekOfSemester},
};

pub(crate) fn map_schedule_models(
    name: ScheduleName,
    week_start: NaiveDate,
    schedule_id: i64,
    r#type: ScheduleType,
    mpei_classes: Vec<MpeiClasses>,
    week_of_semester: WeekOfSemester,
) -> Schedule {
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
    Schedule {
        id: schedule_id.to_string(),
        name: name.as_string(),
        r#type,
        weeks: vec![Week {
            week_of_semester: match week_of_semester {
                WeekOfSemester::Studying(num) => num as i8,
                WeekOfSemester::NonStudying => -1,
            },
            week_of_year: week_start.week_of_year(),
            first_day_of_week: week_start.to_owned(),
            days,
        }],
    }
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
    } else if raw_type.contains("экз") {
        ClassesType::Exam
    } else if raw_type.contains("консул") {
        ClassesType::Consultation
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
    match (time.start.hour(), time.start.minute()) {
        (9, 20) => 1,
        (11, 10) => 2,
        (13, 45) => 3,
        (15, 35) => 4,
        (17, 20) => 5,
        (18, 55) => 6,
        (20, 30) => 7,
        _ => -1,
    }
}
