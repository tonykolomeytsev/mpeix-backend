mod common_api;
mod method;
mod query;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn common_api(attr: TokenStream, item: TokenStream) -> TokenStream {
    common_api::common_api(attr, item)
}

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    method::method("get", attr, item)
}

#[proc_macro_attribute]
pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream {
    query::query(attr, item)
}
