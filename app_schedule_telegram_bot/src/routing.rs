use actix_web::{
    http::StatusCode,
    web::{Data, Json, Path},
};
use domain_telegram_bot::Update;

use crate::{AppTelegramBot, AppTelegramBotError};

#[actix_web::post("/v1/telegram_webhook_{secret}")]
async fn telegram_webhook_v1(
    path: Path<String>,
    payload: Json<Update>,
    state: Data<AppTelegramBot>,
) -> Result<(String, StatusCode), AppTelegramBotError> {
    let secret = path.into_inner();
    Ok(state
        .feature_telegram_bot
        .reply(payload.into_inner(), secret)
        .await
        .map(|_| ("ok".to_string(), StatusCode::OK))?)
}
