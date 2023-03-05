#![doc = include_str!("../README.md")]

use std::fmt::Display;

pub use restix_impl::*;
pub use restix_macro::*;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(reqwest::Error);

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HttpClient error: {}", &self.0)
    }
}

impl AsRef<reqwest::Error> for Error {
    fn as_ref(&self) -> &reqwest::Error {
        &self.0
    }
}

#[doc = include_str!("../README.md")]
#[derive(Clone, Default)]
pub struct Restix(reqwest::Client);

impl Restix {
    pub fn builder() -> RestixBuilder {
        RestixBuilder::new()
    }
}

#[derive(Default)]
pub struct RestixBuilder {
    client: Option<reqwest::Client>,
}

impl RestixBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn build(self) -> Restix {
        Restix(self.client.unwrap_or_default())
    }
}

impl crate::Restix {
    pub async fn execute_raw<'a, B>(
        &self,
        method: crate::Method,
        url: &str,
        queries: Option<Vec<(&str, &str)>>,
        body: Option<B>,
    ) -> crate::Result<reqwest::Response>
    where
        B: serde::Serialize,
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
            builder = builder.json(&body)
        }
        builder.send().await.map_err(Error)
    }

    pub async fn execute_with_serde<'a, B, R>(
        &self,
        method: crate::Method,
        url: &str,
        queries: Option<Vec<(&str, &str)>>,
        body: Option<B>,
    ) -> crate::Result<R>
    where
        B: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        self.execute_raw(method, url, queries, body)
            .await?
            .json::<R>()
            .await
            .map_err(Error)
    }
}
