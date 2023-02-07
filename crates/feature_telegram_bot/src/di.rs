use std::sync::Arc;

use domain_bot::usecases::ReplyUseCase;
use domain_telegram_bot::usecases::{ReplyToTelegramUseCase, SetWebhookUseCase};

use crate::{Config, FeatureTelegramBot};

impl FeatureTelegramBot {
    pub fn new(
        reply_use_case: Arc<ReplyUseCase>,
        set_webhook_use_case: Arc<SetWebhookUseCase>,
        reply_to_telegram_use_case: Arc<ReplyToTelegramUseCase>,
    ) -> Self {
        Self {
            config: Config::default(),
            reply_use_case,
            set_webhook_use_case,
            reply_to_telegram_use_case,
        }
    }
}
