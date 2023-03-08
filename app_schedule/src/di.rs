use std::sync::Arc;

use common_database::create_db_pool;
use common_restix::create_reqwest_client;
use domain_schedule::{
    id::repository::ScheduleIdRepository,
    mpei_api::MpeiApi,
    schedule::repository::ScheduleRepository,
    schedule_shift::repository::ScheduleShiftRepository,
    search::repository::ScheduleSearchRepository,
    usecases::{
        GetScheduleIdUseCase, GetScheduleUseCase, InitDomainScheduleUseCase, SearchScheduleUseCase,
    },
};
use domain_schedule_cooldown::ScheduleCooldownRepository;
use feature_schedule::v1::FeatureSchedule;

use crate::AppSchedule;

pub struct AppComponent;

impl AppComponent {
    pub fn create_app() -> AppSchedule {
        let db_pool = Arc::new(create_db_pool().expect("DI error while creating db pool"));
        let api = MpeiApi::builder()
            .client(create_reqwest_client())
            .build()
            .expect("DI error while creating MpeiApi");

        // Repositories
        let schedule_id_repository = Arc::new(ScheduleIdRepository::new(api.to_owned()));
        let schedule_repository = Arc::new(ScheduleRepository::new(api.to_owned()));
        let schedule_shift_repository = Arc::new(ScheduleShiftRepository::default());
        let schedule_search_repository = Arc::new(ScheduleSearchRepository::new(db_pool, api));

        // Use-cases
        let get_schedule_id_use_case =
            Arc::new(GetScheduleIdUseCase::new(schedule_id_repository.clone()));
        let get_schedule_use_case = Arc::new(GetScheduleUseCase::new(
            schedule_id_repository,
            schedule_repository,
            schedule_shift_repository,
            Arc::new(ScheduleCooldownRepository::default()),
        ));
        let search_schedule_use_case = Arc::new(SearchScheduleUseCase::new(
            schedule_search_repository.clone(),
            Arc::new(ScheduleCooldownRepository::default()),
        ));
        let init_domain_schedule_use_case =
            InitDomainScheduleUseCase::new(schedule_search_repository);

        AppSchedule {
            feature_schedule: FeatureSchedule::new(
                get_schedule_id_use_case,
                get_schedule_use_case,
                search_schedule_use_case,
            ),
            init_domain_schedule_use_case,
        }
    }
}
