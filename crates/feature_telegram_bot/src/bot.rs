use std::{env, sync::Arc};

use anyhow::{anyhow, ensure};
use common_errors::errors::CommonError;
use domain_bot::{
    models::Reply, peer::repository::PlatformId, renderer::RenderTargetPlatform,
    usecases::GenerateReplyUseCase,
};
use domain_telegram_bot::KeyboardButton;
use domain_telegram_bot::{
    usecases::{ReplyToTelegramUseCase, SetWebhookUseCase},
    ChatType, CommonKeyboardMarkup, InlineKeyboardButton, InlineKeyboardMarkup,
    ReplyKeyboardMarkup, ReplyKeyboardRemove, Update,
};
use once_cell::sync::Lazy;

pub struct FeatureTelegramBot {
    pub(crate) config: Config,
    pub(crate) generate_reply_use_case: Arc<GenerateReplyUseCase>,
    pub(crate) set_webhook_use_case: Arc<SetWebhookUseCase>,
    pub(crate) reply_to_telegram_use_case: Arc<ReplyToTelegramUseCase>,
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
    ($text:expr $(,)?) => {
        InlineKeyboardButton {
            text: $text.to_owned(),
        }
    };
}
macro_rules! button {
    ($text:expr $(,)?) => {
        KeyboardButton {
            text: $text.to_owned(),
        }
    };
}

static KEYBOARD_EMPTY: Lazy<CommonKeyboardMarkup> = Lazy::new(|| {
    CommonKeyboardMarkup::Remove(ReplyKeyboardRemove {
        remove_keyboard: true,
    })
});
static KEYBOARD_INLINE_HELP: Lazy<CommonKeyboardMarkup> = Lazy::new(|| {
    CommonKeyboardMarkup::Inline(InlineKeyboardMarkup {
        inline_keyboard: vec![vec![inline_button!("Помощь")]],
    })
});
static KEYBOARD_DEFAULT: Lazy<CommonKeyboardMarkup> = Lazy::new(|| {
    CommonKeyboardMarkup::Reply(ReplyKeyboardMarkup {
        keyboard: vec![
            vec![button!("Ближайшие пары")],
            vec![button!("Пары сегодня"), button!("Пары завтра")],
            vec![button!("Помощь"), button!("Сменить расписание")],
        ],
        one_time_keyboard: false,
        input_field_placeholder: None,
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
        if let Some(message) = update.message {
            let reply = if let Some(text) = message.text {
                self.generate_reply_use_case
                    .generate_reply(PlatformId::Telegram(message.chat.id), &text)
                    .await
                    .unwrap_or(Reply::InternalError)
            } else {
                Reply::UnknownMessageType
            };

            let text = domain_bot::renderer::render_message(&reply, RenderTargetPlatform::Telegram);
            let keyboard = self.render_keyboard(&reply, &message.chat.r#type);
            self.reply_to_telegram_use_case
                .reply(&text, message.chat.id, &keyboard)
                .await?;

            Ok(())
        } else {
            Err(anyhow!(CommonError::user(
                "Callback with null 'message' field"
            )))
        }
    }

    fn render_keyboard(&self, reply: &Reply, chat_type: &ChatType) -> CommonKeyboardMarkup {
        match (reply, chat_type) {
            (Reply::UnknownMessageType, _) => KEYBOARD_INLINE_HELP.to_owned(),
            (_, t) if !matches!(t, ChatType::Private) => KEYBOARD_EMPTY.to_owned(),
            (
                Reply::ScheduleSearchResults {
                    schedule_name: _,
                    results,
                },
                _,
            ) => CommonKeyboardMarkup::Inline(InlineKeyboardMarkup {
                inline_keyboard: results
                    .iter()
                    .map(|text| vec![inline_button!(text)])
                    .collect(),
            }),
            _ => KEYBOARD_DEFAULT.to_owned(),
        }
    }
}
