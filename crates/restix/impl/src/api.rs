use proc_macro2::TokenStream;
use proc_macro_error::{abort, ResultExt};
use quote::quote;
use syn::{
    token::Async, Block, ExprAssign, ImplItem, ImplItemMethod, ItemTrait, Signature, TraitItem,
    TraitItemMethod, Visibility,
};

use crate::commons::{parse_assign_left_ident, parse_assign_right_litstr, StringExt};

#[derive(Debug)]
struct ApiIR {
    api_name: String,
    base_url: Option<String>,
    methods: Vec<TraitItemMethod>,
    visibility: Visibility,
}
impl ApiIR {
    fn new() -> Self {
        Self {
            api_name: String::new(),
            base_url: None,
            methods: vec![],
            visibility: Visibility::Inherited,
        }
    }
}

pub fn api(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parsing
    let trait_definition = syn::parse2::<ItemTrait>(item)
        .expect_or_abort("Proc macro `api` can be applied only to trait definition");
    let mut ir = ApiIR::new();
    parse_trait_name(&trait_definition, &mut ir);
    parse_base_url(attr, &mut ir);
    parse_trait_signature(&trait_definition, &mut ir);
    // Codegen
    let struct_definition = codegen_struct(&ir);
    let struct_impl = codegen_struct_impl(&ir);

    quote! {
        #struct_definition
        #struct_impl
    }
    .into()
}

fn parse_trait_name(trait_definition: &ItemTrait, ir: &mut ApiIR) {
    ir.api_name = trait_definition.ident.to_string();
}

fn parse_base_url(attr: TokenStream, ir: &mut ApiIR) {
    if attr.is_empty() {
        return;
    }
    let assn =
        syn::parse2::<ExprAssign>(attr).expect_or_abort("Expected `#[api(base_url = \"...\")]`");
    let base_url_ident = parse_assign_left_ident(&assn, || "Expected `base_url` identifier");
    if base_url_ident.to_string() != "base_url" {
        abort!(base_url_ident, "Expected `base_url` identifier");
    }
    let base_url_litstr =
        parse_assign_right_litstr(&assn, || "Expected base url value (string literal)");
    let base_url = base_url_litstr.value();
    if base_url.ends_with("/") {
        abort!(
            base_url_litstr,
            "Remove trailing '/' from `base_url` string"
        );
    }
    ir.base_url = Some(base_url)
}

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

fn codegen_struct(ir: &ApiIR) -> TokenStream {
    let vis = &ir.visibility;
    let name = ir.api_name.as_ident();
    quote! {
        #vis struct #name {
            client: ::restix::Client,
            base_url: ::std::string::String,
        }
    }
}

fn codegen_struct_impl(ir: &ApiIR) -> TokenStream {
    let name = ir.api_name.as_ident();
    let methods = codegen_struct_impl_methods(ir);
    let constructor = codegen_struct_impl_constructor(ir);

    quote! {
        impl #name {
            #constructor
            #methods
        }
    }
}

fn codegen_struct_impl_constructor(ir: &ApiIR) -> TokenStream {
    let name = &ir.api_name.as_ident();
    if let Some(base_url) = &ir.base_url {
        quote! {
            pub fn new(client: ::restix::Client) -> #name {
                return #name {
                    client,
                    base_url: #base_url.to_owned(),
                }
            }
        }
    } else {
        // create constructor with `base_url` argument
        quote! {
            pub fn new(
                client: ::restix::Client,
                base_url: ::std::string::String,
            ) -> #name {
                return #name {
                    client,
                    base_url,
                }
            }
        }
    }
}

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
