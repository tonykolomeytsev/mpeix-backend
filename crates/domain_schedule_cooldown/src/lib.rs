use chrono::{DateTime, Duration, Local};
use common_rust::env;
pub use tokio::sync::Mutex;

/// Provides "cooldown" functionality in case of errors on MPEI server.
/// Counts the time during which it is impossible to reconnect to the server that returned the error.
/// During the "cooldown", the expiration policy of the schedule cache is ignored
/// and schedules are taken from the cache anyway.
pub struct ScheduleCooldownRepository {
    cooldown_duration: Duration,
    last_error_time: Mutex<Option<DateTime<Local>>>,
}

impl Default for ScheduleCooldownRepository {
    fn default() -> Self {
        let cooldown_duration = env::get_parsed_or("SCHEDULE_COOLDOWN_DURATION_MIN", 1);

        Self {
            cooldown_duration: Duration::minutes(cooldown_duration),
            last_error_time: Mutex::new(None),
        }
    }
}

impl ScheduleCooldownRepository {
    /// Set cooldown timer active
    pub async fn activate(&self) {
        *self.last_error_time.lock().await = Some(Local::now())
    }

    /// Check if cooldown timer still active or not
    pub async fn is_cooldown_active(&self) -> bool {
        let last_error_time = self.last_error_time.lock().await;
        last_error_time.is_some() && !self.is_expired(&*last_error_time, &self.cooldown_duration)
    }

    /// Taken from `commin_in_memory_cache`
    fn is_expired(&self, start: &Option<DateTime<Local>>, duration: &Duration) -> bool {
        start
            .and_then(|s| s.checked_add_signed(*duration))
            .filter(|&e| e <= Local::now())
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::Mutex;

    use chrono::{Duration, Local};

    use super::ScheduleCooldownRepository;

    #[test]
    fn test_activate_and_check_active() {
        let mut repo = ScheduleCooldownRepository::default();
        repo.cooldown_duration = Duration::minutes(1);
        tokio_test::block_on(repo.activate());
        assert_eq!(true, tokio_test::block_on(repo.is_cooldown_active()));
    }

    #[test]
    fn test_cooldown_is_inactive_without_activating() {
        let mut repo = ScheduleCooldownRepository::default();
        repo.cooldown_duration = Duration::minutes(1);
        assert_eq!(false, tokio_test::block_on(repo.is_cooldown_active()));
    }

    #[test]
    fn test_activate_and_check_inactive() {
        let mut repo = ScheduleCooldownRepository::default();
        repo.cooldown_duration = Duration::minutes(1);
        // kinda activate now
        repo.last_error_time = Mutex::new(Some(Local::now()));
        assert_eq!(true, tokio_test::block_on(repo.is_cooldown_active()));

        // kinda activate half a minute ago
        repo.last_error_time = Mutex::new(Some(
            Local::now()
                .checked_sub_signed(Duration::seconds(30))
                .unwrap(),
        ));
        assert_eq!(true, tokio_test::block_on(repo.is_cooldown_active()));

        // kinda activate minute ago
        repo.last_error_time = Mutex::new(Some(
            Local::now()
                .checked_sub_signed(Duration::minutes(1))
                .unwrap(),
        ));
        assert_eq!(false, tokio_test::block_on(repo.is_cooldown_active()))
    }
}
