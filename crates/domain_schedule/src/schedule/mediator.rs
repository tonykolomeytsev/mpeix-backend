use std::hash::Hash;

use anyhow::{anyhow, Ok};
use chrono::{Datelike, NaiveDate};
use common_errors::errors::CommonError;
use common_in_memory_cache::{Entry, InMemoryCache};
use common_persistent_cache::PersistentCache;
use domain_schedule_models::dto::v1::Schedule;
use tokio::sync::Mutex;

use super::compat::{writing, ReadingPersistentEntry, WritingPersistentEntry};

pub struct CacheMediator<'a> {
    in_memory_cache: &'a Mutex<InMemoryCache<InMemoryCacheKey, Schedule>>,
    persistent_cache: &'a Mutex<PersistentCache>,
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct InMemoryCacheKey {
    pub name: String,
    pub r#type: String,
    pub week_start: NaiveDate,
}

impl<'a> CacheMediator<'a> {
    pub fn new(
        in_memory_cache: &'a Mutex<InMemoryCache<InMemoryCacheKey, Schedule>>,
        persistent_cache: &'a Mutex<PersistentCache>,
    ) -> Self {
        Self {
            in_memory_cache,
            persistent_cache,
        }
    }

    pub async fn get(
        &mut self,
        key: &InMemoryCacheKey,
        ignore_expiration: bool,
    ) -> anyhow::Result<Option<Schedule>> {
        // restore value to the lru cache from file, if needed
        if !self.in_memory_cache.lock().await.contains(key) {
            self.restore_from_persistent(key).await?;
        }
        // return value if exists and satisfies expiration policy
        if let Some((schedule, expired)) = self.in_memory_cache.lock().await.peek(key) {
            if !expired || ignore_expiration {
                return Ok(Some(schedule.to_owned()));
            }
        }
        Ok(None)
    }

    async fn restore_from_persistent(&mut self, key: &InMemoryCacheKey) -> anyhow::Result<()> {
        if let Some(entry) = self
            .persistent_cache
            .lock()
            .await
            .get::<String, ReadingPersistentEntry>(key.to_string())
            .await
            .map_err(|e| anyhow!(CommonError::internal(e)))?
        {
            self.push_to_lru(key, entry.into()).await?;
        }
        Ok(())
    }

    async fn push_to_lru(
        &mut self,
        key: &InMemoryCacheKey,
        entry: Entry<Schedule>,
    ) -> anyhow::Result<()> {
        if let Some((lru_key, lru_entry)) = self
            .in_memory_cache
            .lock()
            .await
            .insert_entry(key.to_owned(), entry)
        {
            // ignore entry update, do not ignore entry extrusion
            if &lru_key == key {
                return Ok(());
            }
            self.persistent_cache
                .lock()
                .await
                .insert::<String, WritingPersistentEntry>(lru_key.to_string(), &writing(&lru_entry))
                .await
                .map_err(|e| anyhow!(CommonError::internal(e)))?;
        }
        Ok(())
    }

    pub async fn insert(&mut self, key: InMemoryCacheKey, value: Schedule) -> anyhow::Result<()> {
        let entry = Entry::new(value);
        // immediately write provided value to the persistent cache
        self.persistent_cache
            .lock()
            .await
            .insert::<String, WritingPersistentEntry>(key.to_string(), &writing(&entry))
            .await
            .map_err(|e| anyhow!(CommonError::internal(e)))?;

        self.push_to_lru(&key, entry).await
    }
}

impl ToString for InMemoryCacheKey {
    fn to_string(&self) -> String {
        let year = &self.week_start.year();
        let r#type = &self.r#type.to_lowercase();
        let name = &self.name.to_uppercase();

        format!(
            "{}/{} {} [{}].cache",
            year,
            r#type,
            name,
            &self.week_start.format("%Y-%m-%d")
        )
    }
}
