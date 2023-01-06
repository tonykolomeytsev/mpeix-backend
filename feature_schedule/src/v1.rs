use domain_schedule::repository::{Error, State};
use domain_schedule_models::dto::v1::{self, ScheduleType};
use thiserror::Error;

#[derive(Default)]
pub struct FeatureScheduleState {
    schedule_repository_state: State,
}

#[derive(Debug, Error)]
pub enum FeatureScheduleError {
    #[error("Error when getting schedule id: {0}")]
    GetIdError(Error),
    #[error("Error when getting schedule: {0}")]
    GetScheduleError(Error),
}

pub async fn get_id(
    name: String,
    r#type: ScheduleType,
    state: &FeatureScheduleState,
) -> Result<i64, FeatureScheduleError> {
    domain_schedule::repository::get_id(name, r#type, &state.schedule_repository_state)
        .await
        .map_err(FeatureScheduleError::GetIdError)
}

pub async fn get_schedule(
    name: String,
    r#type: ScheduleType,
    offset: i32,
    state: &FeatureScheduleState,
) -> Result<v1::Schedule, FeatureScheduleError> {
    domain_schedule::repository::get_schedule(
        name,
        r#type,
        offset,
        &state.schedule_repository_state,
    )
    .await
    .map_err(FeatureScheduleError::GetScheduleError)
}
