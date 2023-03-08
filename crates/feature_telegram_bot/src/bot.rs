use std::sync::Arc;

use anyhow::{ensure, Context};
use common_errors::errors::CommonError;
use common_rust::env;
use domain_bot::{
    models::Reply, peer::repository::PlatformId, renderer::RenderTargetPlatform,
    usecases::GenerateReplyUseCase,
};
use domain_telegram_bot::{
    usecases::{DeleteMessageUseCase, ReplyToTelegramUseCase, SetWebhookUseCase},
    ChatType, CommonKeyboardMarkup, InlineKeyboardButton, InlineKeyboardMarkup, Update,
};
use log::error;

pub struct FeatureTelegramBot {
    pub(crate) config: Config,
    pub(crate) generate_reply_use_case: Arc<GenerateReplyUseCase>,
    pub(crate) set_webhook_use_case: Arc<SetWebhookUseCase>,
    pub(crate) reply_to_telegram_use_case: Arc<ReplyToTelegramUseCase>,
    pub(crate) delete_message_use_case: Arc<DeleteMessageUseCase>,
}

pub(crate) struct Config {
    secret: String,
    webhook_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            secret: env::required("TELEGRAM_BOT_SECRET"),
            webhook_url: env::required("TELEGRAM_BOT_WEBHOOK_URL"),
        }
    }
}

macro_rules! button {
    ($text:expr, $cq:expr $(,)?) => {
        InlineKeyboardButton {
            text: $text.to_owned(),
            callback_data: $cq.to_owned(),
        }
    };
}

impl FeatureTelegramBot {
    pub async fn set_webhook(&self) -> anyhow::Result<()> {
        self.set_webhook_use_case
            .set_webhook(&self.config.webhook_url)
            .await
    }

    pub async fn reply(&self, update: Update, secret: String) -> anyhow::Result<()> {
        ensure!(
            secret == self.config.secret,
            CommonError::user("Request has invalid secret key")
        );
        let (text, message, is_callback) = if let Some(cq) = update.callback_query {
            (cq.data, cq.message, true)
        } else {
            (
                update.message.as_ref().and_then(|it| it.text.to_owned()),
                update.message,
                false,
            )
        };

        if let Some(message) = message {
            let reply = if let Some(text) = text {
                self.generate_reply_use_case
                    .generate_reply(PlatformId::Telegram(message.chat.id), &text)
                    .await
                    .unwrap_or_else(|e| {
                        error!("{e}");
                        Reply::InternalError
                    })
            } else {
                Reply::UnknownMessageType
            };
            let text = domain_bot::renderer::render_message(&reply, RenderTargetPlatform::Telegram);
            let keyboard = self.render_keyboard(&reply, &message.chat.r#type);
            self.reply_to_telegram_use_case
                .reply(&text, message.chat.id, keyboard)
                .await
                .with_context(|| "Error while sending reply to telegram")?;

            if is_callback {
                self.delete_message_use_case
                    .delete_message(message.chat.id, message.message_id)
                    .await
                    .unwrap_or_else(|e| error!("Error while deleting message: {e}"));
            }
        } else {
            error!("Cannot send reply, because message is None");
        }

        Ok(())
    }

    fn render_keyboard(&self, reply: &Reply, chat_type: &ChatType) -> Option<CommonKeyboardMarkup> {
        match (reply, chat_type) {
            (
                Reply::ScheduleSearchResults {
                    schedule_name: _,
                    results,
                    results_contains_person,
                },
                _,
            ) => Some(self.render_search_results_keyboard(results, *results_contains_person)),
            _ => None,
        }
    }

    fn render_search_results_keyboard(
        &self,
        results: &[String],
        results_contains_person: bool,
    ) -> CommonKeyboardMarkup {
        if results_contains_person {
            return CommonKeyboardMarkup::Inline(InlineKeyboardMarkup {
                inline_keyboard: results
                    .iter()
                    .map(|text| vec![button!(text, text)])
                    .collect(),
            });
        }

        let mut buttons: Vec<Vec<InlineKeyboardButton>> = vec![];
        let mut iter = results.iter();
        let mut i = 0;

        while i < results.len() - 1 {
            if let (Some(btn1), Some(btn2)) = (iter.next(), iter.next()) {
                buttons.push(vec![button!(btn1, btn1), button!(btn2, btn2)]);
            }
            i += 2;
        }
        if let Some(btn) = iter.next() {
            buttons.push(vec![button!(btn, btn)]);
        }
        CommonKeyboardMarkup::Inline(InlineKeyboardMarkup {
            inline_keyboard: buttons,
        })
    }
}
