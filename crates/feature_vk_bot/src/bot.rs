use std::{env, sync::Arc};

use anyhow::{anyhow, bail, ensure};
use common_errors::errors::CommonError;
use domain_bot::{peer::repository::PlatformId, usecases::GenerateReplyUseCase};
use domain_vk_bot::{
    usecases::ReplyToVkUseCase, NewMessageObject, VkCallbackRequest, VkCallbackType,
};

pub struct FeatureVkBot {
    pub(crate) config: Config,
    pub(crate) generate_reply_use_case: Arc<GenerateReplyUseCase>,
    pub(crate) reply_to_vk_use_case: Arc<ReplyToVkUseCase>,
}

pub(crate) struct Config {
    confirmation_code: String,
    secret: Option<String>,
    group_id: Option<i64>,
}

impl Default for Config {
    fn default() -> Self {
        let confirmation_code = env::var("VK_BOT_CONFIRMATION_CODE")
            .expect("Environment variable VK_BOT_CONFIRMATION_CODE not provided");
        let secret = env::var("VK_BOT_SECRET").ok();
        let group_id = env::var("VK_BOT_GROUP_ID")
            .ok()
            .and_then(|it| it.parse::<i64>().ok());

        Self {
            confirmation_code,
            secret,
            group_id,
        }
    }
}

impl FeatureVkBot {
    pub async fn reply(&self, callback: VkCallbackRequest) -> anyhow::Result<Option<String>> {
        ensure!(
            callback.secret == self.config.secret,
            CommonError::user("Request has invalid secret key")
        );
        if let Some(group_id) = self.config.group_id {
            ensure!(
                callback.group_id == group_id,
                CommonError::user(
                    "Field 'group_id' of the request does not match the one specified in the env"
                )
            )
        }

        match callback.r#type {
            VkCallbackType::Confirmation => Ok(Some(self.config.confirmation_code.to_owned())),
            VkCallbackType::NewMessage => {
                if let Some(NewMessageObject {
                    message,
                    client_info: _,
                }) = callback.object
                {
                    if let Some(text) = message.text {
                        let reply = self
                            .generate_reply_use_case
                            .generate_reply(PlatformId::Vk(message.peer_id), &text)
                            .await?;
                        // TODO: convert Reply model to text
                        self.reply_to_vk_use_case.reply("", message.peer_id).await?;
                    } else {
                        // TODO: handle non-text message (photo/voice/etc...)
                        // now just ignore them
                    }
                    Ok(None)
                } else {
                    bail!(CommonError::internal(
                        "Callback with type 'message' has no field 'object'"
                    ))
                }
            }
            VkCallbackType::Unknown => {
                Err(anyhow!(CommonError::internal("Unsupported callback type")))
            }
        }
    }
}
