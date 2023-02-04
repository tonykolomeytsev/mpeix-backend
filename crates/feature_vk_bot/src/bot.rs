use std::sync::Arc;

use anyhow::{anyhow, ensure};
use common_errors::errors::CommonError;
use domain_bot::usecases::ReplyUseCase;
use domain_vk_bot::{VkCallbackRequest, VkCallbackType};

macro_rules! key_struct {
    ($name:tt, $key:expr) => {
        pub(crate) struct $name(String);
        impl Default for $name {
            fn default() -> Self {
                Self(envmnt::get_or($key, ""))
            }
        }
    };
}

pub struct FeatureVkBot {
    pub(crate) confirmation_key: ConfirmationKey,
    pub(crate) secret: Secret,
    pub(crate) reply_use_case: Arc<ReplyUseCase>,
}

key_struct!(ConfirmationKey, "VK_BOT_CONFIRMATION_KEY");
key_struct!(Secret, "VK_BOT_SECRET");

impl FeatureVkBot {
    pub async fn reply(&self, callback: VkCallbackRequest) -> anyhow::Result<Option<String>> {
        if let Some(secret) = callback.secret {
            ensure!(
                secret == self.secret.0,
                CommonError::user("Invalid secret key!")
            );
        }

        match callback.r#type {
            VkCallbackType::Confirmation => Ok(Some(self.confirmation_key.0.to_owned())),
            VkCallbackType::Unknown => {
                Err(anyhow!(CommonError::internal("Unsupported callback type")))
            }
            VkCallbackType::NewMessage => {
                let _ = self.reply_use_case;
                todo!()
            }
        }
    }
}
