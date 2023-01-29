use domain_schedule::{
    di::DomainScheduleModule,
    usecases::{
        GetScheduleIdUseCase, GetScheduleUseCase, InitDomainScheduleUseCase, SearchScheduleUseCase,
    },
};
use domain_schedule_models::dto::v1::{Schedule, ScheduleSearchResult, ScheduleType};

pub struct FeatureSchedule(
    GetScheduleIdUseCase,
    GetScheduleUseCase,
    SearchScheduleUseCase,
    InitDomainScheduleUseCase,
);

impl Default for FeatureSchedule {
    fn default() -> Self {
        let domain_schedule_module = DomainScheduleModule::default();
        Self(
            domain_schedule_module.get_schedule_id_use_case,
            domain_schedule_module.get_schedule_use_case,
            domain_schedule_module.search_schedule_use_case,
            domain_schedule_module.init_domain_schedule_use_case,
        )
    }
}

impl FeatureSchedule {
    pub async fn get_id(&self, name: String, r#type: ScheduleType) -> anyhow::Result<i64> {
        self.0.get_id(name, r#type).await
    }

    pub async fn get_schedule(
        &self,
        name: String,
        r#type: ScheduleType,
        offset: i32,
    ) -> anyhow::Result<Schedule> {
        self.1.get_schedule(name, r#type, offset).await
    }

    pub async fn search_schedule(
        &self,
        query: String,
        r#type: Option<ScheduleType>,
    ) -> anyhow::Result<Vec<ScheduleSearchResult>> {
        self.2.search(query, r#type).await
    }

    pub async fn init_domain_schedule(&self) -> anyhow::Result<()> {
        self.3.init().await
    }
}
