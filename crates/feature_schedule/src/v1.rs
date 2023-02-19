use std::sync::Arc;

use domain_mobile::AppVersion;
use domain_schedule::usecases::{GetScheduleIdUseCase, GetScheduleUseCase, SearchScheduleUseCase};
use domain_schedule_models::{ClassesType, Schedule, ScheduleSearchResult, ScheduleType};

pub struct FeatureSchedule(
    pub(crate) Arc<GetScheduleIdUseCase>,
    pub(crate) Arc<GetScheduleUseCase>,
    pub(crate) Arc<SearchScheduleUseCase>,
);

impl FeatureSchedule {
    pub async fn get_id(&self, name: String, r#type: ScheduleType) -> anyhow::Result<i64> {
        self.0.get_id(name, r#type).await
    }

    pub async fn get_schedule(
        &self,
        name: String,
        r#type: ScheduleType,
        offset: i32,
        app_version: Option<AppVersion>,
    ) -> anyhow::Result<Schedule> {
        let mut schedule = self.1.get_schedule(name, r#type, offset).await?;

        // for backward compatibility with old mpeix apps
        if let Some(mpeix_version) = app_version {
            // if it is 1.X.X version ...
            if mpeix_version.major < 2 {
                // replace all rich classes types to Undefined,
                // to prevent crashes on mobile
                schedule
                    .weeks
                    .iter_mut()
                    .flat_map(|week| week.days.iter_mut())
                    .flat_map(|day| day.classes.iter_mut())
                    .filter(|class| {
                        matches!(class.r#type, ClassesType::Exam | ClassesType::Consultation)
                    })
                    .for_each(|class| class.r#type = ClassesType::Undefined);
            }
        }

        Ok(schedule)
    }

    pub async fn search_schedule(
        &self,
        query: String,
        r#type: Option<ScheduleType>,
    ) -> anyhow::Result<Vec<ScheduleSearchResult>> {
        self.2.search(query, r#type).await
    }
}
