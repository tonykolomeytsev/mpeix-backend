use common_rust::env;
use log::info;

/// Get address tuple (Host, Port) from environment variables `HOST` and `PORT`.
/// Default host in prod builds is `0.0.0.0`, in debug builds is `127.0.0.1`.
/// Default port is 8080 for all types of build.
pub fn get_address() -> (String, u16) {
    let host = env::get_or(
        "HOST",
        if cfg!(debug_assertions) {
            "127.0.0.1"
        } else {
            "0.0.0.0"
        },
    );
    let port = env::get_parsed_or::<u16>("PORT", 8080);
    info!("Starting server on {}:{}", host, port);
    (host, port)
}

/// Create struct for app scope Error and implement all necessary standard
/// and actix-web traits for further use as `Responder`.
///
/// Following traits will be implemented:
/// - [std::fmt::Debug]
/// - [std::fmt::Display]
/// - From<[anyhow::Error]>
/// - [actix_web::ResponseError]
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

        pub struct $name(anyhow::Error);

        impl Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }

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
                let status_code = self.status_code();
                HttpResponse::build(status_code)
                    .insert_header(ContentType::plaintext())
                    .body(format!("Error code: {}", status_code))
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
