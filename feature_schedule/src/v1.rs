use domain_schedule::usecases::{GetIdUseCase, GetScheduleUseCase};
use domain_schedule_models::dto::v1::{self, ScheduleType};

#[derive(Default)]
pub struct FeatureSchedule(GetIdUseCase, GetScheduleUseCase);

impl FeatureSchedule {
    pub async fn get_id(&self, name: String, r#type: ScheduleType) -> anyhow::Result<i64> {
        self.0.get_id(name, r#type).await
    }

    pub async fn get_schedule(
        &self,
        name: String,
        r#type: ScheduleType,
        offset: i32,
    ) -> anyhow::Result<v1::Schedule> {
        self.1.get_schedule(name, r#type, offset).await
    }
}
