#![doc = include_str!("../README.md")]

pub use restix_impl::*;
pub use restix_macro::*;

#[cfg(all(not(feature = "reqwest"), feature = "json"))]
compile_error!(r#"The "reqwest" feature must be enabled if the "json" feature is enabled"#);

#[cfg(all(not(feature = "json"), not(feature = "reqwest")))]
compile_error!(
    r#"At least one "reqwest" feature must be enabled in order to use the restix library"#
);

// impl crate::Restix {
//     pub async fn execute_raw<'a, B>(
//         &self,
//         method: crate::Method,
//         url: &str,
//         queries: Vec<(&str, &str)>,
//         body: Option<B>,
//     ) -> crate::Result<reqwest::Response>
//     where
//         B: serde::Serialize,
//     {
//         let method = match method {
//             crate::Method::Get => reqwest::Method::GET,
//             crate::Method::Post => reqwest::Method::POST,
//         };

//         let mut builder = self.0.request(method, url).query(&queries);
//         if let Some(body) = body {
//             builder = builder.json(&body)
//         }
//         builder.send().await.map_err(Error)
//     }

//     pub async fn execute_with_serde<'a, B, R>(
//         &self,
//         method: crate::Method,
//         url: &str,
//         queries: Vec<(&str, &str)>,
//         body: Option<B>,
//     ) -> crate::Result<R>
//     where
//         B: serde::Serialize,
//         R: serde::de::DeserializeOwned,
//     {
//         self.execute_raw(method, url, queries, body)
//             .await?
//             .json::<R>()
//             .await
//             .map_err(Error)
//     }
// }
