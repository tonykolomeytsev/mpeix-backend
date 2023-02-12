use actix_web::{
    web::{Data, Json, Path, Query},
    HttpRequest, HttpResponse, Responder,
};
use anyhow::anyhow;
use common_errors::errors::CommonError;
use domain_mobile::AppVersion;
use domain_schedule_models::dto::v1::{
    ParseScheduleTypeError, Schedule, ScheduleSearchResult, ScheduleType,
};
use serde::{Deserialize, Serialize};

use crate::{AppSchedule, AppScheduleError};

/// Health check method
/// Returns `200 OK` with text `"I'm alive"` if service is alive
#[actix_web::get("v1/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("I'm alive :)")
}

#[derive(Serialize)]
struct GetIdResponse {
    id: i64,
}

#[actix_web::get("v1/{type}/{name}/id")]
async fn get_id_v1(
    path: Path<(String, String)>,
    state: Data<AppSchedule>,
) -> Result<Json<GetIdResponse>, AppScheduleError> {
    let (r#type, name) = path.into_inner();
    let r#type = r#type.parse::<ScheduleType>()?;
    Ok(Json(GetIdResponse {
        id: state.feature_schedule.get_id(name, r#type).await?,
    }))
}

#[actix_web::get("v1/{type}/{name}/schedule/{offset}")]
async fn get_schedule_v1(
    path: Path<(String, String, i32)>,
    state: Data<AppSchedule>,
    req: HttpRequest,
) -> Result<Json<Schedule>, AppScheduleError> {
    let (r#type, name, offset) = path.into_inner();
    let r#type = r#type.parse::<ScheduleType>()?;
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

#[actix_web::get("v1/search")]
async fn search_schedule_v1(
    query: Query<SearchQuery>,
    state: Data<AppSchedule>,
) -> Result<Json<Vec<ScheduleSearchResult>>, AppScheduleError> {
    let r#type = match &query.r#type {
        Some(r#type) => Some(r#type.parse::<ScheduleType>()?),
        None => None,
    };
    Ok(Json(
        state
            .feature_schedule
            .search_schedule(query.query.clone(), r#type)
            .await?,
    ))
}

fn get_app_version(req: &HttpRequest) -> Option<AppVersion> {
    req.headers()
        .get("X-App-Version")
        .and_then(|it| it.to_str().ok())
        .and_then(|it| it.parse::<AppVersion>().ok())
}

impl From<ParseScheduleTypeError> for AppScheduleError {
    fn from(value: ParseScheduleTypeError) -> Self {
        Self(anyhow!(CommonError::user(value)))
    }
}
