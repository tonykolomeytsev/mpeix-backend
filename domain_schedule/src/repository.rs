use crate::sources;
use crate::time::DateTimeExt;
use anyhow::{anyhow, ensure, Context};
use chrono::{Local, Weekday};
use common_errors::errors::CommonError;
use domain_schedule_models::dto::v1::{Schedule, ScheduleType};
use lazy_static::lazy_static;

#[derive(Default)]
pub struct State {
    id_source_state: sources::id::State,
    schedule_source_state: sources::schedule::State,
}

pub async fn get_id(name: String, r#type: ScheduleType, state: &State) -> anyhow::Result<i64> {
    sources::id::get_id(&name, r#type, &state.id_source_state)
        .await
        .with_context(|| "Error while getting schedule id")
}

lazy_static! {
    static ref MAX_OFFSET: i32 = i32::MAX / 7;
    static ref MIN_OFFSET: i32 = i32::MIN / 7;
}

pub async fn get_schedule(
    name: String,
    r#type: ScheduleType,
    offset: i32,
    state: &State,
) -> anyhow::Result<Schedule> {
    ensure!(
        offset < *MAX_OFFSET,
        CommonError::user("Unacceptably large offset")
    );
    ensure!(
        offset > *MIN_OFFSET,
        CommonError::user("Unacceptably small offset")
    );
    let week_start = Local::now()
        .with_days_offset(offset * 7)
        .map(|dt| dt.date_naive())
        .map(|dt| dt.week(Weekday::Mon).first_day())
        .ok_or_else(|| {
            anyhow!(CommonError::user(format!(
                "Invalid week offset: {}",
                offset
            )))
        })?;
    sources::schedule::get_schedule(
        name,
        r#type,
        week_start,
        &state.schedule_source_state,
        &state.id_source_state,
    )
    .await
    .with_context(|| "Error while getting schedule")
}
