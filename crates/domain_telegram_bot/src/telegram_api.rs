use common_rust::env;
use restix::{api, get};

use crate::BaseResponse;

#[api]
pub trait TelegramApi {
    #[get("/setWebhook")]
    async fn set_webhook(&self, #[query] url: &str) -> BaseResponse;

    #[get("/sendMessage")]
    async fn send_message(
        &self,
        #[query] chat_id: i64,
        #[query] text: &str,
        #[query("reply_markup")] keyboard: Option<String>,
    ) -> BaseResponse;

    #[get("/deleteMessage")]
    async fn delete_message(&self, #[query] chat_id: i64, #[query] message_id: i64)
        -> BaseResponse;
}

impl Default for TelegramApi {
    fn default() -> Self {
        let access_token = env::required("TELEGRAM_BOT_ACCESS_TOKEN");
        let base_url = format!("https://api.telegram.org/bot{access_token}");
        TelegramApi::builder()
            .base_url(base_url)
            .client(
                reqwest::ClientBuilder::new()
                    .gzip(true)
                    .deflate(true)
                    .redirect(reqwest::redirect::Policy::none())
                    .timeout(std::time::Duration::from_secs(15))
                    .connect_timeout(std::time::Duration::from_secs(3))
                    .pool_max_idle_per_host(0)
                    .build()
                    .expect("Error while building reqwest::Client"),
            )
            .build()
            .expect("Error while building TelegramApi")
    }
}
