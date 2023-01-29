use std::sync::Arc;

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
        let mut config = deadpool_postgres::Config::new();
        config.dbname = Some("mpeix".to_string());
        config.host = Some("127.0.0.1".to_string());
        config.port = Some(5432);
        config.user = Some("postgres".to_string());
        config.password = Some("1234".to_string());

        let pool = Arc::new(
            config
                .create_pool(None, tokio_postgres::NoTls)
                .expect("Error during Postgres pool creation"),
        );

        let schedule_id_repository = Arc::new(ScheduleIdRepository::default());
        let schedule_repository = Arc::new(ScheduleRepository::default());
        let schedule_search_repository = Arc::new(ScheduleSearchRepository::new(pool));

        Self {
            get_schedule_id_use_case: GetScheduleIdUseCase(schedule_id_repository.clone()),
            get_schedule_use_case: GetScheduleUseCase(schedule_id_repository, schedule_repository),
            search_schedule_use_case: SearchScheduleUseCase(schedule_search_repository.clone()),
            init_domain_schedule_use_case: InitDomainScheduleUseCase(schedule_search_repository),
        }
    }
}
