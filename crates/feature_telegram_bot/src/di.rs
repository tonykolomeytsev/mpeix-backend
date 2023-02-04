use std::sync::Arc;

use domain_bot::usecases::ReplyUseCase;

use crate::FeatureTelegramBot;

impl FeatureTelegramBot {
    pub fn new(reply_use_case: Arc<ReplyUseCase>) -> Self {
        Self { reply_use_case }
    }
}
