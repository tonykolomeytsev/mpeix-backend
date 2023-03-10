use std::sync::Arc;

use domain_bot::usecases::GenerateReplyUseCase;
use domain_vk_bot::usecases::ReplyToVkUseCase;

use crate::{Config, FeatureVkBot};

impl FeatureVkBot {
    pub fn new(
        generate_reply_use_case: Arc<GenerateReplyUseCase>,
        reply_to_vk_use_case: Arc<ReplyToVkUseCase>,
    ) -> Self {
        Self {
            config: Config::default(),
            generate_reply_use_case,
            reply_to_vk_use_case,
        }
    }
}
