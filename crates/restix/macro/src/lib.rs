use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn api(attr: TokenStream, item: TokenStream) -> TokenStream {
    restix_impl::api(attr.into(), item.into()).into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    restix_impl::method(restix_impl::Method::Get, attr.into(), item.into()).into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    restix_impl::method(restix_impl::Method::Post, attr.into(), item.into()).into()
}
