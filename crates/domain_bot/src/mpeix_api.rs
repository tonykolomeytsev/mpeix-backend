use std::env;

use anyhow::{anyhow, Context};
use common_errors::errors::CommonError;
use domain_schedule_models::dto::v1::{Schedule, ScheduleSearchResult, ScheduleType};
use reqwest::{redirect::Policy, Client, ClientBuilder};

pub struct MpeixApi {
    hostname: String,
    client: Client,
}

impl Default for MpeixApi {
    fn default() -> Self {
        Self {
            hostname: env::var("SCHEDULE_APP_HOSTNAME")
                .expect("Environment variable SCHEDULE_APP_HOSTNAME not provided"),
            client: ClientBuilder::new()
                .gzip(true)
                .deflate(true)
                .redirect(Policy::none())
                .timeout(std::time::Duration::from_secs(15))
                .connect_timeout(std::time::Duration::from_secs(3))
                .build()
                .expect("Something went wrong when building HttClient"),
        }
    }
}

impl MpeixApi {
    pub async fn get_schedule(
        &self,
        name: &str,
        r#type: &ScheduleType,
        offset: i32,
    ) -> anyhow::Result<Schedule> {
        let base_url = &self.hostname;
        self.client
            .get(format!("{base_url}/v1/{type}/{name}/schedule/{offset}"))
            .send()
            .await
            .map_err(|e| anyhow!(CommonError::gateway(e)))
            .with_context(|| "Error while executing a request to app_schedule microservice")?
            .json::<Schedule>()
            .await
            .map_err(|e| anyhow!(CommonError::internal(e)))
            .with_context(|| "Error while deserializing response from app_schedule microservice")
    }

    pub async fn search_schedule(
        &self,
        query: &str,
        r#type: &ScheduleType,
    ) -> anyhow::Result<Vec<ScheduleSearchResult>> {
        let base_url = &self.hostname;
        self.client
            .get(format!("{base_url}/v1/search"))
            .query(&[("type", r#type.to_string()), ("q", query.to_owned())])
            .send()
            .await
            .map_err(|e| anyhow!(CommonError::gateway(e)))
            .with_context(|| "Error while executing a request to app_schedule microservice")?
            .json::<Vec<ScheduleSearchResult>>()
            .await
            .map_err(|e| anyhow!(CommonError::internal(e)))
            .with_context(|| "Error while deserializing response from app_schedule microservice")
    }
}
