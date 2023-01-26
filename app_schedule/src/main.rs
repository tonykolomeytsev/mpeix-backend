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
use feature_schedule::v1::FeatureSchedule;
use serde::Serialize;

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
    let r#type = parse_schedule_type(r#type)?;
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
    let r#type = parse_schedule_type(r#type)?;
    Ok(Json(state.0.get_schedule(name, r#type, offset).await?))
}

/// Because we cannot implement trait `actix_web::FromRequest` for `ScheduleType`.
/// They belongs to different crates and no one belongs this crate.
/// I do not want to add `actix-web` dependency to `domain_schedule_models` crate.
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
struct AppSchedule(FeatureSchedule);

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
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
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
