use std::sync::Arc;

use common_di::di_constructor;
use domain_schedule_cooldown::ScheduleCooldownRepository;

use crate::{
    id::repository::ScheduleIdRepository,
    schedule::repository::ScheduleRepository,
    schedule_shift::repository::ScheduleShiftRepository,
    search::repository::ScheduleSearchRepository,
    usecases::{
        GetScheduleIdUseCase, GetScheduleUseCase, InitDomainScheduleUseCase, SearchScheduleUseCase,
    },
};

di_constructor! { GetScheduleIdUseCase(schedule_id_repository: Arc<ScheduleIdRepository>) }
di_constructor! {
    GetScheduleUseCase {
        schedule_id_repository: Arc<ScheduleIdRepository>,
        schedule_repository: Arc<ScheduleRepository>,
        schedule_shift_repository: Arc<ScheduleShiftRepository>,
        schedule_cooldown_repository: Arc<ScheduleCooldownRepository>
    }
}
di_constructor! {
    SearchScheduleUseCase {
        schedule_search_repository: Arc<ScheduleSearchRepository>,
        schedule_cooldown_repository: Arc<ScheduleCooldownRepository>
    }
}
di_constructor! {
    InitDomainScheduleUseCase(schedule_search_repository: Arc<ScheduleSearchRepository>)
}
