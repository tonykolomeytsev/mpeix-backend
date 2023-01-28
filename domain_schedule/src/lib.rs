use anyhow::bail;
use common_errors::errors::CommonError;
use domain_schedule_models::dto::v1::ScheduleType;

pub mod di;
pub mod dto;
pub(crate) mod id;
pub(crate) mod schedule;
pub(crate) mod search;
pub(crate) mod time;
pub mod usecases;

/// Because we cannot implement trait `actix_web::FromRequest` for `ScheduleType`.
/// They belongs to different crates and no one belongs this crate.
/// I do not want to add `actix-web` dependency to `domain_schedule_models` crate.
pub fn parse_schedule_type(r#type: &str) -> anyhow::Result<ScheduleType> {
    match r#type {
        "group" => Ok(ScheduleType::Group),
        "person" => Ok(ScheduleType::Person),
        "room" => Ok(ScheduleType::Room),
        _ => bail!(CommonError::UserError(format!(
            "Unsupported schedule type: {type}"
        ))),
    }
}
