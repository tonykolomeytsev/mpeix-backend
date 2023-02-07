use std::sync::Arc;

use domain_bot::usecases::ReplyUseCase;
use domain_vk_bot::usecases::ReplyToVkUseCase;

use crate::{Config, FeatureVkBot};

impl FeatureVkBot {
    pub fn new(
        reply_use_case: Arc<ReplyUseCase>,
        reply_to_vk_use_case: Arc<ReplyToVkUseCase>,
    ) -> Self {
        Self {
            config: Config::default(),
            reply_use_case,
            reply_to_vk_use_case,
        }
    }
}
