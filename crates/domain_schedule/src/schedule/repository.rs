use anyhow::Context;
use chrono::{Days, NaiveDate};
use common_in_memory_cache::InMemoryCache;
use common_persistent_cache::PersistentCache;
use domain_schedule_models::dto::v1::{Schedule, ScheduleType};
use tokio::sync::Mutex;

use crate::{dto::mpeix::ScheduleName, mpei_api::MpeiApi, time::WeekOfSemester};

use super::{
    mapping::map_schedule_models,
    mediator::{CacheMediator, InMemoryCacheKey},
};

pub struct ScheduleRepository {
    api: MpeiApi,
    in_memory_cache: Mutex<InMemoryCache<InMemoryCacheKey, Schedule>>,
    persistent_cache: Mutex<PersistentCache>,
}

impl Default for ScheduleRepository {
    fn default() -> Self {
        let cache_capacity = envmnt::get_usize("SCHEDULE_CACHE_CAPACITY", 500);
        let cache_max_hits = envmnt::get_u32("SCHEDULE_CACHE_MAX_HITS", 10);
        let cache_lifetife = envmnt::get_i64("SCHEDULE_CACHE_LIFETIME_HOURS", 6);
        let cache_dir = envmnt::get_or("SCHEDULE_CACHE_DIR", "./cache");

        Self {
            api: MpeiApi::default(),
            in_memory_cache: Mutex::new(
                InMemoryCache::with_capacity(cache_capacity)
                    .max_hits(cache_max_hits)
                    .expires_after_creation(chrono::Duration::hours(cache_lifetife)),
            ),
            persistent_cache: Mutex::new(PersistentCache::new(cache_dir.into())),
        }
    }
}

impl ScheduleRepository {
    pub async fn get_schedule_from_cache(
        &self,
        name: ScheduleName,
        r#type: ScheduleType,
        week_start: NaiveDate,
        ignore_expiration: bool,
    ) -> anyhow::Result<Option<Schedule>> {
        let mut mediator = CacheMediator::new(&self.in_memory_cache, &self.persistent_cache);
        let key = InMemoryCacheKey {
            name: name.as_string(),
            r#type: r#type.to_string(),
            week_start,
        };

        mediator
            .get(&key, ignore_expiration)
            .await
            .with_context(|| "Error while getting schedule from cache via CacheMediator")
    }

    pub async fn insert_schedule_to_cache(
        &self,
        name: ScheduleName,
        r#type: ScheduleType,
        week_start: NaiveDate,
        schedule: Schedule,
    ) -> anyhow::Result<()> {
        let mut mediator = CacheMediator::new(&self.in_memory_cache, &self.persistent_cache);
        let key = InMemoryCacheKey {
            name: name.as_string(),
            r#type: r#type.to_string(),
            week_start,
        };

        mediator
            .insert(key, schedule)
            .await
            .with_context(|| "Error while inserting schedule to cache via CacheMediator")
    }

    pub async fn get_schedule_from_remote(
        &self,
        schedule_id: i64,
        name: ScheduleName,
        r#type: ScheduleType,
        week_start: NaiveDate,
        week_of_semester: WeekOfSemester,
    ) -> anyhow::Result<Schedule> {
        let week_end = week_start
            .checked_add_days(Days::new(6))
            .expect("Week end date always reachable");

        let schedule_response = self
            .api
            .get_schedule(r#type.to_owned(), schedule_id, &week_start, &week_end)
            .await?;

        Ok(map_schedule_models(
            name,
            week_start,
            schedule_id,
            r#type,
            schedule_response,
            week_of_semester,
        ))
    }
}