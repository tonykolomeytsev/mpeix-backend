use domain_schedule_models::{Schedule, ScheduleSearchResult};
use restix::{api, get};
use serde::Deserialize;

#[api]
pub trait MpeixApi {
    #[get("/v1/{type}/{name}/schedule/{offset}")]
    async fn schedule(&self, r#type: Path, name: Path, offset: Path) -> Schedule;

    #[get("/v1/search")]
    #[query(query = "q")]
    #[map_response_with(map_search_response)]
    async fn search(&self, query: Query, r#type: Option<Query>) -> Vec<ScheduleSearchResult>;
}

#[derive(Deserialize)]
struct SearchResponse {
    items: Vec<ScheduleSearchResult>,
}

fn map_search_response(input: SearchResponse) -> Vec<ScheduleSearchResult> {
    input.items
}
