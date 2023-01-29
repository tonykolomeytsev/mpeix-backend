use std::sync::Arc;

use anyhow::{bail, Context};
use common_in_memory_cache::InMemoryCache;
use deadpool_postgres::Pool;
use domain_schedule_models::dto::v1::{ScheduleSearchResult, ScheduleType};
use log::info;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

use crate::{dto::mpeix::ScheduleSearchQuery, mpei_api::MpeiApi};

use super::mapping::map_search_models;

pub struct ScheduleSearchRepository {
    api: MpeiApi,
    db_pool: Arc<Pool>,
    in_memory_cache: Mutex<InMemoryCache<TypedSearchQuery, Vec<ScheduleSearchResult>>>,
}

/// Helper struct for [ScheduleSearchRepository]:
/// Key for in-memory cache
#[derive(Hash, PartialEq, Eq)]
struct TypedSearchQuery(ScheduleSearchQuery, Option<ScheduleType>);

#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "users")] // singular 'user' is a keyword..
struct ScheduleSearchResultDb {
    pub name: String,
    pub description: String,
    pub id: String,
    pub r#type: String,
}

impl ScheduleSearchRepository {
    pub fn new(db_pool: Arc<Pool>) -> Self {
        let cache_capacity = envmnt::get_usize("SCHEDULE_SEARCH_CACHE_CAPACITY", 3000);
        let cache_lifetife = envmnt::get_i64("SCHEDULE_SEARCH_CACHE_LIFETIME_MINUTES", 5);

        Self {
            api: MpeiApi::default(),
            db_pool,
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
        map_search_models(self.api.search(query.as_ref(), r#type).await?)
            .with_context(|| "Error while mapping response from MPEI backend")
    }

    pub async fn init_schedule_search_results_db(&self) -> anyhow::Result<()> {
        let client = self.db_pool.get().await?;
        let stmt = include_str!("../../sql/create_schedule_search_results.pgsql");
        client
            .query(stmt, &[])
            .await
            .with_context(|| "Error during table 'schedule_search_results' creation")?;
        info!("Table 'schedule_search_results' initialization passed successfully");
        Ok(())
    }

    pub async fn get_results_from_db(
        &self,
        query: &ScheduleSearchQuery,
        r#type: Option<ScheduleType>,
    ) -> anyhow::Result<Vec<ScheduleSearchResult>> {
        let client = self.db_pool.get().await?;
        let stmt = if let Some(r#type) = r#type {
            include_str!("../../sql/select_all_schedule_search_results_typed.pgsql")
                .replace("$2", &r#type.to_string())
        } else {
            include_str!("../../sql/select_all_schedule_search_results.pgsql").to_string()
        }
        .replace("$1", query.as_ref());

        let results = client
            .query(&stmt, &[])
            .await?
            .iter()
            .map(|row| ScheduleSearchResultDb::from_row_ref(row).unwrap())
            .map(map_db_model)
            .collect::<anyhow::Result<Vec<ScheduleSearchResult>>>()
            .with_context(|| "Error while getting schedule search results from db")?;
        Ok(results)
    }
}

fn map_db_model(value: ScheduleSearchResultDb) -> anyhow::Result<ScheduleSearchResult> {
    Ok(ScheduleSearchResult {
        name: value.name,
        description: value.description,
        id: value.id,
        r#type: match value.r#type.as_str() {
            "group" => ScheduleType::Group,
            "person" => ScheduleType::Person,
            "room" => ScheduleType::Room,
            _ => bail!(
                "Database contains invalid schedule type value: '{}'",
                value.r#type
            ),
        },
    })
}
