use std::sync::Arc;

use anyhow::{anyhow, ensure};
use chrono::{Local, Weekday};
use common_errors::errors::CommonError;
use domain_schedule_models::dto::v1::{Schedule, ScheduleType};
use lazy_static::lazy_static;
use log::info;

use crate::{
    id::repository::ScheduleIdRepository,
    schedule::repository::ScheduleRepository,
    time::{DateTimeExt, NaiveDateExt},
};

/// Get numeric `ID` of schedule by its `name` and `type`.
/// - `type` is enum of `Group`, `Person`, `Room`.
///
/// This UseCase uses injected singleton instance of [ScheduleIdRepository].
/// Check [crate::di] module for details.
#[derive(Default)]
pub struct GetScheduleIdUseCase(pub(crate) Arc<ScheduleIdRepository>);

impl GetScheduleIdUseCase {
    /// Get numeric `ID` of schedule by its `name` and `type`.
    /// See [GetScheduleIdUseCase] description.
    pub async fn get_id(&self, name: String, r#type: ScheduleType) -> anyhow::Result<i64> {
        self.0.get_id(name, r#type).await
    }
}

lazy_static! {
    static ref MAX_OFFSET: i32 = i32::MAX / 7;
    static ref MIN_OFFSET: i32 = i32::MIN / 7;
}

/// Get [Schedule] model by schedule `name`, `type`, and `offset`.
/// - `type` is enum of `Group`, `Person`, `Room`.
/// - `offset` is the number from which the required week for the answer is calculated.
///     * If `offset` is `0`, the schedule for the current week will be returned.
///     * If offset is `1`, the next week will be returned.
///     * If offset is `-1`, the previous week will be returned.
///     * If offset is `-4`, the week that was 28 days ago will be returned.
///
/// This UseCase is maximally cache-friendly.
/// It returns even expired cache entries in cases when remote is unavailable.
///
/// This UseCase uses injected singleton instances of [ScheduleIdRepository] and [ScheduleRepository].
/// Check [crate::di] module for details.
#[derive(Default)]
pub struct GetScheduleUseCase(
    pub(crate) Arc<ScheduleIdRepository>,
    pub(crate) Arc<ScheduleRepository>,
);

impl GetScheduleUseCase {
    /// Get [Schedule] model by schedule `name`, `type`, and `offset`.
    /// See [GetScheduleUseCase] descrition.
    pub async fn get_schedule(
        &self,
        name: String,
        r#type: ScheduleType,
        offset: i32,
    ) -> anyhow::Result<Schedule> {
        ensure!(offset < *MAX_OFFSET, CommonError::user("Too large offset"));
        ensure!(offset > *MIN_OFFSET, CommonError::user("Too small offset"));

        let week_start = Local::now()
            .with_days_offset(offset * 7)
            .map(|dt| dt.date_naive())
            .map(|dt| dt.week(Weekday::Mon).first_day())
            .ok_or_else(|| anyhow!(CommonError::user("Invalid week offset")))?;
        let ignore_expiration = week_start.is_past_week();

        if let Some(schedule) = self
            .1
            .get_schedule_from_cache(
                name.to_owned(),
                r#type.to_owned(),
                week_start,
                ignore_expiration,
            )
            .await?
        {
            info!("Got schedule from cache");
            return Ok(schedule);
        }

        let schedule_id = self.0.get_id(name.to_owned(), r#type.to_owned()).await?;

        let remote = self
            .1
            .get_schedule_from_remote(schedule_id, name.to_owned(), r#type.to_owned(), week_start)
            .await;

        // if we cannot get value from remote and didn't disable expiration policy at the beginning,
        // try to disable expiration policy and look for cached value again
        if remote.is_err() && !ignore_expiration {
            if let Some(schedule) = self
                .1
                .get_schedule_from_cache(name.to_owned(), r#type.to_owned(), week_start, true)
                .await?
            {
                info!("Got expired schedule from cache (remote unavailable)");
                return Ok(schedule);
            }
        }

        if remote.is_ok() {
            // put new remote value into the cache
            self.1
                .insert_schedule_to_cache(
                    name,
                    r#type,
                    week_start,
                    remote.as_ref().unwrap().to_owned(),
                )
                .await?;
            info!("Got schedule from remote");
        }

        // if we have not even expired cached value, return error about remote request
        remote
    }
}
