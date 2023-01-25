use anyhow::{anyhow, bail, Context};
use common_errors::errors::CommonError;
use common_in_memory_cache::InMemoryCache;
use domain_schedule_models::dto::v1::ScheduleType;
use lazy_static::lazy_static;
use log::info;
use regex::Regex;
use reqwest::{redirect::Policy, Client, ClientBuilder};
use tokio::sync::Mutex;

use crate::dto::mpei::MpeiSearchResult;

const MPEI_API_SEARCH_ENDPOINT: &str = "http://ts.mpei.ru/api/search";
const MPEI_QUERY_TERM: &str = "term";
const MPEI_QUERY_TYPE: &str = "name";

lazy_static! {
    static ref SPACES_PATTERN: Regex = Regex::new(r"\s{2,}").unwrap();
}

pub struct ScheduleIdRepository {
    client: Client,
    cache: Mutex<InMemoryCache<ScheduleName, ScheduleId>>,
}

/// Helper struct for [GetIdUseCase]:
/// Key for in-memory cache
#[derive(Hash, PartialEq, Eq)]
struct ScheduleName {
    name: String,
    r#type: String,
}

/// Helper struct for [GetIdUseCase]:
/// Value for in-memory cache
struct ScheduleId(i64);

impl Default for ScheduleIdRepository {
    fn default() -> Self {
        Self {
            client: ClientBuilder::new()
                .gzip(true)
                .deflate(true)
                .redirect(Policy::none())
                .timeout(std::time::Duration::from_secs(15))
                .connect_timeout(std::time::Duration::from_secs(3))
                .build()
                .expect("Something went wrong when building HttClient"),
            cache: Mutex::new(
                InMemoryCache::with_capacity(3000)
                    .expires_after_creation(chrono::Duration::hours(12)),
            ),
        }
    }
}

impl ScheduleIdRepository {
    pub async fn get_id(&self, name: String, r#type: ScheduleType) -> anyhow::Result<i64> {
        let cache_key = ScheduleName {
            r#type: r#type.to_mpei(),
            name: name.to_owned(),
        };
        if let Some(value) = self.cache.lock().await.get(&cache_key) {
            info!("Got schedule id from cache");
            return Ok(value.0);
        };
        let search_results = self
            .client
            .get(MPEI_API_SEARCH_ENDPOINT)
            .query(&[
                (MPEI_QUERY_TERM, name.to_owned()),
                (MPEI_QUERY_TYPE, r#type.to_mpei()),
            ])
            .send()
            .await
            .map_err(|e| anyhow!(CommonError::gateway(e)))
            .with_context(|| "Error while executing a request to MPEI backend")?
            .json::<Vec<MpeiSearchResult>>()
            .await
            .map_err(|e| anyhow!(CommonError::internal(e)))
            .with_context(|| "Error while deserializing response from MPEI backend")?;

        match search_results.first() {
            Some(search_result) if self.fuzzy_equals(&search_result.label, &name) => {
                info!("Got schedule id from remote");
                // Put value to cache
                self.cache
                    .lock()
                    .await
                    .insert(cache_key, ScheduleId(search_result.id));
                Ok(search_result.id)
            }
            _ => bail!(CommonError::user(format!(
                "Schedule with type '{:?}' and name '{}' not found",
                r#type, name
            ))),
        }
    }

    fn fuzzy_equals(&self, a: &str, b: &str) -> bool {
        let clear_a = SPACES_PATTERN.replace_all(a, " ");
        let clear_b = SPACES_PATTERN.replace_all(b, " ");
        clear_a.to_lowercase() == clear_b.to_lowercase()
    }
}
