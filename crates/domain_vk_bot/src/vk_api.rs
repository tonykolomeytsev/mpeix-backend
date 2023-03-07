use reqwest::{redirect::Policy, ClientBuilder};
use restix::{api, get};

use crate::BaseResponse;

pub const VK_API_VERSION: &str = "5.130";

#[api(base_url = "https://api.vk.com")]
pub trait VkApi {
    #[get("/method/messages.send")]
    async fn send_message(
        &self,
        api_version: Query,
        access_token: Query,
        random_id: Query,
        text: Query,
        peer_id: Query,
        keyboard: Option<Query>,
    ) -> BaseResponse;
}

impl Default for VkApi {
    fn default() -> Self {
        VkApiBuilder::new()
            .client(
                ClientBuilder::new()
                    .gzip(true)
                    .deflate(true)
                    .redirect(Policy::none())
                    .timeout(std::time::Duration::from_secs(15))
                    .connect_timeout(std::time::Duration::from_secs(3))
                    .pool_max_idle_per_host(0)
                    .build()
                    .expect("Error while building reqwest::Client"),
            )
            .build()
            .expect("Error while building VkApi")
    }
}
