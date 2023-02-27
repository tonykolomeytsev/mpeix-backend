use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn common_api(attr: TokenStream, item: TokenStream) -> TokenStream {
    restix_impl::common_api::common_api(attr.into(), item.into()).into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    restix_impl::method::method(restix_impl::Method::Get, attr.into(), item.into()).into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    restix_impl::method::method(restix_impl::Method::Post, attr.into(), item.into()).into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream {
    restix_impl::query::query(attr.into(), item.into()).into()
}
