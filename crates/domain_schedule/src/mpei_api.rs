use std::fmt::Display;

use anyhow::{anyhow, Context};
use chrono::NaiveDate;
use common_errors::errors::CommonError;
use domain_schedule_models::ScheduleType;
use log::info;
use reqwest::{redirect::Policy, Client, ClientBuilder, IntoUrl};
use serde::{de::DeserializeOwned, Serialize};

use crate::dto::mpei::MpeiClasses;

pub struct MpeiApi(Client);

impl MpeiApi {
    pub fn with_timeout_ms(timeout: u64) -> Self {
        Self(
            ClientBuilder::new()
                .gzip(true)
                .deflate(true)
                .redirect(Policy::none())
                .timeout(std::time::Duration::from_secs(15))
                .connect_timeout(std::time::Duration::from_millis(timeout))
                .pool_max_idle_per_host(3)
                .build()
                .expect("Something went wrong when building HttClient"),
        )
    }
}

impl MpeiApi {
    pub async fn search<T: DeserializeOwned>(
        &self,
        query: &str,
        r#type: &ScheduleType,
    ) -> anyhow::Result<T> {
        self.get(
            "http://ts.mpei.ru/api/search",
            &[("term", query.to_string()), ("type", r#type.to_string())],
        )
        .await
    }

    pub async fn get_schedule(
        &self,
        r#type: ScheduleType,
        schedule_id: i64,
        start: &NaiveDate,
        end: &NaiveDate,
    ) -> anyhow::Result<Vec<MpeiClasses>> {
        self.get(
            format!("http://ts.mpei.ru/api/schedule/{type}/{schedule_id}"),
            &[
                ("start", &start.format("%Y.%m.%d").to_string()),
                ("finish", &end.format("%Y.%m.%d").to_string()),
            ],
        )
        .await
    }

    async fn get<U, Q, T>(&self, url: U, query: &Q) -> anyhow::Result<T>
    where
        U: IntoUrl + Display,
        Q: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        info!("-> GET {url}");
        self.0
            .get(url)
            .query(query)
            .send()
            .await
            .map_err(|e| anyhow!(CommonError::gateway(e)))
            .with_context(|| "Error while executing a request to MPEI backend")?
            .json::<T>()
            .await
            .map_err(|e| anyhow!(CommonError::internal(e)))
            .with_context(|| "Error while deserializing response from MPEI backend")
    }
}
