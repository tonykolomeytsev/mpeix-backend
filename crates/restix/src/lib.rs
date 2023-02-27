use std::fmt::Display;

pub use client::*;
pub use restix_impl::*;
pub use restix_macro::*;

pub type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.inner())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HttpClient error: {}", self.inner())
    }
}

#[cfg(feature = "reqwest")]
mod client {
    #[derive(Debug)]
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
            body: Option<B>,
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
                builder = builder.json(&body)
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
