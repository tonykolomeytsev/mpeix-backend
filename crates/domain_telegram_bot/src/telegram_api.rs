use std::env;

use anyhow::{anyhow, Context};
use common_errors::errors::CommonError;
use reqwest::{redirect::Policy, Client, ClientBuilder};

/// Representation of the Telegram API
pub struct TelegramApi {
    access_token: String,
    webhook_url: String,
    client: Client,
}

impl Default for TelegramApi {
    fn default() -> Self {
        Self {
            access_token: env::var("TELEGRAM_BOT_ACCESS_TOKEN")
                .expect("Environment variable TELEGRAM_BOT_ACCESS_TOKEN not provided"),
            webhook_url: env::var("TELEGRAM_BOT_WEBHOOK_URL")
                .expect("Environment variable TELEGRAM_BOT_WEBHOOK_URL not provided"),
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

impl TelegramApi {
    pub async fn set_webhook(&self) -> anyhow::Result<()> {
        let access_token = &self.access_token;
        let response = self
            .client
            .get(format!(
                "https://api.telegram.org/bot{access_token}/setWebhook"
            ))
            .query(&[("url", &self.webhook_url)])
            .send()
            .await
            .map_err(|e| anyhow!(CommonError::gateway(e)))
            .with_context(|| "Error while executing a request to telegram backend")?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!(CommonError::gateway(format!(
                "Telegram backend response status: {}",
                response.status()
            ))))
        }
    }

    pub async fn send_message(
        &self,
        text: &str,
        chat_id: i64,
        additional_query: Option<&[(&str, &str)]>,
    ) -> anyhow::Result<()> {
        let access_token = &self.access_token;
        let mut request = self
            .client
            .get(format!(
                "https://api.telegram.org/bot{access_token}/sendMessage"
            ))
            .query(&[("text", text), ("chat_id", &chat_id.to_string())]);
        if let Some(query) = additional_query {
            for (k, v) in query {
                request = request.query(&[(k, v)]);
            }
        }
        let response = request
            .send()
            .await
            .map_err(|e| anyhow!(CommonError::gateway(e)))
            .with_context(|| "Error while executing a request to telegram backend")?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to get error text ({e})"));
            Err(anyhow!(CommonError::gateway(format!(
                "Telegram response: {status}, {text}"
            ))))
        }
    }

    pub async fn delete_message(&self, chat_id: i64, message_id: i64) -> anyhow::Result<()> {
        let access_token = &self.access_token;
        let response = self
            .client
            .get(format!(
                "https://api.telegram.org/bot{access_token}/sendMessage"
            ))
            .query(&[
                ("chat_id", &chat_id.to_string()),
                ("message_id", &message_id.to_string()),
            ])
            .send()
            .await
            .map_err(|e| anyhow!(CommonError::gateway(e)))
            .with_context(|| "Error while executing a request to telegram backend")?;

        if response.status().is_success() {
            dbg!((response.status(), response.text().await));
            Ok(())
        } else {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to get error text ({e})"));
            Err(anyhow!(CommonError::gateway(format!(
                "Telegram response: {status}, {text}"
            ))))
        }
    }
}
