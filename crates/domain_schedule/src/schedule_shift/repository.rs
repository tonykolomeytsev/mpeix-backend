use std::{path::PathBuf, str::FromStr};

use anyhow::{anyhow, Context};
use chrono::{Duration, NaiveDate};
use common_in_memory_cache::InMemoryCache;
use common_rust::env;
use domain_schedule_shift::ScheduleShift;
use tokio::sync::Mutex;

use crate::time::{NaiveDateExt, WeekOfSemester};

pub struct ScheduleShiftRepository {
    cache: Mutex<InMemoryCache<(), ScheduleShift>>,
    config_path: PathBuf,
}

impl Default for ScheduleShiftRepository {
    fn default() -> Self {
        let config_path = env::get_or("SCHEDULE_SHIFT_CONFIG_PATH", "./schedule_shift.toml");
        Self {
            cache: Mutex::new(
                InMemoryCache::with_capacity(1).expires_after_creation(Duration::minutes(1)),
            ),
            config_path: config_path.into(),
        }
    }
}

impl ScheduleShiftRepository {
    pub async fn get_week_of_semester(
        &self,
        week_start: &NaiveDate,
    ) -> anyhow::Result<WeekOfSemester> {
        let mut cache = self.cache.lock().await;
        if cache.get(&()).is_none() {
            if self.config_path.exists() {
                cache.insert(
                    (),
                    ScheduleShift::from_file(&self.config_path)
                        .await
                        .with_context(|| "Cannot access shift config file")?,
                );
            } else {
                cache.insert(
                    (),
                    ScheduleShift::from_str(include_str!(
                        "../../../domain_schedule_shift/res/default_schedule_shift.toml"
                    ))?,
                );
            }
        }

        week_start
            .week_of_semester(cache.get(&()))
            .ok_or_else(|| anyhow!("Cannot calculate week of semester for '{week_start}'"))
    }
}
