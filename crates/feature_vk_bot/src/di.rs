use std::sync::Arc;

use domain_bot::usecases::ReplyUseCase;

use crate::{ConfirmationKey, FeatureVkBot, Secret};

impl FeatureVkBot {
    pub fn new(reply_use_case: Arc<ReplyUseCase>) -> Self {
        Self {
            confirmation_key: ConfirmationKey::default(),
            secret: Secret::default(),
            reply_use_case,
        }
    }
}
