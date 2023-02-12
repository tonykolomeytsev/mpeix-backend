use std::sync::Arc;

use domain_bot::usecases::GenerateReplyUseCase;
use domain_telegram_bot::usecases::{
    DeleteMessageUseCase, ReplyToTelegramUseCase, SetWebhookUseCase,
};

use crate::{Config, FeatureTelegramBot};

impl FeatureTelegramBot {
    pub fn new(
        generate_reply_use_case: Arc<GenerateReplyUseCase>,
        set_webhook_use_case: Arc<SetWebhookUseCase>,
        reply_to_telegram_use_case: Arc<ReplyToTelegramUseCase>,
        delete_message_use_case: Arc<DeleteMessageUseCase>,
    ) -> Self {
        Self {
            config: Config::default(),
            generate_reply_use_case,
            set_webhook_use_case,
            reply_to_telegram_use_case,
            delete_message_use_case,
        }
    }
}
