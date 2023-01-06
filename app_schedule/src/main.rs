use domain_schedule_models::dto::v1::{self, ScheduleType};
use feature_schedule::v1::FeatureScheduleState;
use rocket::serde::Serialize;
use rocket::{http::Status, serde::json::Json};
use rocket::{routes, State};

type AppError = (Status, Option<String>);

#[derive(Serialize)]
struct GetIdResponse {
    id: i64,
}

#[rocket::get("/v1/<type>/<name>/id")]
async fn get_id_v1(
    r#type: &str,
    name: String,
    state: &State<FeatureScheduleState>,
) -> Result<Json<GetIdResponse>, AppError> {
    Ok(Json(GetIdResponse {
        id: feature_schedule::v1::get_id(name, map_schedule_type(r#type)?, state.inner())
            .await
            .map_err(|e| (Status::BadRequest, Some(e.to_string())))?,
    }))
}

#[rocket::get("/v1/<type>/<name>/schedule/<offset>")]
async fn get_schedule_v1(
    r#type: &str,
    name: String,
    offset: i32,
    state: &State<FeatureScheduleState>,
) -> Result<Json<v1::Schedule>, AppError> {
    Ok(Json(
        feature_schedule::v1::get_schedule(name, map_schedule_type(r#type)?, offset, state.inner())
            .await
            .map_err(|e| (Status::BadRequest, Some(e.to_string())))?,
    ))
}

fn map_schedule_type(r#type: &str) -> Result<ScheduleType, AppError> {
    match r#type {
        "group" => Ok(ScheduleType::Group),
        "person" => Ok(ScheduleType::Person),
        "room" => Ok(ScheduleType::Room),
        _ => Err((Status::NotFound, None)),
    }
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![get_id_v1, get_schedule_v1])
        .manage(FeatureScheduleState::default())
}
