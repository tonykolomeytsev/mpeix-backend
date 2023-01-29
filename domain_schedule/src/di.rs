use std::sync::Arc;

use common_database::create_db_pool;

use crate::{
    id::repository::ScheduleIdRepository,
    schedule::repository::ScheduleRepository,
    search::repository::ScheduleSearchRepository,
    usecases::{
        GetScheduleIdUseCase, GetScheduleUseCase, InitDomainScheduleUseCase, SearchScheduleUseCase,
    },
};

pub struct DomainScheduleModule {
    pub get_schedule_id_use_case: GetScheduleIdUseCase,
    pub get_schedule_use_case: GetScheduleUseCase,
    pub search_schedule_use_case: SearchScheduleUseCase,
    pub init_domain_schedule_use_case: InitDomainScheduleUseCase,
}

impl Default for DomainScheduleModule {
    fn default() -> Self {
        let db_pool = Arc::new(create_db_pool().expect("Error while creating db pool"));
        let schedule_id_repository = Arc::new(ScheduleIdRepository::default());
        let schedule_repository = Arc::new(ScheduleRepository::default());
        let schedule_search_repository = Arc::new(ScheduleSearchRepository::new(db_pool));

        Self {
            get_schedule_id_use_case: GetScheduleIdUseCase(schedule_id_repository.clone()),
            get_schedule_use_case: GetScheduleUseCase(schedule_id_repository, schedule_repository),
            search_schedule_use_case: SearchScheduleUseCase(schedule_search_repository.clone()),
            init_domain_schedule_use_case: InitDomainScheduleUseCase(schedule_search_repository),
        }
    }
}
