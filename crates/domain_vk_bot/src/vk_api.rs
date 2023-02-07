use std::env;

use anyhow::{anyhow, Context};
use common_errors::errors::CommonError;
use reqwest::{redirect::Policy, Client, ClientBuilder};

const VK_API_VERSION: &str = "5.130";

pub struct VkApi {
    access_token: String,
    client: Client,
}

impl Default for VkApi {
    fn default() -> Self {
        Self {
            access_token: env::var("VK_BOT_ACCESS_TOKEN")
                .expect("Environment variable TELEGRAM_BOT_ACCESS_TOKEN not provided"),
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

impl VkApi {
    pub async fn send_message(
        &self,
        text: &str,
        peer_id: i64,
        additional_query: Option<&[(&str, &str)]>,
    ) -> anyhow::Result<()> {
        let mut request = self
            .client
            .get("https://api.vk.com/method/messages.send")
            .query(&[
                ("v", VK_API_VERSION),
                ("access_token", &self.access_token),
                ("random_id", &rand::random::<i32>().to_string()),
                ("peer_id", &peer_id.to_string()),
                ("message", text),
            ]);
        if let Some(query) = additional_query {
            for (k, v) in query {
                request = request.query(&[(k, v)]);
            }
        }
        let response = request
            .send()
            .await
            .map_err(|e| anyhow!(CommonError::gateway(e)))
            .with_context(|| "Error while executing a request to vk backend")?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!(CommonError::gateway(format!(
                "VK backend response status: {}",
                response.status()
            ))))
        }
    }
}
