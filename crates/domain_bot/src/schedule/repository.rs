use common_restix::ResultExt;
use domain_schedule_models::{Schedule, ScheduleType};

use crate::mpeix_api::MpeixApi;

/// Repository for accessing `app_schedule` microservice schedules.
///
/// We do not need caching or other complex logic here, because it
/// is implemented on the side of the `app_schedule` microservice.
pub struct ScheduleRepository(pub(crate) MpeixApi);

impl ScheduleRepository {
    pub async fn get_schedule(
        &self,
        name: &str,
        r#type: &ScheduleType,
        offset: i8,
    ) -> anyhow::Result<Schedule> {
        self.0
            .schedule(name, r#type, offset as i32)
            .await
            .with_common_error()
    }
}
