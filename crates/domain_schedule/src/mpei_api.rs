use domain_schedule_models::ScheduleType;
use restix::{api, get};

use crate::dto::mpei::{MpeiClasses, MpeiSearchResult};

#[api(base_url = "http://ts.mpei.ru/api")]
pub trait MpeiApi {
    #[get("/search")]
    async fn search(
        &self,
        #[query("term")] query: &str,
        #[query] r#type: &ScheduleType,
    ) -> Vec<MpeiSearchResult>;

    #[get("/schedule/{type}/{id}")]
    async fn schedule(
        &self,
        #[path] r#type: &ScheduleType,
        #[path] id: i64,
        #[query] start: &str,
        #[query] finish: &str,
        #[query] lng: u8,
    ) -> Vec<MpeiClasses>;
}
