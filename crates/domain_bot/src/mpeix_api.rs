use domain_schedule_models::{Schedule, ScheduleSearchResult};
use restix::{api, get};
use serde::Deserialize;

#[api(base_url = "http://localhost:8000/api/v1/schedules")]
pub trait MpeixApi {
    #[get("/v1/{type}/{name}/schedule/{offset}")]
    async fn schedule(&self, r#type: Path, name: Path, offset: Path) -> Schedule;

    #[get("/v1/search")]
    #[query(query = "q")]
    #[map_response_with(SearchResponse::items)]
    async fn search(&self, query: Query, r#type: Option<Query>) -> Vec<ScheduleSearchResult>;
}

#[derive(Deserialize)]
struct SearchResponse {
    items: Vec<ScheduleSearchResult>,
}

impl SearchResponse {
    fn items(self) -> Vec<ScheduleSearchResult> {
        self.items
    }
}
