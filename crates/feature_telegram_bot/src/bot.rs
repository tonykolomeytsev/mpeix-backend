use std::sync::Arc;

use domain_bot::usecases::ReplyUseCase;
use domain_telegram_bot::Update;

pub struct FeatureTelegramBot {
    pub(crate) reply_use_case: Arc<ReplyUseCase>,
}

impl FeatureTelegramBot {
    pub async fn set_webhook(&self) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn reply(&self, update: Update, secret: String) -> anyhow::Result<()> {
        let _ = self.reply_use_case;
        let _ = update;
        let _ = secret;
        todo!()
    }
}
