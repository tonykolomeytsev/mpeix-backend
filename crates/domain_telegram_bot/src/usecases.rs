use anyhow::Context;
use common_errors::errors::CommonError;

use crate::{telegram_api::TelegramApi, CommonKeyboardMarkup};

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
    pub async fn reply(
        &self,
        text: &str,
        chat_id: i64,
        keyboard: &CommonKeyboardMarkup,
    ) -> anyhow::Result<()> {
        let keyboard = match keyboard {
            CommonKeyboardMarkup::Inline(kb) => serde_json::to_string(kb),
            CommonKeyboardMarkup::Reply(kb) => serde_json::to_string(kb),
            CommonKeyboardMarkup::Remove(kb) => serde_json::to_string(kb),
        }
        .with_context(|| {
            CommonError::internal("Error while serializing telegram keyboard to JSON")
        })?;
        self.0
            .send_message(text, chat_id, Some(&[("reply_markup", &keyboard)]))
            .await
    }
}
