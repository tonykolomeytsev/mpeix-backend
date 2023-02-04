use std::sync::Arc;

use domain_schedule::{
    schedule::repository::ScheduleRepository, search::repository::ScheduleSearchRepository,
};

use crate::peer::repository::PeerRepository;

pub struct InitDomainBotUseCase(pub(crate) Arc<PeerRepository>);

impl InitDomainBotUseCase {
    pub async fn init(&self) -> anyhow::Result<()> {
        self.0.init_peer_tables().await
    }
}

pub struct ReplyUseCase(
    pub(crate) Arc<PeerRepository>,
    pub(crate) Arc<ScheduleRepository>,
    pub(crate) Arc<ScheduleSearchRepository>,
);
