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
/// #[api(base_url = "http://example.com/")] // also you can use this macro without `base_url`
/// pub trait ExampleApi {
///     #[get("health")]
///     async fn search(&self, q: Query) -> Vec<String>;
/// }
/// ```
///
/// Read more about arguments and return types in the method attribute macros documentation
/// (`#[get]`, `#[post]`, and others).
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
///     pub fn builder() -> ExampleApiBuilder {
///         // implementation
///     }
///
///     #[get("/health")]
///     async fn search(&self, q: Query) -> Vec<String> {
///         todo!()
///     }
/// }
///
/// pub struct ExampleApiBuilderError;
///
/// pub struct ExampleApiBuilder {
///     // implementation
/// }
/// ```
///
/// ## Usage
/// You can create an API instance like this:
/// ```no_run
/// let api = ExampleApi::builder()
///     .base_url("http://example.com/") // you can override base url if you want
///     .client( // specify the restix client
///         Restix::builder() // with the native reqwest client
///             .client(reqwest::Client::default())
///             .build()
///             .unwrap(),
///     )
///     .build()
///     .unwrap();
/// ```
///
/// Next, you can safely clone the Api instance, because it has `reqwest::Client`
/// under the hood, which in turn has `Arc` under the hood.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn api(attr: TokenStream, item: TokenStream) -> TokenStream {
    restix_impl::api(attr.into(), item.into()).into()
}

/// # Restix `get` attribute macro
///
/// A method marked with this attribute will send a `GET` request to the specified endpoint.
///
/// You can use the following argument types for your methods:
/// - `Query : AsRef<str>`: Each argument of this type will be added as a query to the request URL.
/// - `Path : Display`: Each argument of this type will be used in the request path.
/// - `Body : serde::Serialize`: Each method can have only one body.
///
/// You can use `Response` as return type of your method if you want to get
/// `Result<reqwest::Response>` as return type in generated implementation.
///
/// You can use any type `T : serde::DeserializeOwned` as return type of your method
/// if you want to get `Result<T>` as return type in generated implementation.
///
/// # Example
///
/// ## Trait definition
/// ```no_run
/// #[api]
/// pub trait MyApi {
///     #[get("/health")]
///     async fn health(&self) -> Response;
///
///     #[get("/search")]
///     async fn search(&self, q: Query) -> Vec<String>;
///
///     #[get("/status/{id}")]
///     async fn status(&self, id: Path) -> Response;
/// }
/// ```
/// ## Api instance usage
/// ```no_run
/// let api = MyApi::builder()
///     .base_url("https://api.example.org")
///     .build()
///     .unwrap();
///
/// // request to `https://api.example.org/health`
/// api.health().await; // Result<reqwest::Response>
///
/// // request to `https://api.example.org/search?q=apple`
/// api.search("apple").await; // Result<Vec<String>>
///
/// // request to `https://api.example.org/status/12345`
/// api.status("12345").await; // Result<reqwest::Response>
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
/// You can use the following argument types for your methods:
/// - `Query : AsRef<str>`: Each argument of this type will be added as a query to the request URL.
/// - `Path : Display`: Each argument of this type will be used in the request path.
/// - `Body : serde::Serialize`: Each method can have only one body.
///
/// You can use `Response` as return type of your method if you want to get
/// `Result<reqwest::Response>` as return type in generated implementation.
///
/// You can use any type `T : serde::DeserializeOwned` as return type of your method
/// if you want to get `Result<T>` as return type in generated implementation.
///
/// # Example
///
/// ## Trait definition
/// ```no_run
/// #[api]
/// pub trait MyApi {
///     #[get("/health")]
///     async fn health(&self) -> Response;
///
///     #[get("/search")]
///     async fn search(&self, q: Query) -> Vec<String>;
///
///     #[post("/status/{id}")]
///     async fn status(&self, id: Path, body: Body) -> Response;
/// }
/// ```
/// ## Api instance usage
/// ```no_run
/// let api = MyApi::builder()
///     .base_url("https://api.example.org")
///     .build()
///     .unwrap();
///
/// // request to `https://api.example.org/health`
/// api.health().await; // Result<reqwest::Response>
///
/// // request to `https://api.example.org/search?q=apple`
/// api.search("apple").await; // Result<Vec<String>>
///
/// // request to `https://api.example.org/status/12345`
/// api.status("12345", Status::default()).await; // Result<reqwest::Response>
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    restix_impl::method(restix_impl::Method::Post, attr.into(), item.into()).into()
}
