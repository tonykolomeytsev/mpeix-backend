use std::sync::Arc;

use common_di::di_constructor;

use crate::{
    mpeix_api::MpeixApi,
    peer::repository::PeerRepository,
    schedule::repository::ScheduleRepository,
    search::repository::ScheduleSearchRepository,
    usecases::{
        GenerateReplyUseCase, GetUpcomingEventsUseCase, InitDomainBotUseCase, TextToActionUseCase,
    },
};

di_constructor! { ScheduleRepository(api: MpeixApi) }
di_constructor! { ScheduleSearchRepository(api: MpeixApi) }
di_constructor! { InitDomainBotUseCase(peer_repository: Arc<PeerRepository>) }
di_constructor! { GetUpcomingEventsUseCase(schedule_repository: Arc<ScheduleRepository>) }
di_constructor! {
    GenerateReplyUseCase(
        text_to_action_use_case: Arc<TextToActionUseCase>,
        peer_repository: Arc<PeerRepository>,
        schedule_repository: Arc<ScheduleRepository>,
        schedule_search_repository: Arc<ScheduleSearchRepository>,
        get_upcoming_events_use_case: Arc<GetUpcomingEventsUseCase>
    )
}
