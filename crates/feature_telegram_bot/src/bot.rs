use std::{env, sync::Arc};

use anyhow::{ensure, Context};
use common_errors::errors::CommonError;
use domain_bot::{
    models::Reply, peer::repository::PlatformId, renderer::RenderTargetPlatform,
    usecases::GenerateReplyUseCase,
};
use domain_telegram_bot::{
    usecases::{DeleteMessageUseCase, ReplyToTelegramUseCase, SetWebhookUseCase},
    ChatType, CommonKeyboardMarkup, InlineKeyboardButton, InlineKeyboardMarkup,
    ReplyKeyboardRemove, Update,
};
use log::error;
use once_cell::sync::Lazy;

pub struct FeatureTelegramBot {
    pub(crate) config: Config,
    pub(crate) generate_reply_use_case: Arc<GenerateReplyUseCase>,
    pub(crate) set_webhook_use_case: Arc<SetWebhookUseCase>,
    pub(crate) reply_to_telegram_use_case: Arc<ReplyToTelegramUseCase>,
    pub(crate) delete_message_use_case: Arc<DeleteMessageUseCase>,
}

pub(crate) struct Config {
    secret: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            secret: env::var("TELEGRAM_BOT_SECRET")
                .expect("Environment variable TELEGRAM_BOT_SECRET not provided"),
        }
    }
}

macro_rules! inline_button {
    ($text:expr, $cq:expr $(,)?) => {
        InlineKeyboardButton {
            text: $text.to_owned(),
            callback_data: $cq.to_owned(),
        }
    };
}

static KEYBOARD_EMPTY: Lazy<CommonKeyboardMarkup> = Lazy::new(|| {
    CommonKeyboardMarkup::Remove(ReplyKeyboardRemove {
        remove_keyboard: true,
    })
});

impl FeatureTelegramBot {
    pub async fn set_webhook(&self) -> anyhow::Result<()> {
        self.set_webhook_use_case.set_webhook().await
    }

    pub async fn reply(&self, update: Update, secret: String) -> anyhow::Result<()> {
        ensure!(
            secret == self.config.secret,
            CommonError::user("Request has invalid secret key")
        );
        let (text, message) = if let Some(cq) = update.callback_query {
            (cq.data, cq.message)
        } else {
            (
                update.message.as_ref().and_then(|it| it.text.to_owned()),
                update.message,
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
                .reply(&text, message.chat.id, &keyboard)
                .await
                .with_context(|| "Error while sending reply to telegram")?;
        } else {
            error!("Cannot send reply, because message is None");
        }

        // TODO: hide search results message
        let _ = self.delete_message_use_case;

        Ok(())
    }

    fn render_keyboard(&self, reply: &Reply, chat_type: &ChatType) -> CommonKeyboardMarkup {
        match (reply, chat_type) {
            (
                Reply::ScheduleSearchResults {
                    schedule_name: _,
                    results,
                },
                _,
            ) => CommonKeyboardMarkup::Inline(InlineKeyboardMarkup {
                inline_keyboard: results
                    .iter()
                    .map(|text| vec![inline_button!(text, text)])
                    .collect(),
            }),
            _ => KEYBOARD_EMPTY.to_owned(),
        }
    }
}
