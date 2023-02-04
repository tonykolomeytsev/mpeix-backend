use std::sync::Arc;

use common_database::create_db_pool;
use domain_bot::{
    peer::repository::PeerRepository,
    usecases::{InitDomainBotUseCase, ReplyUseCase},
};
use domain_schedule::{
    id::repository::ScheduleIdRepository,
    schedule::repository::ScheduleRepository,
    schedule_shift::repository::ScheduleShiftRepository,
    search::repository::ScheduleSearchRepository,
    usecases::{
        GetScheduleIdUseCase, GetScheduleUseCase, InitDomainScheduleUseCase, SearchScheduleUseCase,
    },
};
use feature_schedule::v1::FeatureSchedule;
use feature_telegram_bot::FeatureTelegramBot;
use feature_vk_bot::FeatureVkBot;

use crate::AppSchedule;

pub struct AppComponent;

impl AppComponent {
    pub fn create_app() -> AppSchedule {
        let db_pool = Arc::new(create_db_pool().expect("DI error while creating db pool"));

        // Repositories
        let schedule_id_repository = Arc::new(ScheduleIdRepository::default());
        let schedule_repository = Arc::new(ScheduleRepository::default());
        let schedule_shift_repository = Arc::new(ScheduleShiftRepository::default());
        let schedule_search_repository = Arc::new(ScheduleSearchRepository::new(db_pool.clone()));
        let peer_repository = Arc::new(PeerRepository::new(db_pool));

        // Use-cases
        let get_schedule_id_use_case =
            Arc::new(GetScheduleIdUseCase::new(schedule_id_repository.clone()));
        let get_schedule_use_case = Arc::new(GetScheduleUseCase::new(
            schedule_id_repository,
            schedule_repository.clone(),
            schedule_shift_repository,
        ));
        let search_schedule_use_case = Arc::new(SearchScheduleUseCase::new(
            schedule_search_repository.clone(),
        ));
        let reply_use_case = Arc::new(ReplyUseCase::new(
            peer_repository.clone(),
            schedule_repository,
            schedule_search_repository.clone(),
        ));
        let init_domain_schedule_use_case =
            InitDomainScheduleUseCase::new(schedule_search_repository);
        let init_domain_bot_use_case = InitDomainBotUseCase::new(peer_repository);

        AppSchedule {
            feature_schedule: FeatureSchedule::new(
                get_schedule_id_use_case,
                get_schedule_use_case,
                search_schedule_use_case,
            ),
            feature_telegram_bot: FeatureTelegramBot::new(reply_use_case.clone()),
            feature_vk_bot: FeatureVkBot::new(reply_use_case),
            init_domain_schedule_use_case,
            init_domain_bot_use_case,
        }
    }
}
