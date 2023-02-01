use anyhow::anyhow;
use chrono::{Duration, NaiveDate};
use common_in_memory_cache::InMemoryCache;
use domain_schedule_shift::ScheduleShift;
use tokio::sync::Mutex;

use crate::time::{NaiveDateExt, WeekOfSemester};

pub struct ScheduleShiftRepository {
    cache: Mutex<InMemoryCache<(), ScheduleShift>>,
    config_path: String,
}

impl Default for ScheduleShiftRepository {
    fn default() -> Self {
        let config_path = envmnt::get_or("SCHEDULE_SHIFT_CONFIG_PATH", "./schedule_shift.toml");
        Self {
            cache: Mutex::new(
                InMemoryCache::with_capacity(1).expires_after_creation(Duration::minutes(1)),
            ),
            config_path,
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
            cache.insert((), ScheduleShift::from_file(&self.config_path).await?);
        }

        week_start
            .week_of_semester(cache.get(&()))
            .ok_or_else(|| anyhow!("Cannot calculate week of semester for '{week_start}'"))
    }
}
