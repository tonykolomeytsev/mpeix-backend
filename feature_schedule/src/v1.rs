use domain_schedule::repository::State;
use domain_schedule_models::dto::v1::{self, ScheduleType};

#[derive(Default)]
pub struct FeatureScheduleState {
    schedule_repository_state: State,
}

pub async fn get_id(
    name: String,
    r#type: ScheduleType,
    state: &FeatureScheduleState,
) -> anyhow::Result<i64> {
    domain_schedule::repository::get_id(name, r#type, &state.schedule_repository_state).await
}

pub async fn get_schedule(
    name: String,
    r#type: ScheduleType,
    offset: i32,
    state: &FeatureScheduleState,
) -> anyhow::Result<v1::Schedule> {
    domain_schedule::repository::get_schedule(
        name,
        r#type,
        offset,
        &state.schedule_repository_state,
    )
    .await
}
