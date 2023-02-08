use std::sync::Arc;

use crate::{
    peer::repository::PeerRepository,
    schedule::repository::ScheduleRepository,
    search::repository::ScheduleSearchRepository,
    usecases::{
        GenerateReplyUseCase, GetUpcomingEventsUseCase, InitDomainBotUseCase, TextToActionUseCase,
    },
};

impl InitDomainBotUseCase {
    pub fn new(peer_repository: Arc<PeerRepository>) -> Self {
        Self(peer_repository)
    }
}

impl GetUpcomingEventsUseCase {
    pub fn new(schedule_repository: Arc<ScheduleRepository>) -> Self {
        Self(schedule_repository)
    }
}

impl GenerateReplyUseCase {
    pub fn new(
        text_to_action_use_case: Arc<TextToActionUseCase>,
        peer_repository: Arc<PeerRepository>,
        schedule_repository: Arc<ScheduleRepository>,
        schedule_search_repository: Arc<ScheduleSearchRepository>,
        get_upcoming_events_use_case: Arc<GetUpcomingEventsUseCase>,
    ) -> Self {
        Self(
            text_to_action_use_case,
            peer_repository,
            schedule_repository,
            schedule_search_repository,
            get_upcoming_events_use_case,
        )
    }
}
