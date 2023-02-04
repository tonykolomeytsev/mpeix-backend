use std::sync::Arc;

use deadpool_postgres::Pool;

use crate::{peer::repository::PeerRepository, usecases::InitDomainBotUseCase};

pub struct DomainBotModule {
    init_domain_bot_use_case: Arc<InitDomainBotUseCase>,
}

impl DomainBotModule {
    pub fn new(db_pool: Arc<Pool>) -> Self {
        let peer_repository = Arc::new(PeerRepository::new(db_pool));

        Self {
            init_domain_bot_use_case: Arc::new(InitDomainBotUseCase(peer_repository)),
        }
    }
}
