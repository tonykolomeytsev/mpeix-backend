use std::sync::Arc;

use anyhow::Context;
use common_errors::errors::CommonError;

use crate::{telegram_api::TelegramApi, CommonKeyboardMarkup};

/// Set weebhookfor Telegram Bot API manually.
/// This use case must be started **STRICTLY** before the server starts.
pub struct SetWebhookUseCase(pub(crate) Arc<TelegramApi>);

impl SetWebhookUseCase {
    pub async fn set_webhook(&self) -> anyhow::Result<()> {
        self.0.set_webhook().await
    }
}

/// Send message reply to Telegram
pub struct ReplyToTelegramUseCase(pub(crate) Arc<TelegramApi>);

impl ReplyToTelegramUseCase {
    pub async fn reply(
        &self,
        text: &str,
        chat_id: i64,
        keyboard: Option<CommonKeyboardMarkup>,
    ) -> anyhow::Result<()> {
        if let Some(keyboard) = keyboard {
            let keyboard = match keyboard {
                CommonKeyboardMarkup::Inline(kb) => serde_json::to_string(&kb),
                CommonKeyboardMarkup::Reply(kb) => serde_json::to_string(&kb),
                CommonKeyboardMarkup::Remove(kb) => serde_json::to_string(&kb),
            }
            .with_context(|| {
                CommonError::internal("Error while serializing telegram keyboard to JSON")
            })?;
            self.0
                .send_message(text, chat_id, Some(&[("reply_markup", &keyboard)]))
                .await
        } else {
            self.0.send_message(text, chat_id, None).await
        }
    }
}

/// Delete message in telegram chat
pub struct DeleteMessageUseCase(pub(crate) Arc<TelegramApi>);

impl DeleteMessageUseCase {
    pub async fn delete_message(&self, chat_id: i64, message_id: i64) -> anyhow::Result<()> {
        self.0
            .delete_message(chat_id, message_id)
            .await
            .with_context(|| "Error while deleting message")
    }
}
