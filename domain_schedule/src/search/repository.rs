use anyhow::{anyhow, Context};
use common_errors::errors::CommonError;
use common_in_memory_cache::InMemoryCache;
use domain_schedule_models::dto::v1::{ScheduleSearchResult, ScheduleType};
use reqwest::{redirect::Policy, Client, ClientBuilder};
use tokio::sync::Mutex;

use crate::dto::{mpei::MpeiSearchResult, mpeix::ScheduleSearchQuery};

use super::mapping::map_search_models;

const MPEI_API_SEARCH_ENDPOINT: &str = "http://ts.mpei.ru/api/search";
const MPEI_QUERY_TERM: &str = "term";
const MPEI_QUERY_TYPE: &str = "type";

pub struct ScheduleSearchRepository {
    client: Client,
    in_memory_cache: Mutex<InMemoryCache<TypedSearchQuery, Vec<ScheduleSearchResult>>>,
}

/// Helper struct for [ScheduleSearchRepository]:
/// Key for in-memory cache
#[derive(Hash, PartialEq, Eq)]
struct TypedSearchQuery(ScheduleSearchQuery, Option<ScheduleType>);

impl Default for ScheduleSearchRepository {
    fn default() -> Self {
        let cache_capacity = envmnt::get_usize("SCHEDULE_SEARCH_CACHE_CAPACITY", 3000);
        let cache_lifetife = envmnt::get_i64("SCHEDULE_SEARCH_CACHE_LIFETIME_MINUTES", 5);

        Self {
            client: ClientBuilder::new()
                .gzip(true)
                .deflate(true)
                .redirect(Policy::none())
                .timeout(std::time::Duration::from_secs(15))
                .connect_timeout(std::time::Duration::from_secs(3))
                .build()
                .expect("Something went wrong when building HttClient"),
            in_memory_cache: Mutex::new(
                InMemoryCache::with_capacity(cache_capacity)
                    .expires_after_creation(chrono::Duration::hours(cache_lifetife)),
            ),
        }
    }
}

impl ScheduleSearchRepository {
    pub async fn get_results_from_cache(
        &self,
        query: ScheduleSearchQuery,
        r#type: Option<ScheduleType>,
    ) -> Option<Vec<ScheduleSearchResult>> {
        let cache_key = TypedSearchQuery(query, r#type);
        if let Some(value) = self.in_memory_cache.lock().await.get(&cache_key) {
            return Some(value.to_owned());
        };
        None
    }

    pub async fn insert_results_to_cache(
        &self,
        query: ScheduleSearchQuery,
        r#type: Option<ScheduleType>,
        results: Vec<ScheduleSearchResult>,
    ) {
        self.in_memory_cache
            .lock()
            .await
            .insert(TypedSearchQuery(query, r#type), results);
    }

    pub async fn get_results_from_remote(
        &self,
        query: &ScheduleSearchQuery,
        r#type: &ScheduleType,
    ) -> anyhow::Result<Vec<ScheduleSearchResult>> {
        let search_results = self
            .client
            .get(MPEI_API_SEARCH_ENDPOINT)
            .query(&[
                (MPEI_QUERY_TERM, query.to_string()),
                (MPEI_QUERY_TYPE, r#type.to_string()),
            ])
            .send()
            .await
            .map_err(|e| anyhow!(CommonError::gateway(e)))
            .with_context(|| "Error while executing a request to MPEI backend")?
            .json::<Vec<MpeiSearchResult>>()
            .await
            .map_err(|e| anyhow!(CommonError::internal(e)))
            .with_context(|| "Error while deserializing response from MPEI backend")?;

        map_search_models(search_results)
            .with_context(|| "Error while mapping response from MPEI backend")
    }
}
