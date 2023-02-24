use std::sync::Arc;

use anyhow::{anyhow, ensure, Context};
use chrono::{Local, NaiveDate, Weekday};
use common_errors::errors::{CommonError, CommonErrorExt};
use domain_schedule_cooldown::ScheduleCooldownRepository;
use domain_schedule_models::{Schedule, ScheduleSearchResult, ScheduleType};
use lazy_static::lazy_static;
use log::{debug, info, warn};
use tokio::sync::Mutex;

use crate::{
    dto::mpeix::{ScheduleName, ScheduleSearchQuery},
    id::repository::ScheduleIdRepository,
    schedule::repository::ScheduleRepository,
    schedule_shift::repository::ScheduleShiftRepository,
    search::repository::ScheduleSearchRepository,
    time::{DateTimeExt, NaiveDateExt, WeekOfSemester},
};

/// Get numeric `ID` of schedule by its `name` and `type`.
/// - `type` is enum of `Group`, `Person`, `Room`.
///
/// This UseCase uses injected singleton instance of [ScheduleIdRepository].
/// Check [crate::di] module for details.
pub struct GetScheduleIdUseCase(pub(crate) Arc<ScheduleIdRepository>);

impl GetScheduleIdUseCase {
    /// Get numeric `ID` of schedule by its `name` and `type`.
    /// See [GetScheduleIdUseCase] description.
    pub async fn get_id(&self, name: String, r#type: ScheduleType) -> anyhow::Result<i64> {
        let name = ScheduleName::new(name, r#type.clone())?;
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
/// This UseCase uses injected singleton instances of [ScheduleIdRepository],
/// [ScheduleRepository] and [ScheduleShiftRepository].
/// Check [crate::di] module for details.
pub struct GetScheduleUseCase {
    pub(crate) schedule_id_repository: Arc<ScheduleIdRepository>,
    pub(crate) schedule_repository: Arc<ScheduleRepository>,
    pub(crate) schedule_shift_repository: Arc<ScheduleShiftRepository>,
    pub(crate) schedule_cooldown_repository: Arc<ScheduleCooldownRepository>,
    pub(crate) _lock: Mutex<()>,
}

impl GetScheduleUseCase {
    /// Get [Schedule] model by schedule `name`, `type`, and `offset`.
    /// See [GetScheduleUseCase] descrition.
    pub async fn get_schedule(
        &self,
        name: String,
        r#type: ScheduleType,
        offset: i32,
    ) -> anyhow::Result<Schedule> {
        debug!("GetScheduleUseCase(name='{name}', type='{type}', offset={offset})");
        ensure!(offset < *MAX_OFFSET, CommonError::user("Too large offset"));
        ensure!(offset > *MIN_OFFSET, CommonError::user("Too small offset"));

        let name = ScheduleName::new(name, r#type.clone())?;
        let week_start = Local::now()
            .with_days_offset(offset * 7)
            .map(|dt| dt.date_naive())
            .map(|dt| dt.week(Weekday::Mon).first_day())
            .ok_or_else(|| anyhow!(CommonError::user("Invalid week offset")))?;
        let week_of_semester = self
            .schedule_shift_repository
            .get_week_of_semester(&week_start)
            .await?;
        // Always ignore expiration policy for past weeks
        // and also in case of active "cooldown"
        let ignore_expiration = week_start.is_past_week()
            || self.schedule_cooldown_repository.is_cooldown_active().await;

        // try to get schedule from cache first
        if let Some(schedule) = self
            .get_schedule_from_cache(
                &name,
                &r#type,
                week_start,
                &week_of_semester,
                ignore_expiration,
            )
            .await?
        {
            return Ok(schedule);
        }

        // Trying to get schedule id from remote, do not return error in case of error
        // remember error to process it in next steps
        let remote = self
            .get_schedule_from_remote(&name, &r#type, week_start, &week_of_semester)
            .await;

        if let Err(e) = &remote {
            warn!("{e}"); // full error description is in anyhow context
            if let Some(CommonError::GatewayError(_)) = e.as_common_error() {
                warn!("Cooldown will be activated...");
                self.schedule_cooldown_repository.activate().await;
            }
        }

        // if we cannot get value from remote and didn't disable expiration policy at the beginning,
        // try to disable expiration policy and look for cached value again
        if remote.is_err() && !ignore_expiration {
            if let Some(schedule) = self
                .get_schedule_from_cache(&name, &r#type, week_start, &week_of_semester, true)
                .await?
            {
                return Ok(schedule);
            }
        }

        // If we successfully got new value from remote,
        // put it into the cache
        if remote.is_ok() {
            // put new remote value into the cache
            self.schedule_repository
                .insert_schedule_to_cache(
                    name,
                    r#type,
                    week_start,
                    remote.as_ref().unwrap().to_owned(),
                )
                .await?;
            debug!("Got schedule from remote");
        }

        // if we have not even expired cached value, return error about remote request
        remote
    }

    async fn get_schedule_from_remote(
        &self,
        name: &ScheduleName,
        r#type: &ScheduleType,
        week_start: NaiveDate,
        week_of_semester: &WeekOfSemester,
    ) -> anyhow::Result<Schedule> {
        // Trying to get schedule id from remote
        let schedule_id = self
            .schedule_id_repository
            .get_id(name.to_owned(), r#type.to_owned())
            .await
            .with_context(|| "Error while getting schedule id from remote")?;

        // get schedule from remote by its id, if previous step was successful
        self.schedule_repository
            .get_schedule_from_remote(
                schedule_id,
                name.to_owned(),
                r#type.to_owned(),
                week_start,
                week_of_semester.to_owned(),
            )
            .await
            .with_context(|| "Error while getting schedule from remote")
    }

    async fn get_schedule_from_cache(
        &self,
        name: &ScheduleName,
        r#type: &ScheduleType,
        week_start: NaiveDate,
        week_of_semester: &WeekOfSemester,
        ignore_expiration: bool,
    ) -> anyhow::Result<Option<Schedule>> {
        if let Some(mut schedule) = self
            .schedule_repository
            .get_schedule_from_cache(
                name.to_owned(),
                r#type.to_owned(),
                week_start,
                ignore_expiration,
            )
            .await?
        {
            debug!("Got schedule from cache (ignore_expiration={ignore_expiration})");
            {
                // fix schedule week_of_semester according to new schedule shift rules
                self.fix_schedule_shift_if_needed(
                    &mut schedule,
                    &week_of_semester,
                    name.to_owned(),
                )
                .await
                .with_context(|| "Error while fixing schedule shift")?;
            }
            return Ok(Some(schedule));
        }
        Ok(None)
    }

    async fn fix_schedule_shift_if_needed(
        &self,
        schedule: &mut Schedule,
        week_of_semester: &WeekOfSemester,
        name: ScheduleName,
    ) -> anyhow::Result<()> {
        debug!("Checking if the schedule needs to be corrected shift...");
        let mut week = schedule
            .weeks
            .first_mut()
            .ok_or_else(|| anyhow!("Encountered invalid schedule with empty weeks field"))?;

        let true_week_of_semester = match week_of_semester {
            WeekOfSemester::Studying(week_of_semester) => *week_of_semester as i8,
            WeekOfSemester::NonStudying => -1,
        };

        if true_week_of_semester != week.week_of_semester {
            week.week_of_semester = true_week_of_semester;
            self.schedule_repository
                .insert_schedule_to_cache(
                    name,
                    schedule.r#type.clone(),
                    week.first_day_of_week,
                    schedule.clone(),
                )
                .await?;
            info!("Schedule 'week_of_semester' field was fixed to {true_week_of_semester}");
        }
        Ok(())
    }
}

/// Get [Vec] of [ScheduleSearchResult].
///
/// This use-case is similar to [GetScheduleIdUseCase], but differs from it in that
/// it does not return the ID of the first search result, but returns all search results.
///
/// Due to the fact that a database is connected to this use case,
/// users can do search even when the MPEI website is unavailable.
///
/// Algorithm of the use case:
/// Look for in-memory cached result for given type and query, and return cached result if exists.
/// In this case no requests are made to the database or the MPEI backend. If cache do not have
/// necessary value, we made request to the MPEI backend and put results of the request to
/// the database (add new entries, update old entries). Even if the request fails, we do search
/// in the database and return best mathes.
pub struct SearchScheduleUseCase(pub(crate) Arc<ScheduleSearchRepository>);

impl SearchScheduleUseCase {
    pub async fn search(
        &self,
        query: String,
        r#type: Option<ScheduleType>,
    ) -> anyhow::Result<Vec<ScheduleSearchResult>> {
        let query = ScheduleSearchQuery::new(query)?;
        if let Some(cached_value) = self
            .0
            .get_results_from_cache(query.to_owned(), r#type.to_owned())
            .await
        {
            debug!("Got schedule search result from cache");
            return Ok(cached_value);
        }

        let remote_results = if let Some(r#type) = &r#type {
            self.0.get_results_from_remote(&query, r#type).await
        } else {
            let mut output = Vec::<ScheduleSearchResult>::new();
            let mut groups = self
                .0
                .get_results_from_remote(&query, &ScheduleType::Group)
                .await;
            let mut persons = self
                .0
                .get_results_from_remote(&query, &ScheduleType::Person)
                .await;
            if let Ok(groups) = &mut groups {
                output.append(groups);
            }
            if let Ok(persons) = &mut persons {
                output.append(persons);
            }
            Ok(output)
        };
        if let Ok(results) = remote_results {
            if !results.is_empty() {
                self.0.insert_results_to_db(results).await?;
            }
        }

        let mut db_results = self
            .0
            .get_results_from_db(&query, r#type.to_owned())
            .await?;

        let max_idx = db_results.len();
        db_results.sort_by(|a, b| {
            let idx_a = a.name.to_lowercase().find(query.as_ref()).or(Some(max_idx));
            let idx_b = b.name.to_lowercase().find(query.as_ref()).or(Some(max_idx));
            idx_a.cmp(&idx_b)
        });

        self.0
            .insert_results_to_cache(query, r#type, db_results.clone())
            .await;

        Ok(db_results)
    }
}

/// Create databases if needed and run migrations.
/// This use case must be started **STRICTLY** before the server starts.
pub struct InitDomainScheduleUseCase(pub(crate) Arc<ScheduleSearchRepository>);

impl InitDomainScheduleUseCase {
    pub async fn init(&self) -> anyhow::Result<()> {
        self.0
            .init_schedule_search_results_db()
            .await
            .with_context(|| "Database initialization error")
    }
}
