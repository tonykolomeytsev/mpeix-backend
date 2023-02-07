use std::sync::Arc;

use common_database::create_db_pool;
use domain_bot::{
    peer::repository::PeerRepository,
    schedule::repository::ScheduleRepository,
    search::repository::ScheduleSearchRepository,
    usecases::{GetUpcomingEventsUseCase, InitDomainBotUseCase, ReplyUseCase, TextToActionUseCase},
};
use domain_vk_bot::usecases::ReplyToVkUseCase;
use feature_vk_bot::FeatureVkBot;

use crate::AppVkBot;

pub fn create_app() -> AppVkBot {
    let db_pool = Arc::new(create_db_pool().expect("DI error while creating db pool"));

    let peer_repository = Arc::new(PeerRepository::new(db_pool));
    let schedule_repository = Arc::new(ScheduleRepository::default());
    let schedule_search_repository = Arc::new(ScheduleSearchRepository::default());

    let text_to_action_use_case = Arc::new(TextToActionUseCase::default());
    let get_upcoming_events_use_case =
        Arc::new(GetUpcomingEventsUseCase::new(schedule_repository.clone()));
    let reply_use_case = Arc::new(ReplyUseCase::new(
        text_to_action_use_case,
        peer_repository.clone(),
        schedule_repository,
        schedule_search_repository,
        get_upcoming_events_use_case,
    ));
    let reply_to_vk_use_case = Arc::new(ReplyToVkUseCase::default());

    AppVkBot {
        feature_vk_bot: FeatureVkBot::new(reply_use_case, reply_to_vk_use_case),
        init_domain_bot_use_case: InitDomainBotUseCase::new(peer_repository),
    }
}