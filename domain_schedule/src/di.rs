use std::sync::Arc;

use crate::{
    id::repository::ScheduleIdRepository,
    schedule::repository::ScheduleRepository,
    usecases::{GetScheduleIdUseCase, GetScheduleUseCase},
};

pub struct DomainScheduleModule {
    pub get_schedule_id_use_case: GetScheduleIdUseCase,
    pub get_schedule_use_case: GetScheduleUseCase,
}

impl Default for DomainScheduleModule {
    fn default() -> Self {
        let schedule_id_repository = Arc::new(ScheduleIdRepository::default());
        let schedule_repository = Arc::new(ScheduleRepository::default());
        Self {
            get_schedule_id_use_case: GetScheduleIdUseCase(schedule_id_repository.clone()),
            get_schedule_use_case: GetScheduleUseCase(schedule_id_repository, schedule_repository),
        }
    }
}
