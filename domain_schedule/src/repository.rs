use crate::sources;
use crate::time::DateTimeExt;
use anyhow::{anyhow, Context};
use chrono::{Local, Weekday};
use common_errors::errors::CommonError;
use domain_schedule_models::dto::v1::{Schedule, ScheduleType};

#[derive(Default)]
pub struct State {
    id_source_state: sources::id::State,
    schedule_source_state: sources::schedule::State,
}

pub async fn get_id(name: String, r#type: ScheduleType, state: &State) -> anyhow::Result<i64> {
    sources::id::get_id(name, r#type, &state.id_source_state)
        .await
        .with_context(|| "Error while getting schedule id")
}

pub async fn get_schedule(
    name: String,
    r#type: ScheduleType,
    offset: i32,
    state: &State,
) -> anyhow::Result<Schedule> {
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
