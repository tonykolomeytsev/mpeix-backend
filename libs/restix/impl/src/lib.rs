mod api;
mod commons;
mod method;

pub use api::*;
pub use method::*;

#[derive(Debug)]
pub enum Method {
    Get,
    Post,
}
