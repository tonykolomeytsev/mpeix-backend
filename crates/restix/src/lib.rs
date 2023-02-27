use std::fmt::Display;

pub use client::*;
pub use restix_impl::*;
pub use restix_macro::*;
use serde::Serialize;

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HttpClient error: {}", self.inner())
    }
}

pub struct Path(String);

impl Path {
    pub fn new(string: String) -> Self {
        Self(string)
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Query(String);

impl Query {
    pub fn new(string: String) -> Self {
        Self(string)
    }
}

impl AsRef<str> for Query {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

pub struct Body<'a, T: Serialize>(&'a T);

impl<'a, T: Serialize> Body<'a, T> {
    pub fn new(value: &'a T) -> Self {
        Self(value)
    }
}

pub trait StringExt {
    fn as_path(self) -> Path;
    fn as_query(self) -> Query;
}

impl<S: AsRef<str>> StringExt for S {
    fn as_path(self) -> Path {
        Path(self.as_ref().to_owned())
    }

    fn as_query(self) -> Query {
        Query(self.as_ref().to_owned())
    }
}

#[cfg(feature = "reqwest")]
mod client {
    pub struct Error(reqwest::Error);

    impl Error {
        pub fn inner(&self) -> &reqwest::Error {
            &self.0
        }
    }

    #[derive(Clone)]
    pub struct HttpClient(reqwest::Client);

    impl HttpClient {
        pub fn new(client: reqwest::Client) -> Self {
            Self(client)
        }
    }

    impl HttpClient {
        pub async fn execute<'a, B, R>(
            &self,
            method: crate::Method,
            url: &str,
            queries: Option<Vec<(&str, &str)>>,
            body: Option<crate::Body<'a, B>>,
        ) -> crate::Result<R>
        where
            B: serde::Serialize,
            R: serde::de::DeserializeOwned,
        {
            let method = match method {
                crate::Method::Get => reqwest::Method::GET,
                crate::Method::Post => reqwest::Method::POST,
            };

            let mut builder = self.0.request(method, url);
            if let Some(queries) = queries {
                builder = builder.query(&queries)
            }
            if let Some(body) = body {
                builder = builder.json(body.0)
            }
            builder
                .send()
                .await
                .map_err(Error)?
                .json::<R>()
                .await
                .map_err(Error)
        }
    }
}
