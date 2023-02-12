use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use domain_telegram_bot::Update;

use crate::{AppTelegramBot, AppTelegramBotError};

/// Health check method
/// Returns `200 OK` with text `"I'm alive"` if service is alive
#[actix_web::get("v1/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("I'm alive :)")
}

#[actix_web::post("v1/telegram_webhook_{secret}")]
async fn telegram_webhook_v1(
    path: Path<String>,
    payload: Json<Update>,
    state: Data<AppTelegramBot>,
) -> Result<impl Responder, AppTelegramBotError> {
    let secret = path.into_inner();
    Ok(state
        .feature_telegram_bot
        .reply(payload.into_inner(), secret)
        .await
        .map(|_| HttpResponse::Ok().body("ok"))?)
}
