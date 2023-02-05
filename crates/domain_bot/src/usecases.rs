use std::sync::Arc;

use domain_schedule::usecases::{GetScheduleUseCase, SearchScheduleUseCase};

use crate::{
    models::{Reply, UserAction},
    peer::repository::{PeerRepository, PlatformId},
};

/// Create databases if needed and run migrations.
/// This use case must be started **STRICTLY** before the server starts.
pub struct InitDomainBotUseCase(pub(crate) Arc<PeerRepository>);

impl InitDomainBotUseCase {
    pub async fn init(&self) -> anyhow::Result<()> {
        self.0.init_peer_tables().await
    }
}

pub struct ReplyUseCase(
    pub(crate) Arc<PeerRepository>,
    pub(crate) Arc<GetScheduleUseCase>,
    pub(crate) Arc<SearchScheduleUseCase>,
);

impl ReplyUseCase {
    pub async fn reply(
        &self,
        platform_id: PlatformId,
        action: UserAction,
    ) -> anyhow::Result<Reply> {
        let peer = self.0.get_peer_by_platform_id(platform_id).await?;
        match action {
            UserAction::Start => {
                if peer.is_not_started() {
                    Ok(Reply::StartGreetings)
                } else {
                    Ok(Reply::AlreadyStarted {
                        schedule_name: peer.selected_schedule,
                    })
                }
            }
            UserAction::WeekWithOffset(offset) => {
                let schedule = self
                    .1
                    .get_schedule(
                        peer.selected_schedule,
                        peer.selected_schedule_type,
                        offset as i32,
                    )
                    .await?;
                Ok(Reply::Week(schedule))
            }
            _ => todo!(),
        }
    }
}
