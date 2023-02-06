use domain_schedule_models::dto::v1::{ScheduleSearchResult, ScheduleType};

use crate::mpeix_api::MpeixApi;

#[derive(Default)]
pub struct ScheduleSearchRepository(MpeixApi);

impl ScheduleSearchRepository {
    pub async fn search_schedule(
        &self,
        query: &str,
        r#type: &ScheduleType,
    ) -> anyhow::Result<Vec<ScheduleSearchResult>> {
        self.0.search_schedule(query, r#type).await
    }
}