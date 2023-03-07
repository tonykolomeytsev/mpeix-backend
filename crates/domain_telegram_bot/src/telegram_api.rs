use restix::{api, get};

use crate::BaseResponse;

#[api(base_url = "https://api.telegram.org")]
pub trait TelegramApi {
    #[get("/bot{access_token}/setWebhook")]
    fn set_webhook(&self, access_token: Path, url: Query) -> BaseResponse;

    #[get("/bot{access_token}/sendMessage")]
    #[query(keyboard = "reply_markup")]
    async fn send_message(
        &self,
        access_token: Path,
        chat_id: Query,
        text: Query,
        keyboard: Option<Query>,
    ) -> BaseResponse;

    #[get("/bot{access_token}/deleteMessage")]
    async fn delete_message(
        &self,
        access_token: Path,
        chat_id: Query,
        message_id: Query,
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
