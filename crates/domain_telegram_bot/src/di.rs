use std::sync::Arc;

use crate::{
    telegram_api::TelegramApi,
    usecases::{DeleteMessageUseCase, ReplyToTelegramUseCase, SetWebhookUseCase},
};

impl SetWebhookUseCase {
    pub fn new(telegram_api: Arc<TelegramApi>) -> Self {
        Self(telegram_api)
    }
}

impl ReplyToTelegramUseCase {
    pub fn new(telegram_api: Arc<TelegramApi>) -> Self {
        Self(telegram_api)
    }
}

impl DeleteMessageUseCase {
    pub fn new(telegram_api: Arc<TelegramApi>) -> Self {
        Self(telegram_api)
    }
}
