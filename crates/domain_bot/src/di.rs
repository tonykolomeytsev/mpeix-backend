use std::sync::Arc;

use crate::{
    peer::repository::PeerRepository,
    schedule::repository::ScheduleRepository,
    search::repository::ScheduleSearchRepository,
    usecases::{InitDomainBotUseCase, ReplyUseCase, TextToActionUseCase},
};

impl InitDomainBotUseCase {
    pub fn new(peer_repository: Arc<PeerRepository>) -> Self {
        Self(peer_repository)
    }
}

impl ReplyUseCase {
    pub fn new(
        text_to_action_use_case: Arc<TextToActionUseCase>,
        peer_repository: Arc<PeerRepository>,
        schedule_repository: Arc<ScheduleRepository>,
        schedule_search_repository: Arc<ScheduleSearchRepository>,
    ) -> Self {
        Self(
            text_to_action_use_case,
            peer_repository,
            schedule_repository,
            schedule_search_repository,
        )
    }
}
