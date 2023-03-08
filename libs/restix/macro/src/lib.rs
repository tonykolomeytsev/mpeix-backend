use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

/// # Restix `api` attribute macro
///
/// A trait marked with this attribute will generate a structure with the same name
/// and a builder for this structure (with the `*Builder` prefix at the end). The struct
/// will have the same methods with the same signatures as the original trait's methods.
///
/// For the implementation of REST Api calls, mark methods with attribute macros
/// `#[get("...")]`, `#[post("...")]`, and others.
///
/// # Example
///
/// ## Trait definition
/// Let's say there is a trait marked with an attribute macro `#[api]`:
/// ```no_run
/// #[api]
/// pub trait ExampleApi {
///     #[get("/search")]
///     async fn search(&self, #[query] q: &str) -> Vec<String>;
/// }
/// ```
///
/// Read more about arguments and return types in the method attribute macros documentation
/// (`#[get]`, `#[post]`, and others).
///
/// ## Usage
/// You can create and use an API instance like this:
/// ```no_run
/// let api = ExampleApi::builder()
///     .base_url("http://api.example.com")
///     .client(reqwest::Client::default())
///     .build()
///     .unwrap();
///
/// let results = api.search("ilon").await?;
/// ```
///
/// Next, you can safely clone the Api instance, because it has `reqwest::Client`
/// under the hood, which in turn has `Arc` under the hood.
///
/// ## Codegen result
/// Something like the following code will be generated:
/// ```no_run
/// pub struct ExampleApi {
///     client: restix::Restix,
///     base_url: String,
/// }
///
/// impl ExampleApi {
///     #[get("/health")]
///     async fn search(&self, q: Query) -> Vec<String> {
///         todo!()
///     }
/// }
///
/// // builder implementation
/// ```
///
/// ## `base_url` field of `#[api]` macro
///
/// You can use an alternative way of specifying the `base_url` to not specify it to your Api's Builder:
/// ```no_run
/// #[api(base_url = "https://api.telegram.org")]
/// pub trait TelegramApi {
///     #[get("/bot{access_token}/setWebhook")]
///     fn set_webhook(&self, #[path] access_token: &str, #[query] url: &str);
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn api(attr: TokenStream, item: TokenStream) -> TokenStream {
    restix_impl::api(attr.into(), item.into()).into()
}

/// # Restix `get` attribute macro
///
/// A method marked with this attribute will send a `GET` request to the specified endpoint.
///
/// ## Kinds of arguments
/// Each argument must have exactly one attribute from the list: `#[path]`, `#[query]`, `#[body]`
///
/// ### Attribute `#[path]`
/// The path part of the endpoint to send the request to.
/// The value of the argument marked with this attribute will be inserted into the path of the endpoint.
/// Under the hood, `format!` macro is used, so the argument type must implement `std::fmt::Display`.
/// #### Example:
/// ```no_run
/// #[post("/user/{id}/publish")]
/// async fn publish(&self, #[path] id: i64);
/// ```
/// #### Another example:
/// ```no_run
/// #[post("/user/{id}/publish")]
/// async fn publish(&self, #[path("id")] my_long_arg_name_id: i64);
/// ```
///
/// ### Attribute `#[query]`
/// The value of the argument marked with this attribute will be added as query to the request url.
/// Under the hood, `format!` macro is used to convert value to `String`,
/// so the argument type must implement `std::fmt::Display`.
/// #### Example:
/// Here query key is the argument name, query value is the argument value.
/// In other words, send request to `https://.../search?query=...`
/// ```no_run
/// #[get("/search")]
/// async fn search(&self, #[query] query: &str) -> Vec<String>;
/// ```
/// #### Another example:
/// Here query key is attribute argument (`"q"`), query value is the argument value.
/// In other words, send request to `https://.../search?query=...`
/// Send request to `https://.../search?q=...`
/// ```no_run
/// #[get("/search")]
/// async fn search(&self, #[query("q")] query: &str) -> Vec<String>;
/// ```
///
/// ### Attribute `#[body]`
/// There can be only one argument with this attribute, and it cannot be optional.
/// The argument type must implement `serde::Serialize`. The argument value will be added to the request body.
/// #### Example:
/// ```no_run
/// #[post("/send_message")]
/// async fn send_message(&self, #[body] message: &Message) -> Update;
/// ```
///
/// Leave the return type of the method empty so that in the generated implementation the return type
/// is `Result<Response>` from the Http client being used. For example, if the `"reqwest"` feature is enabled,
/// return type will be `reqwest::Result<reqwest::Response>`.
///
/// You can use any type `T : serde::DeserializeOwned` as return type of your method
/// if you want to get `Result<T>` as return type in generated implementation. Feature `"json"` should be
/// enabled to make this work.
///
/// # Example
///
/// ## Trait definition
/// ```no_run
/// #[api]
/// pub trait MyApi {
///     #[get("/health")]
///     async fn health(&self);
///
///     #[get("/search")]
///     async fn search(&self, #[query] q: &str) -> Vec<String>;
/// }
/// ```
/// ## Api instance usage
/// ```no_run
/// let api = MyApi::builder()
///     .base_url("https://api.example.org")
///     .client(reqwest::Client::default())
///     .build()
///     .unwrap();
///
/// // request to `https://api.example.org/health`
/// api.health().await; // reqwest::Result<reqwest::Response>
///
/// // request to `https://api.example.org/search?q=apple`
/// api.search("apple").await; // reqwest::Result<Vec<String>>
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    restix_impl::method(restix_impl::Method::Get, attr.into(), item.into()).into()
}

/// # Restix `post` attribute macro
///
/// A method marked with this attribute will send a `POST` request to the specified endpoint.
///
/// ## Kinds of arguments
/// Each argument must have exactly one attribute from the list: `#[path]`, `#[query]`, `#[body]`
///
/// ### Attribute `#[path]`
/// The path part of the endpoint to send the request to.
/// The value of the argument marked with this attribute will be inserted into the path of the endpoint.
/// Under the hood, `format!` macro is used, so the argument type must implement `std::fmt::Display`.
/// #### Example:
/// ```no_run
/// #[post("/user/{id}/publish")]
/// async fn publish(&self, #[path] id: i64);
/// ```
/// #### Another example:
/// ```no_run
/// #[post("/user/{id}/publish")]
/// async fn publish(&self, #[path("id")] my_long_arg_name_id: i64);
/// ```
///
/// ### Attribute `#[query]`
/// The value of the argument marked with this attribute will be added as query to the request url.
/// Under the hood, `format!` macro is used to convert value to `String`,
/// so the argument type must implement `std::fmt::Display`.
/// #### Example:
/// Here query key is the argument name, query value is the argument value.
/// In other words, send request to `https://.../search?query=...`
/// ```no_run
/// #[get("/search")]
/// async fn search(&self, #[query] query: &str) -> Vec<String>;
/// ```
/// #### Another example:
/// Here query key is attribute argument (`"q"`), query value is the argument value.
/// In other words, send request to `https://.../search?query=...`
/// Send request to `https://.../search?q=...`
/// ```no_run
/// #[get("/search")]
/// async fn search(&self, #[query("q")] query: &str) -> Vec<String>;
/// ```
///
/// ### Attribute `#[body]`
/// There can be only one argument with this attribute, and it cannot be optional.
/// The argument type must implement `serde::Serialize`. The argument value will be added to the request body.
/// #### Example:
/// ```no_run
/// #[post("/send_message")]
/// async fn send_message(&self, #[body] message: &Message) -> Update;
/// ```
///
/// Leave the return type of the method empty so that in the generated implementation the return type
/// is `Result<Response>` from the Http client being used. For example, if the `"reqwest"` feature is enabled,
/// return type will be `reqwest::Result<reqwest::Response>`.
///
/// You can use any type `T : serde::DeserializeOwned` as return type of your method
/// if you want to get `Result<T>` as return type in generated implementation. Feature `"json"` should be
/// enabled to make this work.
///
/// # Example
///
/// ## Trait definition
/// ```no_run
/// #[api]
/// pub trait MyApi {
///     #[get("/health")]
///     async fn health(&self);
///
///     #[get("/search")]
///     async fn search(&self, #[query] q: &str) -> Vec<String>;
/// }
/// ```
/// ## Api instance usage
/// ```no_run
/// let api = MyApi::builder()
///     .base_url("https://api.example.org")
///     .client(reqwest::Client::default())
///     .build()
///     .unwrap();
///
/// // request to `https://api.example.org/health`
/// api.health().await; // reqwest::Result<reqwest::Response>
///
/// // request to `https://api.example.org/search?q=apple`
/// api.search("apple").await; // reqwest::Result<Vec<String>>
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    restix_impl::method(restix_impl::Method::Post, attr.into(), item.into()).into()
}
