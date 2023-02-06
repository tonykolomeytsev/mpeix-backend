use crate::telegram_api::TelegramApi;

#[derive(Default)]
pub struct SetWebhookUseCase(TelegramApi);

impl SetWebhookUseCase {
    pub async fn set_webhook(&self) -> anyhow::Result<()> {
        self.0.set_webhook().await
    }
}

#[derive(Default)]
pub struct ReplyToTelegramUseCase(TelegramApi);

impl ReplyToTelegramUseCase {
    pub async fn reply_to_telegram(&self) -> anyhow::Result<()> {
        todo!()
    }
}
