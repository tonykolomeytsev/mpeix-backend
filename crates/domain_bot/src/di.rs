use std::sync::Arc;

use crate::{peer::repository::PeerRepository, usecases::InitDomainBotUseCase};

impl InitDomainBotUseCase {
    pub fn new(peer_repository: Arc<PeerRepository>) -> Self {
        Self(peer_repository)
    }
}
