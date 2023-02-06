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
        get_schedule_use_case: Arc<ScheduleRepository>,
        search_schedule_use_case: Arc<ScheduleSearchRepository>,
    ) -> Self {
        Self(
            text_to_action_use_case,
            peer_repository,
            get_schedule_use_case,
            search_schedule_use_case,
        )
    }
}
