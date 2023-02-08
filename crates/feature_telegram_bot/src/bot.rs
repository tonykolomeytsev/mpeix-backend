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
            ensure!(
                message.text.is_some(),
                CommonError::user("Callback with type 'message' has null field 'message.text'")
            );
            self.generate_reply_use_case
                .reply(
                    PlatformId::Telegram(message.chat.id),
                    &message.text.unwrap(),
                )
                .await?;
            Ok(())
        } else {
            Err(anyhow!(CommonError::user(
                "Callback with null 'message' field"
            )))
        }
    }
}
