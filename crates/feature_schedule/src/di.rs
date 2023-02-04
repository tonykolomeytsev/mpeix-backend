use std::sync::Arc;

use domain_schedule::usecases::{GetScheduleIdUseCase, GetScheduleUseCase, SearchScheduleUseCase};

use crate::v1::FeatureSchedule;

impl FeatureSchedule {
    pub fn new(
        get_schedule_id_use_case: Arc<GetScheduleIdUseCase>,
        get_schedule_use_case: Arc<GetScheduleUseCase>,
        search_schedule_use_case: Arc<SearchScheduleUseCase>,
    ) -> Self {
        Self(
            get_schedule_id_use_case,
            get_schedule_use_case,
            search_schedule_use_case,
        )
    }
}
