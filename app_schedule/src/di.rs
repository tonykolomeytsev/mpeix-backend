use std::sync::Arc;

use common_database::create_db_pool;
use common_rust::env;
use domain_schedule::{
    id::repository::ScheduleIdRepository,
    schedule::repository::ScheduleRepository,
    schedule_shift::repository::ScheduleShiftRepository,
    search::repository::ScheduleSearchRepository,
    usecases::{
        GetScheduleIdUseCase, GetScheduleUseCase, InitDomainScheduleUseCase, SearchScheduleUseCase,
    },
};
use domain_schedule_cooldown::ScheduleCooldownRepository;
use feature_schedule::v1::FeatureSchedule;
use reqwest::{redirect::Policy, ClientBuilder};
use restix::HttpClient;

use crate::AppSchedule;

pub struct AppComponent;

impl AppComponent {
    pub fn create_app() -> AppSchedule {
        let db_pool = Arc::new(create_db_pool().expect("DI error while creating db pool"));
        let http_client = create_http_client();

        // Repositories
        let schedule_id_repository = Arc::new(ScheduleIdRepository::new(http_client.to_owned()));
        let schedule_repository = Arc::new(ScheduleRepository::new(http_client.to_owned()));
        let schedule_shift_repository = Arc::new(ScheduleShiftRepository::default());
        let schedule_search_repository =
            Arc::new(ScheduleSearchRepository::new(db_pool, http_client));

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

fn create_http_client() -> HttpClient {
    let connect_timeout = env::get_parsed_or("GATEWAY_CONNECT_TIMEOUT", 1500);

    HttpClient::new(
        ClientBuilder::new()
            .gzip(true)
            .deflate(true)
            .redirect(Policy::none())
            .timeout(std::time::Duration::from_secs(15))
            .connect_timeout(std::time::Duration::from_millis(connect_timeout))
            .pool_max_idle_per_host(3)
            .build()
            .expect("Something went wrong when building HttClient"),
    )
}
