use restix::{api, get};

use crate::dto::mpei::{MpeiClasses, MpeiSearchResult};

#[api(base_url = "http://ts.mpei.ru/api")]
pub trait MpeiApi {
    #[get("/search")]
    #[query(query = "term")]
    async fn search(&self, query: Query, r#type: Query) -> Vec<MpeiSearchResult>;

    #[get("/schedule/{type}/{id}")]
    async fn schedule(
        &self,
        r#type: Path,
        id: Path,
        start: Query,
        finish: Query,
    ) -> Vec<MpeiClasses>;
}
