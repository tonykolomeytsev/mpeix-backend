use restix::{api, get};

use crate::BaseResponse;

#[api(base_url = "https://api.telegram.org")]
pub trait TelegramApi {
    #[get("/bot{access_token}/setWebhook")]
    fn set_webhook(&self, #[path] access_token: &str, #[query] url: &str) -> BaseResponse;

    #[get("/bot{access_token}/sendMessage")]
    async fn send_message(
        &self,
        #[path] access_token: &str,
        #[query] chat_id: i64,
        #[query] text: &str,
        #[query("reply_markup")] keyboard: Option<String>,
    ) -> BaseResponse;

    #[get("/bot{access_token}/deleteMessage")]
    async fn delete_message(
        &self,
        #[path] access_token: &str,
        #[query] chat_id: i64,
        #[query] message_id: i64,
    ) -> BaseResponse;
}

impl Default for TelegramApi {
    fn default() -> Self {
        TelegramApi::builder()
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
