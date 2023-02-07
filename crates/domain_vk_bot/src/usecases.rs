use crate::vk_api::VkApi;

#[derive(Default)]
pub struct ReplyToVkUseCase(VkApi);

impl ReplyToVkUseCase {
    pub async fn reply(&self) -> anyhow::Result<()> {
        todo!()
    }
}
