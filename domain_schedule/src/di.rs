use std::sync::Arc;

use crate::{
    id::repository::ScheduleIdRepository,
    schedule::repository::ScheduleRepository,
    search::repository::ScheduleSearchRepository,
    usecases::{GetScheduleIdUseCase, GetScheduleUseCase, SearchScheduleUseCase},
};

pub struct DomainScheduleModule {
    pub get_schedule_id_use_case: GetScheduleIdUseCase,
    pub get_schedule_use_case: GetScheduleUseCase,
    pub search_schedule_use_case: SearchScheduleUseCase,
}

impl Default for DomainScheduleModule {
    fn default() -> Self {
        let schedule_id_repository = Arc::new(ScheduleIdRepository::default());
        let schedule_repository = Arc::new(ScheduleRepository::default());
        let schedule_search_repository = ScheduleSearchRepository::default();

        Self {
            get_schedule_id_use_case: GetScheduleIdUseCase(schedule_id_repository.clone()),
            get_schedule_use_case: GetScheduleUseCase(schedule_id_repository, schedule_repository),
            search_schedule_use_case: SearchScheduleUseCase(schedule_search_repository),
        }
    }
}
