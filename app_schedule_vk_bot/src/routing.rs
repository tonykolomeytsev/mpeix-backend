use actix_web::{
    http::StatusCode,
    web::{Data, Json},
    HttpResponse, Responder,
};
use domain_vk_bot::VkCallbackRequest;

use crate::{AppVkBot, AppVkBotError};

/// Health check method
/// Returns `200 OK` with text `"I'm alive"` if service is alive
#[actix_web::get("/v1/app_schedule_vk_bot/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("I'm alive :)")
}

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
