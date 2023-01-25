use anyhow::{anyhow, Context};
use chrono::{Days, NaiveDate};
use common_errors::errors::CommonError;
use common_in_memory_cache::InMemoryCache;
use common_persistent_cache::PersistentCache;
use domain_schedule_models::dto::v1::{Schedule, ScheduleType};
use reqwest::{redirect::Policy, Client, ClientBuilder};
use tokio::sync::Mutex;

use crate::dto::mpei::MpeiClasses;

use super::{
    mapping::map_schedule_models,
    mediator::{CacheMediator, InMemoryCacheKey},
};

pub struct ScheduleRepository {
    client: Client,
    in_memory_cache: Mutex<InMemoryCache<InMemoryCacheKey, Schedule>>,
    persistent_cache: Mutex<PersistentCache>,
}

impl Default for ScheduleRepository {
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
            in_memory_cache: Mutex::new(
                InMemoryCache::with_capacity(3000)
                    .expires_after_creation(chrono::Duration::hours(6)),
            ),
            persistent_cache: Mutex::new(PersistentCache::new("./cache".into())),
        }
    }
}

impl ScheduleRepository {
    pub async fn get_schedule_from_cache(
        &self,
        name: String,
        r#type: ScheduleType,
        week_start: NaiveDate,
        ignore_expiration: bool,
    ) -> anyhow::Result<Option<Schedule>> {
        let mut mediator = CacheMediator::new(&self.in_memory_cache, &self.persistent_cache);
        let key = InMemoryCacheKey {
            name: name.to_owned(),
            r#type: r#type.to_mpei(),
            week_start,
        };

        mediator.get(&key, ignore_expiration).await
    }

    pub async fn insert_schedule_to_cache(
        &self,
        name: String,
        r#type: ScheduleType,
        week_start: NaiveDate,
        schedule: Schedule,
    ) -> anyhow::Result<()> {
        let mut mediator = CacheMediator::new(&self.in_memory_cache, &self.persistent_cache);
        let key = InMemoryCacheKey {
            name: name.to_owned(),
            r#type: r#type.to_mpei(),
            week_start,
        };

        mediator.insert(key, schedule).await
    }

    pub async fn get_schedule_from_remote(
        &self,
        schedule_id: i64,
        name: String,
        r#type: ScheduleType,
        week_start: NaiveDate,
    ) -> anyhow::Result<Schedule> {
        let week_end = week_start
            .checked_add_days(Days::new(6))
            .expect("Week end date always reachable");

        let schedule_response = self
            .client
            .get(format!(
                "http://ts.mpei.ru/api/schedule/{0}/{1}",
                r#type.to_mpei(),
                schedule_id
            ))
            .query(&[
                ("start", &week_start.format("%Y.%m.%d").to_string()),
                ("finish", &week_end.format("%Y.%m.%d").to_string()),
            ])
            .send()
            .await
            .map_err(|e| anyhow!(CommonError::gateway(e)))
            .with_context(|| "Error while executing a request to MPEI backend")?
            .json::<Vec<MpeiClasses>>()
            .await
            .map_err(|e| anyhow!(CommonError::internal(e)))
            .with_context(|| "Error while deserializing response from MPEI backend")?;

        map_schedule_models(name, week_start, schedule_id, r#type, schedule_response)
    }
}
