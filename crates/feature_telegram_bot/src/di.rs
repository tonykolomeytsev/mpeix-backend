use std::sync::Arc;

use domain_bot::usecases::ReplyUseCase;
use domain_telegram_bot::usecases::SetWebhookUseCase;

use crate::FeatureTelegramBot;

impl FeatureTelegramBot {
    pub fn new(
        reply_use_case: Arc<ReplyUseCase>,
        set_webhook_use_case: Arc<SetWebhookUseCase>,
    ) -> Self {
        Self {
            reply_use_case,
            set_webhook_use_case,
        }
    }
}
