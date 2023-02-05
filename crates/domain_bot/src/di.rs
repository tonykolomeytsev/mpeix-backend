use std::sync::Arc;

use domain_schedule::usecases::{GetScheduleUseCase, SearchScheduleUseCase};

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
        get_schedule_use_case: Arc<GetScheduleUseCase>,
        search_schedule_use_case: Arc<SearchScheduleUseCase>,
    ) -> Self {
        Self(
            peer_repository,
            get_schedule_use_case,
            search_schedule_use_case,
        )
    }
}
