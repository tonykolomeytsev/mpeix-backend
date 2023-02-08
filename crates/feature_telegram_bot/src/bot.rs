use std::{env, sync::Arc};

use anyhow::{anyhow, ensure};
use common_errors::errors::CommonError;
use domain_bot::{peer::repository::PlatformId, usecases::GenerateReplyUseCase};
use domain_telegram_bot::{
    usecases::{ReplyToTelegramUseCase, SetWebhookUseCase},
    Update,
};

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
            if let Some(text) = message.text {
                let reply = self
                    .generate_reply_use_case
                    .generate_reply(PlatformId::Telegram(message.chat.id), &text)
                    .await?;
                // TODO: convert Reply model to text
                self.reply_to_telegram_use_case
                    .reply("", message.chat.id)
                    .await?;
            } else {
                // TODO: handle non-text message (photo/voice/etc...)
                // now just ignore them
            }
            Ok(())
        } else {
            Err(anyhow!(CommonError::user(
                "Callback with null 'message' field"
            )))
        }
    }
}
