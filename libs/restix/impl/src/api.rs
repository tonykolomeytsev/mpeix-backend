use proc_macro2::TokenStream;
use proc_macro_error::{abort, ResultExt};
use quote::{quote, ToTokens};
use syn::{
    parse::Parse, punctuated::Punctuated, token::Async, Block, ExprAssign, Ident, ImplItem,
    ImplItemMethod, ItemTrait, LitStr, Signature, Token, TraitItem, TraitItemMethod, Visibility,
};

use crate::commons::StringExt;

/// Intermediate representation of an Api trait definition.
/// This structure is generated from the `#[api]` attribute macro.
struct ApiIR {
    name: Ident,
    methods: Vec<TraitItemMethod>,
    visibility: Visibility,
}

#[derive(Default)]
struct AttrPropertiesIR {
    base_url: Option<LitStr>,
}

impl Parse for ApiIR {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item_trait: ItemTrait = input.parse()?;
        Ok(ApiIR {
            name: item_trait.ident,
            methods: item_trait
                .items
                .iter()
                .filter_map(|item| match item {
                    TraitItem::Method(method) => Some(method.to_owned()),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            visibility: item_trait.vis,
        })
    }
}

impl Parse for AttrPropertiesIR {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(AttrPropertiesIR::default());
        }
        let result: Punctuated<ExprAssign, Token![,]> = Punctuated::parse_terminated(input)?;
        let mut props = AttrPropertiesIR::default();
        for assn in result {
            let ident: Ident = syn::parse2(assn.left.to_token_stream())?;
            let value: LitStr = syn::parse2(assn.right.to_token_stream())?;
            match ident.to_string().as_str() {
                "base_url" => props.base_url = Some(value),
                id => {
                    let message = format!("Unknown identifier `{id}`, expected `base_url`");
                    return Err(syn::Error::new(ident.span(), message));
                }
            }
        }
        Ok(props)
    }
}

/// # Restix `api` attribute macro
///
/// A trait marked with this attribute will generate a structure with the same name
/// and a builder for this structure (with the `*Builder` prefix at the end). The struct
/// will have the same methods with the same signatures as the original trait's methods.
///
/// For the implementation of REST Api calls, mark methods with attribute macros
/// `#[get("...")]`, `#[post("...")]`, and others.
pub fn api(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parsing
    let ir: ApiIR = syn::parse2(item).unwrap_or_abort();
    let attr_props: AttrPropertiesIR = syn::parse2(attr).unwrap_or_abort();
    // Analyzing
    analyze_attr_props(&attr_props);
    // Codegen
    let struct_definition = codegen_struct(&ir);
    let builder_definition = codegen_struct_builder(&ir, &attr_props);

    quote! {
        #struct_definition
        #builder_definition
    }
}

fn analyze_attr_props(attr_props: &AttrPropertiesIR) {
    if let Some(base_url) = &attr_props.base_url {
        if base_url.value().is_empty() {
            abort!(base_url, "`base_url` should not be empty");
        }
        if base_url.value().ends_with('/') {
            abort!(base_url, "`base_url` should not end with `/`");
        }
    }
}

/// Generate the code for the struct definition and implementation
/// (with `builder()` method and methods copied from source trait)
fn codegen_struct(ir: &ApiIR) -> TokenStream {
    let vis = &ir.visibility;
    let name = &ir.name;
    let builder_name = format!("{}Builder", &ir.name).as_ident();
    let methods = codegen_struct_impl_methods(ir);
    let client_type = codegen_client_type();

    quote! {
        #[derive(Clone)]
        #vis struct #name {
            client: #client_type,
            base_url: ::std::string::String,
        }

        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name :: default()
            }
            #methods
        }
    }
}

#[cfg(feature = "reqwest")]
fn codegen_client_type() -> TokenStream {
    quote!(::reqwest::Client)
}

/// Generate builder for Api struct.
/// Builder allow us to override `base_url` field.
fn codegen_struct_builder(ir: &ApiIR, attr_props: &AttrPropertiesIR) -> TokenStream {
    let vis = &ir.visibility;
    let name = &ir.name;
    let builder_name = format!("{}Builder", &ir.name).as_ident();
    let builder_error_name = format!("{}BuilderError", &ir.name).as_ident();
    let builder_error_description = format!("Cannot construct {name} with {{}}");
    let client_type = codegen_client_type();
    let base_url = if let Some(base_url) = attr_props.base_url.as_ref().map(LitStr::value) {
        quote!(::std::option::Option::Some(#base_url.to_owned()))
    } else {
        quote!(::std::option::Option::None)
    };

    quote! {
        #vis struct #builder_name {
            client: ::std::option::Option<#client_type>,
            base_url: ::std::option::Option<::std::string::String>,
        }

        impl Default for #builder_name {
            fn default() -> #builder_name {
                #builder_name {
                    client: ::std::option::Option::None,
                    base_url: #base_url,
                }
            }
        }

        impl #builder_name {
            pub fn new() -> #builder_name {
                #builder_name :: default()
            }

            pub fn base_url(mut self, base_url: ::std::string::String) -> #builder_name {
                self.base_url = ::std::option::Option::Some(base_url);
                self
            }

            pub fn client(mut self, client: #client_type) -> #builder_name {
                self.client = ::std::option::Option::Some(client);
                self
            }

            pub fn build(self) -> ::std::result::Result<#name, #builder_error_name> {
                if self.base_url.is_none() || self.base_url.as_ref().unwrap().is_empty() {
                    return ::std::result::Result::Err(#builder_error_name("empty `base_url`".to_owned()));
                }
                if self.client.is_none() {
                    return ::std::result::Result::Err(#builder_error_name("empty `client`".to_owned()))
                }
                ::std::result::Result::Ok(#name {
                    client: self.client.unwrap(),
                    base_url: self.base_url.unwrap(),
                })
            }
        }

        #[derive(::std::fmt::Debug)]
        #vis struct #builder_error_name(String);

        impl ::std::error::Error for #builder_error_name {}

        impl ::std::fmt::Display for #builder_error_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, #builder_error_description, &self.0)
            }
        }
    }
}

/// Generate stud struct methods from trait methods.
/// All methods will be forced to be `pub` and `async`.
fn codegen_struct_impl_methods(ir: &ApiIR) -> TokenStream {
    let vis: Visibility = syn::parse_quote!(pub);
    let block: Block = syn::parse_quote!({ todo!() });
    let asyncness: Async = syn::parse_quote!(async);

    let methods = ir
        .methods
        .iter()
        .map(|method| {
            ImplItem::Method(ImplItemMethod {
                attrs: method.attrs.to_owned(),
                vis: vis.to_owned(),
                defaultness: None,
                sig: Signature {
                    asyncness: Some(asyncness),
                    ..method.sig.to_owned()
                },
                block: block.to_owned(),
            })
        })
        .collect::<Vec<_>>();
    quote! {
        #( #methods )*
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use proc_macro2::Group;

    #[test]
    fn test_parse_empty_trait() {
        let trait_definition: ItemTrait = syn::parse_quote! {
            #[api]
            pub trait ExampleTrait {}
        };
        let ir: ApiIR = syn::parse2(trait_definition.to_token_stream()).unwrap();
        assert_eq!(ir.name, "ExampleTrait");
    }

    #[test]
    fn test_parse_empty_trait_with_base_url() {
        let trait_definition: ItemTrait = syn::parse_quote! {
            #[api(base_url = "https://example.com")]
            pub trait ExampleTrait {}
        };
        let ir: ApiIR = syn::parse2(trait_definition.to_token_stream()).unwrap();
        let group: Group =
            syn::parse2(trait_definition.attrs.first().unwrap().tokens.to_owned()).unwrap();
        let attr_props: AttrPropertiesIR = syn::parse2(group.stream().to_token_stream()).unwrap();
        assert_eq!(ir.name, "ExampleTrait");
        assert_eq!(
            attr_props.base_url.map(|it| it.value()),
            Some("https://example.com".to_string())
        );
    }
}
