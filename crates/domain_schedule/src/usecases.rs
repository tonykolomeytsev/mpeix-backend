use std::sync::Arc;

use anyhow::{anyhow, ensure, Context};
use chrono::{Local, Weekday};
use common_errors::errors::CommonError;
use domain_schedule_models::dto::v1::{Schedule, ScheduleSearchResult, ScheduleType};
use lazy_static::lazy_static;
use log::info;

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
pub struct GetScheduleUseCase(
    pub(crate) Arc<ScheduleIdRepository>,
    pub(crate) Arc<ScheduleRepository>,
    pub(crate) Arc<ScheduleShiftRepository>,
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

        let name = ScheduleName::new(name, r#type.clone())?;
        let week_start = Local::now()
            .with_days_offset(offset * 7)
            .map(|dt| dt.date_naive())
            .map(|dt| dt.week(Weekday::Mon).first_day())
            .ok_or_else(|| anyhow!(CommonError::user("Invalid week offset")))?;
        let week_of_semester = self.2.get_week_of_semester(&week_start).await?;
        let ignore_expiration = week_start.is_past_week();

        if let Some(mut schedule) = self
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
            return Ok(schedule);
        }

        let schedule_id = self.0.get_id(name.to_owned(), r#type.to_owned()).await?;

        let remote = self
            .1
            .get_schedule_from_remote(
                schedule_id,
                name.to_owned(),
                r#type.to_owned(),
                week_start,
                week_of_semester.to_owned(),
            )
            .await;

        // if we cannot get value from remote and didn't disable expiration policy at the beginning,
        // try to disable expiration policy and look for cached value again
        if remote.is_err() && !ignore_expiration {
            if let Some(mut schedule) = self
                .1
                .get_schedule_from_cache(name.to_owned(), r#type.to_owned(), week_start, true)
                .await?
            {
                info!("Got expired schedule from cache (remote unavailable)");
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

    async fn fix_schedule_shift_if_needed(
        &self,
        schedule: &mut Schedule,
        week_of_semester: &WeekOfSemester,
        name: ScheduleName,
    ) -> anyhow::Result<()> {
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
            self.1
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
            info!("Got schedule search result from cache");
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
