use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum CommonError {
    InternalError(String),
    GatewayError(String),
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

impl Display for CommonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommonError::InternalError(s) => writeln!(f, "Internal error: {}", s),
            CommonError::GatewayError(s) => writeln!(f, "Gateway error: {}", s),
            CommonError::UserError(s) => writeln!(f, "User error: {}", s),
        }
    }
}

impl Error for CommonError {}
