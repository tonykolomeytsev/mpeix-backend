use common_restix::ResultExt;
use domain_schedule_models::{ScheduleSearchResult, ScheduleType};

use crate::mpeix_api::MpeixApi;

/// Repository for accessing app_schedule microservice search results.
///
/// We do not need caching or other complex logic here, because it
/// is implemented on the side of the `app_schedule` microservice.
pub struct ScheduleSearchRepository(pub(crate) MpeixApi);

impl ScheduleSearchRepository {
    pub async fn search_schedule(
        &self,
        query: &str,
        r#type: Option<&ScheduleType>,
    ) -> anyhow::Result<Vec<ScheduleSearchResult>> {
        self.0
            .search(query, r#type.map(ToString::to_string))
            .await
            .with_common_error()
            .map(|it| it.items)
    }
}
