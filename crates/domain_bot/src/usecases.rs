use std::sync::Arc;

use crate::peer::repository::PeerRepository;

pub struct InitDomainBotUseCase(pub(crate) Arc<PeerRepository>);

impl InitDomainBotUseCase {
    pub async fn init(&self) -> anyhow::Result<()> {
        self.0.init_peer_tables().await
    }
}
