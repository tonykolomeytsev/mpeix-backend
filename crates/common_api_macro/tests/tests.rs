use common_api_macro::{common_api, get, query, Body, Path, Query, Result};

#[common_api(base_url = "http://ts.mpei.ru/api")]
trait MpeiApi {
    #[get("/search")]
    #[query(q = "term")]
    async fn search(&self, q: Query, r#type: Query) -> Result<Vec<String>>;

    #[get("/schedule/{type}/{schedule_id}")]
    async fn schedule(&self, r#type: Path, schedule_id: Path) -> Result<String>;
}
