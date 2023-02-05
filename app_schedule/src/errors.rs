use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use anyhow::anyhow;
use common_errors::errors::CommonError;
use domain_schedule_models::dto::v1::ParseScheduleTypeError;

#[derive(Debug, derive_more::Display)]
#[display(fmt = "{_0}")]
pub struct AppScheduleError(anyhow::Error);

impl From<anyhow::Error> for AppScheduleError {
    fn from(value: anyhow::Error) -> Self {
        Self(value)
    }
}

impl From<ParseScheduleTypeError> for AppScheduleError {
    fn from(value: ParseScheduleTypeError) -> Self {
        Self(anyhow!(CommonError::user(value)))
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
