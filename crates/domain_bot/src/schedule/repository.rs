use domain_schedule_models::dto::v1::{Schedule, ScheduleType};

use crate::mpeix_api::MpeixApi;

#[derive(Default)]
pub struct ScheduleRepository(MpeixApi);

impl ScheduleRepository {
    pub async fn get_schedule(
        &self,
        name: &str,
        r#type: &ScheduleType,
        offset: i8,
    ) -> anyhow::Result<Schedule> {
        self.0.get_schedule(name, r#type, offset as i32).await
    }
}
