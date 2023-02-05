use std::{env, sync::Arc};

use anyhow::{anyhow, bail, ensure};
use common_errors::errors::CommonError;
use domain_bot::{peer::repository::PlatformId, usecases::ReplyUseCase};
use domain_vk_bot::{VkCallbackRequest, VkCallbackType};

pub struct FeatureVkBot {
    pub(crate) config: Config,
    pub(crate) reply_use_case: Arc<ReplyUseCase>,
}

pub(crate) struct Config {
    confirmation_code: String,
    access_token: String,
    secret: Option<String>,
    group_id: Option<i64>,
}

impl Default for Config {
    fn default() -> Self {
        let confirmation_code = env::var("VK_BOT_CONFIRMATION_CODE")
            .expect("Environment variable VK_BOT_CONFIRMATION_CODE not provided");
        let access_token = env::var("VK_BOT_ACCESS_TOKEN")
            .expect("Environment variable VK_BOT_CONFIRMATION_CODE not provided");
        let secret = env::var("VK_BOT_SECRET").ok();
        let group_id = env::var("VK_BOT_GROUP_ID")
            .ok()
            .and_then(|it| it.parse::<i64>().ok());

        Self {
            confirmation_code,
            access_token,
            secret,
            group_id,
        }
    }
}

impl FeatureVkBot {
    pub async fn reply(&self, callback: VkCallbackRequest) -> anyhow::Result<Option<String>> {
        ensure!(
            callback.secret == self.config.secret,
            CommonError::user("Invalid secret key!")
        );

        match callback.r#type {
            VkCallbackType::Confirmation => Ok(Some(self.config.confirmation_code.to_owned())),
            VkCallbackType::NewMessage => {
                if let Some(message) = callback.object {
                    self.reply_use_case
                        .reply(PlatformId::Vk(message.message.peer_id), action???)
                        .await?;
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
