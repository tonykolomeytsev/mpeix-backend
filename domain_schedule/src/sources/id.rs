use common_in_memory_cache::InMemoryCache;
use domain_schedule_models::dto::v1::ScheduleType;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{redirect::Policy, Client, ClientBuilder};
use thiserror::Error;
use tokio::sync::Mutex;

use crate::dto::mpei::MpeiSearchResult;

const CACHE_CAPACITY: usize = 3000;
const CACHE_LIFETIME_HOURS: i64 = 12;

const MPEI_API_SEARCH_ENDPOINT: &str = "http://ts.mpei.ru/api/search";
const MPEI_QUERY_TERM: &str = "term";
const MPEI_QUERY_TYPE: &str = "name";

/// Key for in-memory cache
#[derive(Hash, PartialEq, Eq)]
struct ScheduleName {
    name: String,
    r#type: String,
}

// Value for in-memory cache
struct ScheduleId(i64);

/// Global state for id source. Lives throughout the entire time the application is running.
pub struct State {
    client: Client,
    cache: Mutex<InMemoryCache<ScheduleName, ScheduleId>>,
}

impl Default for State {
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
                InMemoryCache::with_capacity(CACHE_CAPACITY)
                    .expires_after_creation(chrono::Duration::hours(CACHE_LIFETIME_HOURS)),
            ),
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Request error: {0}")]
    RequestError(reqwest::Error),
    #[error("Response deserialization error: {0}")]
    DeserializationError(reqwest::Error),
    #[error("Schedule with name \"{0}\" not found")]
    ScheduleNotFound(String),
}

/// Get `id` from in-memory cache if value present in cache,
/// or get `id` from remote (`ts.mpei.ru`).
pub async fn get_id(name: String, r#type: ScheduleType, state: &State) -> Result<i64, Error> {
    let cache_key = ScheduleName {
        r#type: r#type.to_mpei(),
        name: name.to_owned(),
    };
    let mut cache = state.cache.lock().await;
    if let Some(value) = cache.get(&cache_key) {
        return Ok(value.0);
    };
    let search_results = state
        .client
        .get(MPEI_API_SEARCH_ENDPOINT)
        .query(&[
            (MPEI_QUERY_TERM, name.to_owned()),
            (MPEI_QUERY_TYPE, r#type.to_mpei()),
        ])
        .send()
        .await
        .map_err(Error::RequestError)?
        .json::<Vec<MpeiSearchResult>>()
        .await
        .map_err(Error::DeserializationError)?;

    match search_results.first() {
        Some(search_result) if fuzzy_equals(&search_result.label, &name) => {
            // Put value to cache
            cache.insert(cache_key, ScheduleId(search_result.id));
            Ok(search_result.id)
        }
        _ => Err(Error::ScheduleNotFound(name.to_owned())),
    }
}

lazy_static! {
    static ref SPACES_PATTERN: Regex = Regex::new(r"\s{2,}").unwrap();
}

fn fuzzy_equals(a: &str, b: &str) -> bool {
    let clear_a = SPACES_PATTERN.replace_all(a, " ");
    let clear_b = SPACES_PATTERN.replace_all(b, " ");
    clear_a.to_lowercase() == clear_b.to_lowercase()
}
