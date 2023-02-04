use std::sync::Arc;

use domain_schedule::{
    schedule::repository::ScheduleRepository, search::repository::ScheduleSearchRepository,
};

use crate::{
    peer::repository::PeerRepository,
    usecases::{InitDomainBotUseCase, ReplyUseCase},
};

impl InitDomainBotUseCase {
    pub fn new(peer_repository: Arc<PeerRepository>) -> Self {
        Self(peer_repository)
    }
}

impl ReplyUseCase {
    pub fn new(
        peer_repository: Arc<PeerRepository>,
        schedule_repository: Arc<ScheduleRepository>,
        schedule_search_repository: Arc<ScheduleSearchRepository>,
    ) -> Self {
        Self(
            peer_repository,
            schedule_repository,
            schedule_search_repository,
        )
    }
}
