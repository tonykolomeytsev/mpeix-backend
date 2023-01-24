mod errors;

use actix_web::{
    get, middleware,
    web::Path,
    web::{Data, Json},
    App, HttpResponse, HttpServer, Responder,
};
use anyhow::bail;
use common_errors::errors::CommonError;
use domain_schedule_models::dto::v1::{self, ScheduleType};
use feature_schedule::v1::FeatureScheduleState;
use serde::Serialize;

use crate::errors::AppScheduleError;

#[get("/are_you_alive")]
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
    state: Data<AppScheduleState>,
) -> Result<Json<GetIdResponse>, AppScheduleError> {
    let (r#type, name) = path.into_inner();
    let r#type = parse_schedule_type(r#type)?;
    Ok(Json(GetIdResponse {
        id: feature_schedule::v1::get_id(name, r#type, &state.feature_schedule_state).await?,
    }))
}

#[actix_web::get("/v1/{type}/{name}/schedule/{offset}")]
async fn get_schedule_v1(
    path: Path<(String, String, i32)>,
    state: Data<AppScheduleState>,
) -> Result<Json<v1::Schedule>, AppScheduleError> {
    let (r#type, name, offset) = path.into_inner();
    let r#type = parse_schedule_type(r#type)?;
    Ok(Json(
        feature_schedule::v1::get_schedule(name, r#type, offset, &state.feature_schedule_state)
            .await?,
    ))
}

/// Because we cannot implement trait `actix_web::FromRequest` for `ScheduleType`.
/// They belongs to different crates and no one belongs this crate.
fn parse_schedule_type(r#type: String) -> anyhow::Result<ScheduleType> {
    match r#type.as_str() {
        "group" => Ok(ScheduleType::Group),
        "person" => Ok(ScheduleType::Person),
        "room" => Ok(ScheduleType::Room),
        _ => bail!(CommonError::UserError(format!(
            "Unsupported schedule type: {}",
            r#type
        ))),
    }
}

#[derive(Default)]
struct AppScheduleState {
    feature_schedule_state: FeatureScheduleState,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(Data::new(AppScheduleState::default()))
            .service(are_you_alive)
            .service(get_id_v1)
            .service(get_schedule_v1)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
