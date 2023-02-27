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

#[derive(Clone, Default)]
pub struct Client(NativeClient);

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }
}

pub struct ClientBuilder {
    client: Option<NativeClient>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self { client: None }
    }

    pub fn client(mut self, client: NativeClient) -> Self {
        self.client = Some(client);
        self
    }

    pub fn build(self) -> Client {
        Client(self.client.unwrap_or_default())
    }
}

pub trait RequestMiddleware {
    fn on_request(&self, builder: NativeRequestBuilder);
}

#[cfg(feature = "reqwest")]
mod client {
    pub(crate) type NativeClient = reqwest::Client;
    pub(crate) type NativeRequestBuilder = reqwest::RequestBuilder;

    #[derive(Debug)]
    pub struct Error(reqwest::Error);

    impl Error {
        pub fn inner(&self) -> &reqwest::Error {
            &self.0
        }
    }

    impl crate::Client {
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
