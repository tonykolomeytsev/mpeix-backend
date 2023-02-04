use std::sync::Arc;

use domain_bot::usecases::ReplyUseCase;

use crate::{Config, FeatureVkBot};

impl FeatureVkBot {
    pub fn new(reply_use_case: Arc<ReplyUseCase>) -> Self {
        Self {
            config: Config::default(),
            reply_use_case,
        }
    }
}
