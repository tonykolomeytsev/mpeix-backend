use std::sync::Arc;

use domain_bot::usecases::ReplyUseCase;

pub struct FeatureTelegramBot {
    pub(crate) reply_use_case: Arc<ReplyUseCase>,
}

impl FeatureTelegramBot {
    pub async fn set_webhook(&self) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn reply(&self) -> anyhow::Result<()> {
        let _ = self.reply_use_case;
        todo!()
    }
}
