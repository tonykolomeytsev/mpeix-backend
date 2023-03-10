use anyhow::bail;
use common_errors::errors::CommonError;
use common_in_memory_cache::InMemoryCache;
use common_restix::ResultExt;
use common_rust::env;
use domain_schedule_models::ScheduleType;
use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use tokio::sync::Mutex;

use crate::{
    dto::{mpei::MpeiSearchResult, mpeix::ScheduleName as ValidScheduleName},
    mpei_api::MpeiApi,
};

lazy_static! {
    static ref SPACES_PATTERN: Regex = Regex::new(r"\s{2,}").unwrap();
}

pub struct ScheduleIdRepository {
    api: MpeiApi,
    cache: Mutex<InMemoryCache<ScheduleName, ScheduleId>>,
}

/// Helper struct for [ScheduleIdRepository]:
/// Key for in-memory cache
#[derive(Hash, PartialEq, Eq)]
struct ScheduleName {
    name: String,
    r#type: ScheduleType,
}

/// Helper struct for [ScheduleIdRepository]:
/// Value for in-memory cache
struct ScheduleId(i64);

impl ScheduleIdRepository {
    pub fn new(api: MpeiApi) -> Self {
        let cache_capacity = env::get_parsed_or("SCHEDULE_ID_CACHE_CAPACITY", 3000);
        let cache_max_hits = env::get_parsed_or("SCHEDULE_ID_CACHE_MAX_HITS", 10);
        let cache_lifetife = env::get_parsed_or("SCHEDULE_ID_CACHE_LIFETIME_HOURS", 12);

        Self {
            api,
            cache: Mutex::new(
                InMemoryCache::with_capacity(cache_capacity)
                    .max_hits(cache_max_hits)
                    .expires_after_creation(chrono::Duration::hours(cache_lifetife)),
            ),
        }
    }
}

impl ScheduleIdRepository {
    pub async fn get_id(
        &self,
        name: ValidScheduleName,
        r#type: ScheduleType,
    ) -> anyhow::Result<i64> {
        debug!("Trying to get schedule id from cache...");
        let cache_key = ScheduleName {
            r#type: r#type.to_owned(),
            name: name.to_string(),
        };
        if let Some(value) = self.cache.lock().await.get(&cache_key) {
            debug!("Got schedule id from cache");
            return Ok(value.0);
        };

        debug!("Getting schedule id from remote...");
        match self
            .get_id_from_remote(name.to_owned(), r#type.to_owned())
            .await?
        {
            Some(search_result) => {
                debug!("Got schedule id from remote");
                // Put value to cache
                self.cache
                    .lock()
                    .await
                    .insert(cache_key, ScheduleId(search_result.id));
                Ok(search_result.id)
            }
            _ => bail!(CommonError::user(format!(
                "Schedule with type '{:?}' and name '{}' not found",
                r#type, cache_key.name
            ))),
        }
    }

    async fn get_id_from_remote(
        &self,
        name: ValidScheduleName,
        r#type: ScheduleType,
    ) -> anyhow::Result<Option<MpeiSearchResult>> {
        let search_results = self
            .api
            .search(name.as_ref(), &r#type)
            .await
            .with_common_error()?;
        Ok(search_results
            .into_iter()
            .find(|result| self.fuzzy_equals(name.as_ref(), &result.label)))
    }

    fn fuzzy_equals(&self, a: &str, b: &str) -> bool {
        let clear_a = SPACES_PATTERN.replace_all(a, " ");
        let clear_b = SPACES_PATTERN.replace_all(b, " ");
        clear_a.to_lowercase() == clear_b.to_lowercase()
    }
}
