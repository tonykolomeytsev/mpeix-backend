mod errors;

use actix_web::{
    get, middleware,
    web::{Data, Json},
    web::{Path, Query},
    App, HttpResponse, HttpServer, Responder,
};
use anyhow::bail;
use common_errors::errors::CommonError;
use domain_schedule_models::dto::v1::{self, ScheduleSearchResult, ScheduleType};
use feature_schedule::v1::FeatureSchedule;
use log::info;
use serde::{Deserialize, Serialize};

use crate::errors::AppScheduleError;

/// Health check method
/// Returns `200 OK` with text `"I'm alive"` if service is alive
#[get("/v1/are_you_alive")]
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
        id: state.0.get_id(name, r#type).await?,
    }))
}

#[actix_web::get("/v1/{type}/{name}/schedule/{offset}")]
async fn get_schedule_v1(
    path: Path<(String, String, i32)>,
    state: Data<AppSchedule>,
) -> Result<Json<v1::Schedule>, AppScheduleError> {
    let (r#type, name, offset) = path.into_inner();
    let r#type = parse_schedule_type(&r#type)?;
    Ok(Json(state.0.get_schedule(name, r#type, offset).await?))
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
        state.0.search_schedule(query.query.clone(), r#type).await?,
    ))
}

/// Because we cannot implement trait `actix_web::FromRequest` for `ScheduleType`.
/// They belongs to different crates and no one belongs this crate.
/// I do not want to add `actix-web` dependency to `domain_schedule_models` crate.
pub fn parse_schedule_type(r#type: &str) -> anyhow::Result<ScheduleType> {
    match r#type {
        "group" => Ok(ScheduleType::Group),
        "person" => Ok(ScheduleType::Person),
        "room" => Ok(ScheduleType::Room),
        _ => bail!(CommonError::user(format!(
            "Unsupported schedule type: {type}"
        ))),
    }
}

fn get_addr() -> (String, u16) {
    let host = envmnt::get_or(
        "HOST",
        if cfg!(debug_assertions) {
            "127.0.0.1"
        } else {
            "0.0.0.0"
        },
    );
    let port = envmnt::get_u16("PORT", 8080);
    info!("Starting server on {}:{}", host, port);
    (host, port)
}

#[derive(Default)]
struct AppSchedule(FeatureSchedule);

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", envmnt::get_or("RUST_LOG", "info"));
    env_logger::init();
    let app_state = Data::new(AppSchedule::default());

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(app_state.clone())
            .service(are_you_alive)
            .service(get_id_v1)
            .service(get_schedule_v1)
            .service(search_schedule_v1)
    })
    .bind(get_addr())?
    .run()
    .await
}
