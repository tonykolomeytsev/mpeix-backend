use std::fmt::Display;

pub use client::*;
pub use common_api_macro_impl::*;
use serde::Serialize;

pub type Result<T> = std::result::Result<T, Error>;

pub struct Error;

pub struct Path(String);

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Query(String);

impl AsRef<str> for Query {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

pub struct Body<T: Serialize>(T);

#[cfg(feature = "reqwest")]
mod client {
    pub struct HttpClient(pub reqwest::Client);

    impl HttpClient {
        pub async fn execute<B, R>(
            &self,
            method: &str,
            url: &str,
            queries: Option<Vec<(&str, &str)>>,
            body: Option<crate::Body<B>>,
        ) -> crate::Result<R>
        where
            B: serde::Serialize,
            R: serde::de::DeserializeOwned,
        {
            todo!()
        }
    }
}
