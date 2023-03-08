use anyhow::{bail, Context};
use common_errors::errors::CommonError;
use common_restix::ResultExt;
use log::{error, info};

use crate::{
    vk_api::{self, VkApi},
    BaseResponse, BaseResponseError, Keyboard,
};

/// Send message reply to VK
#[derive(Default)]
pub struct ReplyToVkUseCase(VkApi);

impl ReplyToVkUseCase {
    pub async fn reply(
        &self,
        access_token: &str,
        text: &str,
        peer_id: i64,
        keyboard: Option<Keyboard>,
    ) -> anyhow::Result<()> {
        let keyboard = if let Some(keyboard) = keyboard {
            Some(serde_json::to_string(&keyboard).with_context(|| {
                CommonError::internal("Error while serializing vk keyboard to JSON")
            })?)
        } else {
            None
        };
        self.0
            .send_message(
                vk_api::VK_API_VERSION,
                access_token,
                rand::random::<u32>(),
                text,
                peer_id,
                keyboard,
            )
            .await
            .with_vk_error()
    }
}

trait BaseResponseExt<T>
where
    Self: Sized,
{
    fn with_vk_error(self) -> anyhow::Result<()>;
}

impl BaseResponseExt<BaseResponse> for Result<BaseResponse, reqwest::Error> {
    fn with_vk_error(self) -> anyhow::Result<()> {
        match self.with_common_error() {
            Ok(BaseResponse { error }) => match error {
                Some(BaseResponseError { error_msg }) => {
                    error!("Vk Api rejected mpeix request with description: {error_msg}");
                    bail!(CommonError::internal(error_msg));
                }
                None => info!("Vk Api accepted mpeix request"),
            },
            Err(err) => return Err(err),
        }
        Ok(())
    }
}
