use anyhow::Context;
use common_errors::errors::CommonError;

use crate::{vk_api::VkApi, Keyboard, MessagePeerType};

/// Send message reply to VK
#[derive(Default)]
pub struct ReplyToVkUseCase(VkApi);

impl ReplyToVkUseCase {
    pub async fn reply(
        &self,
        text: &str,
        peer_id: i64,
        peer_type: &MessagePeerType,
        keyboard: &Keyboard,
    ) -> anyhow::Result<()> {
        let keyboard = serde_json::to_string(keyboard).with_context(|| {
            CommonError::internal("Error while serializing vk keyboard to JSON")
        })?;
        self.0
            .send_message(text, peer_id, peer_type, Some(&[("keyboard", &keyboard)]))
            .await
    }
}
