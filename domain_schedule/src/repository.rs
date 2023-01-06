use crate::sources;
use crate::time::DateTimeExt;
use chrono::{Local, Weekday};
use domain_schedule_models::dto::v1::{Schedule, ScheduleType};
use thiserror::Error;

#[derive(Default)]
pub struct State {
    id_source_state: sources::id::State,
    schedule_source_state: sources::schedule::State,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error while getting schedule id: {0}")]
    IdSourceError(sources::id::Error),
    #[error("Error while getting schedule: {0}")]
    ScheduleSourceError(sources::schedule::Error),
    #[error("Cannot calculate date for given offset: {0}")]
    InvalidWeekOffset(i32),
}

pub async fn get_id(name: String, r#type: ScheduleType, state: &State) -> Result<i64, Error> {
    sources::id::get_id(name, r#type, &state.id_source_state)
        .await
        .map_err(Error::IdSourceError)
}

pub async fn get_schedule(
    name: String,
    r#type: ScheduleType,
    offset: i32,
    state: &State,
) -> Result<Schedule, Error> {
    let week_start = Local::now()
        .with_days_offset(offset * 7)
        .map(|dt| dt.date_naive())
        .map(|dt| dt.week(Weekday::Mon).first_day())
        .ok_or_else(|| Error::InvalidWeekOffset(offset))?;
    sources::schedule::get_schedule(
        name,
        r#type,
        week_start,
        &state.schedule_source_state,
        &state.id_source_state,
    )
    .await
    .map_err(Error::ScheduleSourceError)
}
