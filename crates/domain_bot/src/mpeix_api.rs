use std::env;

use anyhow::{anyhow, Context};
use common_errors::errors::CommonError;
use domain_schedule_models::{Schedule, ScheduleSearchResult, ScheduleType};
use reqwest::{redirect::Policy, Client, ClientBuilder};
use serde::Deserialize;

/// Representation of the `app_schedule` microservice API
pub struct MpeixApi {
    base_url: String,
    client: Client,
}

impl Default for MpeixApi {
    fn default() -> Self {
        Self {
            base_url: env::var("APP_SCHEDULE_BASE_URL")
                .expect("Environment variable APP_SCHEDULE_BASE_URL not provided"),
            client: ClientBuilder::new()
                .gzip(true)
                .deflate(true)
                .redirect(Policy::none())
                .timeout(std::time::Duration::from_secs(15))
                .connect_timeout(std::time::Duration::from_secs(3))
                .pool_max_idle_per_host(0)
                .build()
                .expect("Something went wrong when building HttClient"),
        }
    }
}

impl MpeixApi {
    /// Get schedule from `app_schedule` microservice
    pub async fn get_schedule(
        &self,
        name: &str,
        r#type: &ScheduleType,
        offset: i32,
    ) -> anyhow::Result<Schedule> {
        let base_url = &self.base_url;
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

    /// Get search results from `app_schedule` microservice
    pub async fn search_schedule(
        &self,
        query: &str,
        r#type: Option<&ScheduleType>,
    ) -> anyhow::Result<Vec<ScheduleSearchResult>> {
        let base_url = &self.base_url;
        let mut request = self
            .client
            .get(format!("{base_url}/v1/search"))
            .query(&[("q", query)]);

        if let Some(r#type) = r#type {
            request = request.query(&[("type", &r#type.to_string())]);
        }

        #[derive(Deserialize)]
        struct SearchResponse {
            items: Vec<ScheduleSearchResult>,
        }

        request
            .send()
            .await
            .map_err(|e| anyhow!(CommonError::gateway(e)))
            .with_context(|| "Error while executing a request to app_schedule microservice")?
            .json::<SearchResponse>()
            .await
            .map_err(|e| anyhow!(CommonError::internal(e)))
            .map(|it| it.items)
            .with_context(|| "Error while deserializing response from app_schedule microservice")
    }
}
