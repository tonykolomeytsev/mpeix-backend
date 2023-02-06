use log::info;

pub fn get_address() -> (String, u16) {
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

#[macro_export]
macro_rules! define_app_error {
    ($name:tt) => {
        use actix_web::{
            http::{header::ContentType, StatusCode},
            HttpResponse,
        };
        use anyhow::anyhow;
        use common_errors::errors::CommonError;
        use std::fmt::{Debug, Display};

        #[derive(Debug)]
        pub struct $name(anyhow::Error);

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<anyhow::Error> for $name {
            fn from(value: anyhow::Error) -> Self {
                Self(value)
            }
        }

        impl actix_web::ResponseError for $name {
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
    };
}
