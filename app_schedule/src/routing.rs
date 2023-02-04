use actix_web::{
    http::StatusCode,
    web::{Data, Json, Path, Query},
    HttpRequest, HttpResponse, Responder,
};
use anyhow::bail;
use common_errors::errors::CommonError;
use domain_mobile::AppVersion;
use domain_schedule_models::dto::v1::{Schedule, ScheduleSearchResult, ScheduleType};
use domain_telegram_bot::Update;
use domain_vk_bot::VkCallbackRequest;
use serde::{Deserialize, Serialize};

use crate::{errors::AppScheduleError, AppSchedule};

/// Health check method
/// Returns `200 OK` with text `"I'm alive"` if service is alive
#[actix_web::get("/v1/are_you_alive")]
async fn are_you_alive() -> impl Responder {
    HttpResponse::Ok().body("I'm alive :)")
}

#[derive(Serialize)]
struct GetIdResponse {
    id: i64,
}

#[actix_web::get("/v1/{type}/{name}/id")]
async fn get_id_v1(
    path: Path<(String, String)>,
    state: Data<AppSchedule>,
) -> Result<Json<GetIdResponse>, AppScheduleError> {
    let (r#type, name) = path.into_inner();
    let r#type = parse_schedule_type(&r#type)?;
    Ok(Json(GetIdResponse {
        id: state.feature_schedule.get_id(name, r#type).await?,
    }))
}

#[actix_web::get("/v1/{type}/{name}/schedule/{offset}")]
async fn get_schedule_v1(
    path: Path<(String, String, i32)>,
    state: Data<AppSchedule>,
    req: HttpRequest,
) -> Result<Json<Schedule>, AppScheduleError> {
    let (r#type, name, offset) = path.into_inner();
    let r#type = parse_schedule_type(&r#type)?;
    let app_version = get_app_version(&req);
    Ok(Json(
        state
            .feature_schedule
            .get_schedule(name, r#type, offset, app_version)
            .await?,
    ))
}

#[derive(Deserialize)]
struct SearchQuery {
    #[serde(alias = "q")]
    query: String,
    r#type: Option<String>,
}

#[actix_web::get("/v1/search")]
async fn search_schedule_v1(
    query: Query<SearchQuery>,
    state: Data<AppSchedule>,
) -> Result<Json<Vec<ScheduleSearchResult>>, AppScheduleError> {
    let r#type = match &query.r#type {
        Some(r#type) => Some(parse_schedule_type(r#type)?),
        None => None,
    };
    Ok(Json(
        state
            .feature_schedule
            .search_schedule(query.query.clone(), r#type)
            .await?,
    ))
}

#[actix_web::post("/v1/vk_callback")]
async fn vk_callback_v1(
    payload: Json<VkCallbackRequest>,
    state: Data<AppSchedule>,
) -> Result<(String, StatusCode), AppScheduleError> {
    Ok(state
        .feature_vk_bot
        .reply(payload.into_inner())
        .await
        .map(|it| (it.unwrap_or("ok".to_string()), StatusCode::OK))?)
}

#[actix_web::post("/v1/telegram_webhook_{secret}")]
async fn telegram_webhook_v1(
    path: Path<String>,
    payload: Json<Update>,
    state: Data<AppSchedule>,
) -> Result<(String, StatusCode), AppScheduleError> {
    let secret = path.into_inner();
    Ok(state
        .feature_telegram_bot
        .reply(payload.into_inner(), secret)
        .await
        .map(|_| ("ok".to_string(), StatusCode::OK))?)
}

/// Because we cannot implement trait `actix_web::FromRequest` for `ScheduleType`.
/// They belongs to different crates and no one belongs this crate.
/// I do not want to add `actix-web` dependency to `domain_schedule_models` crate.
fn parse_schedule_type(r#type: &str) -> anyhow::Result<ScheduleType> {
    match r#type {
        "group" => Ok(ScheduleType::Group),
        "person" => Ok(ScheduleType::Person),
        "room" => Ok(ScheduleType::Room),
        _ => bail!(CommonError::user(format!(
            "Unsupported schedule type: {type}"
        ))),
    }
}

fn get_app_version(req: &HttpRequest) -> Option<AppVersion> {
    req.headers()
        .get("X-App-Version")
        .and_then(|it| it.to_str().ok())
        .and_then(|it| it.parse::<AppVersion>().ok())
}
