use crate::{dto::mpei::MpeiClasses, sources, time::NaiveDateExt};
use anyhow::{anyhow, Context, Ok};
use chrono::{Days, NaiveDate};
use common_errors::errors::CommonError;
use common_in_memory_cache::InMemoryCache;
use common_persistent_cache::PersistentCache;
use domain_schedule_models::dto::v1::{self, ScheduleType};
use log::info;
use reqwest::{redirect::Policy, Client, ClientBuilder};
use tokio::sync::Mutex;

use super::{
    mapping::map_schedule_models,
    mediator::{CacheMediator, InMemoryCacheKey},
};

pub struct State {
    client: Client,
    in_memory_cache: Mutex<InMemoryCache<InMemoryCacheKey, v1::Schedule>>,
    persistent_cache: Mutex<PersistentCache>,
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
            in_memory_cache: Mutex::new(
                InMemoryCache::with_capacity(3000)
                    .expires_after_creation(chrono::Duration::hours(6)),
            ),
            persistent_cache: Mutex::new(PersistentCache::new("./cache".into())),
        }
    }
}

/// Get schedule from in-memory cache if value present in cache,
/// or get schedule from remote (`ts.mpei.ru`).
pub async fn get_schedule(
    name: String,
    r#type: ScheduleType,
    week_start: NaiveDate,
    schedule_source_state: &State,
    id_source_state: &sources::id::State,
) -> anyhow::Result<v1::Schedule> {
    let mut mediator = CacheMediator::new(
        &schedule_source_state.in_memory_cache,
        &schedule_source_state.persistent_cache,
    );
    let key = InMemoryCacheKey {
        name: name.to_owned(),
        r#type: r#type.to_mpei(),
        week_start,
    };
    let ignore_expiration = week_start.is_past_week();

    if let Some(schedule) = mediator.get(&key, ignore_expiration).await? {
        // if we have not expired value in cache, return this value
        info!("Got schedule from cache");
        return Ok(schedule);
    }

    // or try to get value from remote
    let remote = get_schedule_from_remote(
        name,
        r#type,
        week_start,
        schedule_source_state,
        id_source_state,
    )
    .await;

    // if we cannot get value from remote and didn't disable expiration policy at the beginning,
    // try to disable expiration policy and look for cached value again
    if remote.is_err() && !ignore_expiration {
        if let Some(schedule) = mediator.get(&key, true).await? {
            info!("Got expired schedule from cache (remote unavailable)");
            return Ok(schedule);
        }
    }

    if remote.is_ok() {
        // put new remote value into the cache
        mediator
            .insert(key, remote.as_ref().unwrap().to_owned())
            .await?;
    }

    // if we have not even expired cached value, return error about remote request
    remote
}

async fn get_schedule_from_remote(
    name: String,
    r#type: ScheduleType,
    week_start: NaiveDate,
    schedule_source_state: &State,
    id_source_state: &sources::id::State,
) -> anyhow::Result<v1::Schedule> {
    let schedule_id = sources::id::get_id(&name, r#type.to_owned(), id_source_state)
        .await
        .with_context(|| "Error while using id_source from schedule_source")?;
    let week_end = week_start
        .checked_add_days(Days::new(6))
        .expect("Week end date always reachable");
    let schedule_response = schedule_source_state
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
