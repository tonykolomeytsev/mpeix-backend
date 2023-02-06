use std::sync::Arc;

use domain_bot::usecases::ReplyUseCase;
use domain_telegram_bot::{usecases::SetWebhookUseCase, Update};

pub struct FeatureTelegramBot {
    pub(crate) reply_use_case: Arc<ReplyUseCase>,
    pub(crate) set_webhook_use_case: Arc<SetWebhookUseCase>,
}

impl FeatureTelegramBot {
    pub async fn set_webhook(&self) -> anyhow::Result<()> {
        self.set_webhook_use_case.set_webhook().await
    }

    pub async fn reply(&self, update: Update, secret: String) -> anyhow::Result<()> {
        let _ = self.reply_use_case;
        let _ = update;
        let _ = secret;
        todo!()
    }
}
