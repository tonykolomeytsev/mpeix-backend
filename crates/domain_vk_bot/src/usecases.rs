use anyhow::Context;
use common_errors::errors::CommonError;

use crate::{vk_api::VkApi, Keyboard};

#[derive(Default)]
pub struct ReplyToVkUseCase(VkApi);

impl ReplyToVkUseCase {
    pub async fn reply(&self, text: &str, peer_id: i64, keyboard: &Keyboard) -> anyhow::Result<()> {
        let keyboard = serde_json::to_string(keyboard).with_context(|| {
            CommonError::internal("Error while serializing vk keyboard to JSON")
        })?;
        self.0
            .send_message(text, peer_id, Some(&[("keyboard", &keyboard)]))
            .await
    }
}
