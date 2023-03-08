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
use domain_telegram_bot::{
    telegram_api::TelegramApi,
    usecases::{DeleteMessageUseCase, ReplyToTelegramUseCase, SetWebhookUseCase},
};
use feature_telegram_bot::FeatureTelegramBot;

use crate::AppTelegramBot;

pub fn create_app() -> AppTelegramBot {
    let db_pool = Arc::new(create_db_pool().expect("DI error while creating db pool"));
    let api = MpeixApi::builder()
        .base_url(env::required("APP_SCHEDULE_BASE_URL"))
        .client(create_reqwest_client())
        .build()
        .expect("DI error while creating MpeixApi");

    let peer_repository = Arc::new(PeerRepository::new(db_pool));
    let schedule_repository = Arc::new(ScheduleRepository::new(api.to_owned()));
    let schedule_search_repository = Arc::new(ScheduleSearchRepository::new(api));

    let text_to_action_use_case = Arc::new(TextToActionUseCase::default());
    let get_upcoming_events_use_case =
        Arc::new(GetUpcomingEventsUseCase::new(schedule_repository.clone()));
    let generate_reply_use_case = Arc::new(GenerateReplyUseCase::new(
        text_to_action_use_case,
        peer_repository.clone(),
        schedule_repository,
        schedule_search_repository,
        get_upcoming_events_use_case,
    ));
    let telegram_api = Arc::new(TelegramApi::default());
    let set_webhook_use_case = Arc::new(SetWebhookUseCase::new(telegram_api.clone()));
    let reply_to_telegram_use_case = Arc::new(ReplyToTelegramUseCase::new(telegram_api.clone()));
    let delete_message_use_case = Arc::new(DeleteMessageUseCase::new(telegram_api));

    AppTelegramBot {
        feature_telegram_bot: FeatureTelegramBot::new(
            generate_reply_use_case,
            set_webhook_use_case,
            reply_to_telegram_use_case,
            delete_message_use_case,
        ),
        init_domain_bot_use_case: InitDomainBotUseCase::new(peer_repository),
    }
}
