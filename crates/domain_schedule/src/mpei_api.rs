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

// pub fn with_timeout_ms(timeout: u64) -> Self {
//     Self(
//         ClientBuilder::new()
//             .gzip(true)
//             .deflate(true)
//             .redirect(Policy::none())
//             .timeout(std::time::Duration::from_secs(15))
//             .connect_timeout(std::time::Duration::from_millis(timeout))
//             .pool_max_idle_per_host(3)
//             .build()
//             .expect("Something went wrong when building HttClient"),
//     )
// }
