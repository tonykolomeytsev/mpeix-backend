use std::sync::Arc;

use common_database::create_db_pool;
use common_restix::create_reqwest_client;
use common_rust::env;
use domain_bot::{
    mpeix_api::MpeixApi,
    peer::repository::PeerRepository,
    schedule::repository::ScheduleRepository,
    search::repository::ScheduleSearchRepository,
    usecases::{
        GenerateReplyUseCase, GetUpcomingEventsUseCase, InitDomainBotUseCase, TextToActionUseCase,
    },
};
use domain_vk_bot::usecases::ReplyToVkUseCase;
use feature_vk_bot::FeatureVkBot;

use crate::AppVkBot;

pub fn create_app() -> AppVkBot {
    let db_pool = Arc::new(create_db_pool().expect("DI error while creating db pool"));
    let api = MpeixApi::builder()
        .base_url(env::required("APP_SCHEDULE_BASE_URL"))
        .client(create_reqwest_client())
        .build()
        .expect("DI error while creating MpeixApi");

    let peer_repository = Arc::new(PeerRepository::new(db_pool));
    let schedule_repository = Arc::new(ScheduleRepository::new(api.to_owned()));
    let schedule_search_repository = Arc::new(ScheduleSearchRepository::new(api));

    let text_to_action_use_case = Arc::new(TextToActionUseCase);
    let get_upcoming_events_use_case =
        Arc::new(GetUpcomingEventsUseCase::new(schedule_repository.clone()));
    let generate_reply_use_case = Arc::new(GenerateReplyUseCase::new(
        text_to_action_use_case,
        peer_repository.clone(),
        schedule_repository,
        schedule_search_repository,
        get_upcoming_events_use_case,
    ));
    let reply_to_vk_use_case = Arc::new(ReplyToVkUseCase::default());

    AppVkBot {
        feature_vk_bot: FeatureVkBot::new(generate_reply_use_case, reply_to_vk_use_case),
        init_domain_bot_use_case: InitDomainBotUseCase::new(peer_repository),
    }
}
