use anyhow::Context;
use common_errors::errors::CommonError;

use crate::{vk_api::VkApi, Keyboard};

/// Send message reply to VK
#[derive(Default)]
pub struct ReplyToVkUseCase(VkApi);

impl ReplyToVkUseCase {
    pub async fn reply(
        &self,
        text: &str,
        peer_id: i64,
        keyboard: Option<Keyboard>,
    ) -> anyhow::Result<()> {
        if let Some(keyboard) = keyboard {
            let keyboard = serde_json::to_string(&keyboard).with_context(|| {
                CommonError::internal("Error while serializing vk keyboard to JSON")
            })?;
            self.0
                .send_message(text, peer_id, Some(&[("keyboard", &keyboard)]))
                .await
        } else {
            self.0.send_message(text, peer_id, None).await
        }
    }
}
