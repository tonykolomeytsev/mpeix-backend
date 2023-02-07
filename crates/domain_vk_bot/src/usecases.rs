use crate::vk_api::VkApi;

#[derive(Default)]
pub struct ReplyToVkUseCase(VkApi);

impl ReplyToVkUseCase {
    pub async fn reply(&self, text: &str, peer_id: i64) -> anyhow::Result<()> {
        // TODO: custom keyboard handling
        self.0.send_message(text, peer_id, None).await
    }
}
