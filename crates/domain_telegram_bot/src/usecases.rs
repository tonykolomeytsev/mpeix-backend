use std::sync::Arc;

use anyhow::{bail, Context};
use common_errors::errors::CommonError;
use common_restix::ResultExt;
use log::{error, info};

use crate::{telegram_api::TelegramApi, BaseResponse, CommonKeyboardMarkup};

/// Set weebhookfor Telegram Bot API manually.
/// This use case must be started **STRICTLY** before the server starts.
pub struct SetWebhookUseCase(pub(crate) Arc<TelegramApi>);

impl SetWebhookUseCase {
    pub async fn set_webhook(&self, access_token: &str, url: &str) -> anyhow::Result<()> {
        self.0
            .set_webhook(access_token, url)
            .await
            .with_telegram_error()
    }
}

/// Send message reply to Telegram
pub struct ReplyToTelegramUseCase(pub(crate) Arc<TelegramApi>);

impl ReplyToTelegramUseCase {
    pub async fn reply(
        &self,
        access_token: &str,
        text: &str,
        chat_id: i64,
        keyboard: Option<CommonKeyboardMarkup>,
    ) -> anyhow::Result<()> {
        let keyboard = if let Some(keyboard) = keyboard {
            Some(
                match keyboard {
                    CommonKeyboardMarkup::Inline(kb) => serde_json::to_string(&kb),
                    CommonKeyboardMarkup::Reply(kb) => serde_json::to_string(&kb),
                    CommonKeyboardMarkup::Remove(kb) => serde_json::to_string(&kb),
                }
                .with_context(|| {
                    CommonError::internal("Error while serializing telegram keyboard to JSON")
                })?,
            )
        } else {
            None
        };
        self.0
            .send_message(access_token, chat_id, text, keyboard)
            .await
            .with_telegram_error()
            .with_context(|| "Error while sending Telegram message")
    }
}

/// Delete message in Telegram chat
pub struct DeleteMessageUseCase(pub(crate) Arc<TelegramApi>);

impl DeleteMessageUseCase {
    pub async fn delete_message(
        &self,
        access_token: &str,
        chat_id: i64,
        message_id: i64,
    ) -> anyhow::Result<()> {
        self.0
            .delete_message(access_token, chat_id, message_id)
            .await
            .with_telegram_error()
            .with_context(|| "Error while deleting Telegram message")
    }
}

trait BaseResponseExt<T>
where
    Self: Sized,
{
    fn with_telegram_error(self) -> anyhow::Result<()>;
}

impl BaseResponseExt<BaseResponse> for Result<BaseResponse, reqwest::Error> {
    fn with_telegram_error(self) -> anyhow::Result<()> {
        match self.with_common_error() {
            Ok(BaseResponse { ok, description }) => match (ok, description) {
                (false, Some(description)) => {
                    error!("Telegram Api rejected mpeix request with description: {description}");
                    bail!(CommonError::internal(description));
                }
                (false, None) => {
                    error!("Telegram Api rejected mpeix request without description");
                    bail!(CommonError::internal("Error description was not provided"));
                }
                (true, _) => info!("Telegram Api accepted mpeix request"),
            },
            Err(err) => return Err(err),
        }
        Ok(())
    }
}
