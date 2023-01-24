use std::{error::Error, fmt::Display};

use derive_more::Display;

#[derive(Debug, Display)]
pub enum CommonError {
    #[display(fmt = "Internal error: {}", _0)]
    InternalError(String),
    #[display(fmt = "Gateway error: {}", _0)]
    GatewayError(String),
    #[display(fmt = "User error: {}", _0)]
    UserError(String),
}

impl CommonError {
    pub fn internal<E: Display>(e: E) -> CommonError {
        CommonError::InternalError(e.to_string())
    }

    pub fn gateway<E: Display>(e: E) -> CommonError {
        CommonError::GatewayError(e.to_string())
    }

    pub fn user<E: Display>(e: E) -> CommonError {
        CommonError::UserError(e.to_string())
    }
}

impl Error for CommonError {}
