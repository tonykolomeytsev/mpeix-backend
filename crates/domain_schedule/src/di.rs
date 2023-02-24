use std::sync::Arc;

use domain_schedule_cooldown::ScheduleCooldownRepository;

use crate::{
    id::repository::ScheduleIdRepository,
    schedule::repository::ScheduleRepository,
    schedule_shift::repository::ScheduleShiftRepository,
    search::repository::ScheduleSearchRepository,
    usecases::{
        GetScheduleIdUseCase, GetScheduleUseCase, InitDomainScheduleUseCase, SearchScheduleUseCase,
    },
};

impl GetScheduleIdUseCase {
    pub fn new(schedule_id_repository: Arc<ScheduleIdRepository>) -> Self {
        Self(schedule_id_repository)
    }
}

impl GetScheduleUseCase {
    pub fn new(
        schedule_id_repository: Arc<ScheduleIdRepository>,
        schedule_repository: Arc<ScheduleRepository>,
        schedule_shift_repository: Arc<ScheduleShiftRepository>,
        schedule_cooldown_repository: Arc<ScheduleCooldownRepository>,
    ) -> Self {
        Self {
            schedule_id_repository,
            schedule_repository,
            schedule_shift_repository,
            schedule_cooldown_repository,
        }
    }
}

impl SearchScheduleUseCase {
    pub fn new(schedule_search_repository: Arc<ScheduleSearchRepository>) -> Self {
        Self(schedule_search_repository)
    }
}

impl InitDomainScheduleUseCase {
    pub fn new(schedule_search_repository: Arc<ScheduleSearchRepository>) -> Self {
        Self(schedule_search_repository)
    }
}
