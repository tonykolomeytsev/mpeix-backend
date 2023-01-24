use std::collections::HashMap;

use crate::{
    dto::mpei::MpeiClasses,
    sources,
    time::{NaiveDateExt, WeekOfSemester},
};
use anyhow::{anyhow, bail, Context};
use chrono::{Datelike, Days, NaiveDate, NaiveTime};
use common_errors::errors::CommonError;
use common_in_memory_cache::InMemoryCache;
use domain_schedule_models::dto::v1::{
    self, Classes, ClassesTime, ClassesType, Day, ScheduleType, Week,
};
use reqwest::{redirect::Policy, Client, ClientBuilder};
use tokio::sync::Mutex;

#[derive(Hash, PartialEq, Eq)]
struct ScheduleKey {
    name: String,
    r#type: String,
    week_start: NaiveDate,
}

pub struct State {
    client: Client,
    cache: Mutex<InMemoryCache<ScheduleKey, v1::Schedule>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            client: ClientBuilder::new()
                .gzip(true)
                .deflate(true)
                .redirect(Policy::none())
                .timeout(std::time::Duration::from_secs(15))
                .connect_timeout(std::time::Duration::from_secs(3))
                .build()
                .expect("Something went wrong when building HttClient"),
            cache: Mutex::new(
                InMemoryCache::with_capacity(3000)
                    .expires_after_creation(chrono::Duration::hours(6)),
            ),
        }
    }
}

/// Get schedule from in-memory cache if value present in cache,
/// or get schedule from remote (`ts.mpei.ru`).
pub async fn get_schedule(
    name: String,
    r#type: ScheduleType,
    week_start: NaiveDate,
    schedule_source_state: &State,
    id_source_state: &sources::id::State,
) -> anyhow::Result<v1::Schedule> {
    let cache_key = ScheduleKey {
        name: name.to_owned(),
        r#type: r#type.to_mpei(),
        week_start: week_start.to_owned(),
    };
    let mut cache = schedule_source_state.cache.lock().await;
    if let Some(value) = cache.get(&cache_key) {
        return Ok(value.clone());
    }
    let schedule_id = sources::id::get_id(name, r#type.to_owned(), id_source_state)
        .await
        .with_context(|| "Error while using id_source from schedule_source")?;

    let week_end = week_start
        .checked_add_days(Days::new(6))
        .expect("Week end date always reachable");
    let schedule_response = schedule_source_state
        .client
        .get(format!(
            "http://ts.mpei.ru/api/schedule/{0}/{1}",
            r#type.to_mpei(),
            schedule_id
        ))
        .query(&[
            ("start", &week_start.format("%Y.%m.%d").to_string()),
            ("finish", &week_end.format("%Y.%m.%d").to_string()),
        ])
        .send()
        .await
        .map_err(|e| anyhow!(CommonError::gateway(e)))
        .with_context(|| "Error while executing a request to MPEI backend")?
        .json::<Vec<MpeiClasses>>()
        .await
        .map_err(|e| anyhow!(CommonError::internal(e)))
        .with_context(|| "Error while deserializing response from MPEI backend")?;

    map_schedule_models(&cache_key, schedule_id, r#type, schedule_response)
}

fn map_schedule_models(
    ScheduleKey {
        name,
        r#type: _,
        week_start,
    }: &ScheduleKey,
    schedule_id: i64,
    r#type: ScheduleType,
    mpei_classes: Vec<MpeiClasses>,
) -> anyhow::Result<v1::Schedule> {
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
    Ok(v1::Schedule {
        id: schedule_id.to_string(),
        name: name.to_owned(),
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
