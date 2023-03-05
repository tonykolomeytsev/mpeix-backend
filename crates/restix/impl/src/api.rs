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
    api_name: String,
    props: AttrPropertiesIR,
    methods: Vec<TraitItemMethod>,
    visibility: Visibility,
}

impl Default for ApiIR {
    fn default() -> Self {
        Self {
            api_name: String::new(),
            props: AttrPropertiesIR::default(),
            methods: vec![],
            visibility: Visibility::Inherited,
        }
    }
}

#[derive(Default)]
struct AttrPropertiesIR {
    base_url: Option<LitStr>,
}

impl Parse for AttrPropertiesIR {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let result: Punctuated<ExprAssign, Token![,]> = Punctuated::parse_terminated(input)?;
        let mut props = AttrPropertiesIR::default();
        for assn in result {
            let ident: Ident = syn::parse2(assn.left.to_token_stream())?;
            let value: LitStr = syn::parse2(assn.right.to_token_stream())?;
            match ident.to_string().as_str() {
                "base_url" => props.base_url = Some(value),
                id => {
                    let message = format!("Unknown identifier: `{id}`");
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
    let trait_definition = syn::parse2::<ItemTrait>(item)
        .expect_or_abort("Proc macro `api` can be applied only to trait definition");
    let mut ir = ApiIR::default();
    parse_trait_name(&trait_definition, &mut ir);
    parse_trait_signature(&trait_definition, &mut ir);
    parse_attr(attr, &mut ir);
    // Codegen
    let struct_definition = codegen_struct(&ir);
    let struct_builder_definition = codegen_struct_builder(&ir);

    quote! {
        #struct_definition
        #struct_builder_definition
    }
}

/// Parse name of the trait
fn parse_trait_name(trait_definition: &ItemTrait, ir: &mut ApiIR) {
    ir.api_name = trait_definition.ident.to_string();
}

/// Parse and validate `base_url = "...", map_err = "..."` expressions from the `#[api]` attribute macro
fn parse_attr(attr: TokenStream, ir: &mut ApiIR) {
    if attr.is_empty() {
        return;
    }
    let props = syn::parse2::<AttrPropertiesIR>(attr)
        .expect_or_abort("Expected `#[api(base_url = \"...\", map_err = \"...\")]`");
    if let Some(base_url) = &props.base_url {
        if base_url.value().ends_with('/') {
            abort!(base_url, "Remove trailing '/' from `base_url` string");
        }
    }
    ir.props = props
}

/// Paarse all methods from the trait with `#[api]` attribute macro and
/// also remember the trait visibility
fn parse_trait_signature(trait_definition: &ItemTrait, ir: &mut ApiIR) {
    ir.methods = trait_definition
        .items
        .iter()
        .filter_map(|item| match item {
            TraitItem::Method(method) => Some(method.to_owned()),
            _ => None,
        })
        .collect::<Vec<_>>();
    ir.visibility = trait_definition.vis.to_owned();
}

/// Generate the code for the struct definition and implementation
/// (with `builder()` method and methods copied from source trait)
fn codegen_struct(ir: &ApiIR) -> TokenStream {
    let vis = &ir.visibility;
    let name = ir.api_name.as_ident();
    let builder_name = format!("{}Builder", ir.api_name).as_ident();
    let methods = codegen_struct_impl_methods(ir);

    quote! {
        #[derive(Clone)]
        #vis struct #name {
            client: ::restix::Restix,
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

/// Generate builder for Api struct.
/// Builder allow us to override `base_url` field.
fn codegen_struct_builder(ir: &ApiIR) -> TokenStream {
    let vis = &ir.visibility;
    let api_name = &ir.api_name;
    let name = ir.api_name.as_ident();
    let builder_name = format!("{}Builder", ir.api_name).as_ident();
    let builder_error_name = format!("{}BuilderError", ir.api_name).as_ident();
    let base_url = if let Some(base_url) = &ir.props.base_url.as_ref().map(LitStr::value) {
        quote!(::std::option::Option::Some(#base_url.to_owned()))
    } else {
        quote!(::std::option::Option::None)
    };

    quote! {
        #vis struct #builder_name {
            client: ::std::option::Option<::restix::Restix>,
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

            pub fn client(mut self, client: ::restix::Restix) -> #builder_name {
                self.client = ::std::option::Option::Some(client);
                self
            }

            pub fn build(self) -> ::std::result::Result<#name, #builder_error_name> {
                if self.base_url.is_none() || self.base_url.as_ref().unwrap().is_empty() {
                    return ::std::result::Result::Err(#builder_error_name)
                }
                ::std::result::Result::Ok(#name {
                    client: self.client.unwrap_or_default(),
                    base_url: self.base_url.unwrap(),
                })
            }
        }

        #[derive(::std::fmt::Debug)]
        #vis struct #builder_error_name;

        impl ::std::error::Error for #builder_error_name {}

        impl ::std::fmt::Display for #builder_error_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Cannot construct {} with empty base_url", #api_name)
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

    #[test]
    fn test_parse_trait_name() {
        let mut ir = ApiIR::default();
        let trait_definition: ItemTrait = syn::parse_quote! {
            #[api]
            pub trait ExampleTrait {}
        };
        parse_trait_name(&trait_definition, &mut ir);
        assert_eq!(ir.api_name, "ExampleTrait");
    }

    #[test]
    fn test_parse_attr() {
        let mut ir = ApiIR::default();
        let attr: TokenStream = syn::parse_quote! {
            base_url = "https://example.com",
            map_err = "map_err"
        };
        parse_attr(attr, &mut ir);
        assert_eq!(
            ir.props.base_url.map(|it| it.value()),
            Some("https://example.com".to_owned())
        );
    }

    #[test]
    fn test_parse_trait_signature_empty_signature() {
        let mut ir = ApiIR::default();
        let trait_definition: ItemTrait = syn::parse_quote! {
            #[api]
            pub trait ExampleTrait {
                #[get("/example")]
                async fn example(&self);
            }
        };
        parse_trait_signature(&trait_definition, &mut ir);
        assert!(!ir.methods.is_empty());
        assert_eq!(ir.methods[0].sig.ident, "example");
        assert!(matches!(ir.visibility, Visibility::Public(_)));
    }
}
