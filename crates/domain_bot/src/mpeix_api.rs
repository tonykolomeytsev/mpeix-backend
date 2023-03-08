use domain_schedule_models::{Schedule, ScheduleSearchResult, ScheduleType};
use restix::{api, get};
use serde::Deserialize;

#[api]
pub trait MpeixApi {
    #[get("/v1/{type}/{name}/schedule/{offset}")]
    async fn schedule(
        &self,
        #[path] r#type: &ScheduleType,
        #[path] name: &str,
        #[path] offset: i32,
    ) -> Schedule;

    #[get("/v1/search")]
    #[map_response_with(SearchResponse::items)]
    async fn search(
        &self,
        #[query("q")] query: &str,
        #[query] r#type: Option<ScheduleType>,
    ) -> Vec<ScheduleSearchResult>;
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
