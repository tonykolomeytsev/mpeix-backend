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
    pub async fn reply(&self, text: &str, chat_id: i64) -> anyhow::Result<()> {
        // TODO: custom keyboard handling
        self.0.send_message(text, chat_id, None).await
    }
}
