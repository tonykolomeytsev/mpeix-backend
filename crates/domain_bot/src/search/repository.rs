use domain_schedule_models::dto::v1::{ScheduleSearchResult, ScheduleType};

use crate::mpeix_api::MpeixApi;

/// Repository for accessing app_schedule microservice search results.
///
/// We do not need caching or other complex logic here, because it
/// is implemented on the side of the `app_schedule` microservice.
#[derive(Default)]
pub struct ScheduleSearchRepository(MpeixApi);

impl ScheduleSearchRepository {
    pub async fn search_schedule(
        &self,
        query: &str,
        r#type: Option<&ScheduleType>,
    ) -> anyhow::Result<Vec<ScheduleSearchResult>> {
        self.0.search_schedule(query, r#type).await
    }
}
