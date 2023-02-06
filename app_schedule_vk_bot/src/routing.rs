use actix_web::{
    http::StatusCode,
    web::{Data, Json},
};
use domain_vk_bot::VkCallbackRequest;

use crate::{AppVkBot, AppVkBotError};

#[actix_web::post("/v1/vk_callback")]
async fn vk_callback_v1(
    payload: Json<VkCallbackRequest>,
    state: Data<AppVkBot>,
) -> Result<(String, StatusCode), AppVkBotError> {
    Ok(state
        .feature_vk_bot
        .reply(payload.into_inner())
        .await
        .map(|it| (it.unwrap_or("ok".to_string()), StatusCode::OK))?)
}
