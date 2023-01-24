use actix_web::{
    get,
    http::{header::ContentType, StatusCode},
    web::Path,
    web::{Data, Json},
    App, HttpResponse, HttpServer, Responder,
};
use anyhow::bail;
use common_errors::errors::CommonError;
use domain_schedule_models::dto::v1::{self, ScheduleType};
use feature_schedule::v1::FeatureScheduleState;
use serde::Serialize;

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

#[derive(Debug, derive_more::Display)]
#[display(fmt = "{}", _0)]
struct AppScheduleError(anyhow::Error);

impl From<anyhow::Error> for AppScheduleError {
    fn from(value: anyhow::Error) -> Self {
        Self(value)
    }
}

impl actix_web::ResponseError for AppScheduleError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::plaintext())
            .body(format!("{:?}", self.0))
    }

    fn status_code(&self) -> StatusCode {
        for err in self.0.chain() {
            if let Some(common_err) = err.downcast_ref::<CommonError>() {
                return match common_err {
                    CommonError::GatewayError(_) => StatusCode::BAD_GATEWAY,
                    CommonError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    CommonError::UserError(_) => StatusCode::BAD_REQUEST,
                };
            }
        }
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(Data::new(AppScheduleState::default()))
            .service(are_you_alive)
            .service(get_id_v1)
            .service(get_schedule_v1)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
