use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{quote, ToTokens};
use syn::{
    parse::Parse, spanned::Spanned, Attribute, ExprAssign, ExprParen, FnArg, ImplItemMethod,
    LitStr, ReturnType, Type, TypePath,
};

use crate::{commons::StringExt, Method};

/// Intermediate representation of an Method definition.
struct MethodIR {
    name: Ident,
    attrs: Vec<AttrIR>,
    args: Vec<ArgIR>,
    return_type: ReturnTypeIR,
}

enum AttrIR {
    Query(AttrQueryIR),
    MapResponseWith(AttrMapResponseWithIR),
}

struct AttrQueryIR {
    old_name: Ident,
    new_name: LitStr,
}

struct AttrMapResponseWithIR {
    ident: Ident,
}

enum ArgIR {
    Receiver,
    Typed { name: Ident, r#type: ArgTypeIR },
}

#[derive(Default)]
struct ArgsCounter {
    common: usize,
    query: usize,
    path: usize,
}

enum ArgTypeIR {
    Query { num: usize, optional: bool },
    Path { num: usize },
    Body,
}

enum ReturnTypeIR {
    RawResponse,
    Typed(Type),
}

impl Parse for MethodIR {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let method: ImplItemMethod = input.parse()?;
        let mut args_counter = ArgsCounter::default();
        Ok(MethodIR {
            name: method.sig.ident,
            attrs: method
                .attrs
                .into_iter()
                .map(parse_attr_ir)
                .collect::<syn::Result<Vec<_>>>()?,
            args: method
                .sig
                .inputs
                .into_iter()
                .map(|fn_arg| parse_arg_ir(fn_arg, &mut args_counter))
                .collect::<syn::Result<Vec<_>>>()?,
            return_type: syn::parse2::<ReturnTypeIR>(method.sig.output.into_token_stream())?,
        })
    }
}

fn parse_attr_ir(attr: Attribute) -> syn::Result<AttrIR> {
    Ok(
        match attr.path.get_ident().map(ToString::to_string).as_deref() {
            Some("query") => AttrIR::Query(syn::parse2(attr.tokens)?),
            Some("map_response_with") => AttrIR::MapResponseWith(syn::parse2(attr.tokens)?),
            _ => return Err(syn::Error::new(attr.span(), "Unknown attribute")),
        },
    )
}

impl Parse for AttrQueryIR {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let assn: ExprAssign = syn::parse2(input.parse::<ExprParen>()?.expr.to_token_stream())?;
        Ok(AttrQueryIR {
            old_name: syn::parse2(assn.left.to_token_stream())?,
            new_name: syn::parse2(assn.right.to_token_stream())?,
        })
    }
}

impl Parse for AttrMapResponseWithIR {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = syn::parse2(input.parse::<ExprParen>()?.expr.to_token_stream())?;
        Ok(AttrMapResponseWithIR { ident })
    }
}

fn parse_arg_ir(fn_arg: FnArg, counter: &mut ArgsCounter) -> syn::Result<ArgIR> {
    counter.common += 1;
    match fn_arg {
        FnArg::Receiver(r) => match (counter.common == 1, &r.mutability, &r.reference) {
            (false, _, _) => abort!(r, "First parameter should be &self parameter"),
            (_, Some(_), _) => abort!(r, "Parameter &self should be immutable"),
            (_, _, None) => abort!(r, "Parameter &self should be a reference"),
            _ => Ok(ArgIR::Receiver),
        },
        FnArg::Typed(pat_type) => {
            if counter.common == 1 {
                abort!(pat_type, "First parameter should be &self parameter");
            }
            Ok(ArgIR::Typed {
                name: syn::parse2(pat_type.pat.into_token_stream())?,
                r#type: parse_arg_type_ir(*pat_type.ty, counter)?,
            })
        }
    }
}

fn parse_arg_type_ir(input: Type, counter: &mut ArgsCounter) -> syn::Result<ArgTypeIR> {
    let type_path: TypePath = syn::parse2(input.to_token_stream())?;
    if type_path == syn::parse_quote!(Path) {
        counter.path += 1;
        Ok(ArgTypeIR::Path { num: counter.path })
    } else if type_path == syn::parse_quote!(Query) {
        counter.query += 1;
        Ok(ArgTypeIR::Query {
            num: counter.query,
            optional: false,
        })
    } else if type_path == syn::parse_quote!(Option<Query>) {
        counter.query += 1;
        Ok(ArgTypeIR::Query {
            num: counter.query,
            optional: true,
        })
    } else if type_path == syn::parse_quote!(Body) {
        Ok(ArgTypeIR::Body)
    } else {
        Err(syn::Error::new(
            input.span(),
            "Parameter type must be one of the following types: Path, Query, Option<Query>, Body",
        ))
    }
}

macro_rules! fn_arg_as {
    ($n:ident -> $p:pat) => {
        fn $n(&self) -> Option<&Ident> {
            match self {
                Self::Typed { name, r#type: $p } => Some(name),
                _ => None,
            }
        }
    };
}

impl ArgIR {
    fn_arg_as!(as_path -> ArgTypeIR::Path { .. });
    fn_arg_as!(as_required_query -> ArgTypeIR::Query { optional: false, .. });
    fn_arg_as!(as_optional_query -> ArgTypeIR::Query { optional: true, .. });
    fn_arg_as!(as_body -> ArgTypeIR::Body);

    fn name(&self) -> Option<&Ident> {
        match self {
            Self::Typed { name, .. } => Some(name),
            _ => None,
        }
    }
}

impl ArgTypeIR {
    fn as_ident(&self) -> Ident {
        match self {
            Self::Query { num, .. } => format!("Query{num}").as_ident(),
            Self::Path { num } => format!("Path{num}").as_ident(),
            Self::Body => "Body".as_ident(),
        }
    }
}

impl Parse for ReturnTypeIR {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let return_type: ReturnType = input.parse()?;
        Ok(match return_type {
            ReturnType::Default => ReturnTypeIR::Typed(syn::parse2(quote!(()))?),
            ReturnType::Type(_, t) => match t.as_ref() {
                Type::Path(type_path) => {
                    if type_path.path.is_ident("Response") {
                        ReturnTypeIR::RawResponse
                    } else {
                        ReturnTypeIR::Typed(*t.to_owned())
                    }
                }
                _ => ReturnTypeIR::Typed(*t.to_owned()),
            },
        })
    }
}

/// Parse and validate endroint url arg of attribute macro
fn parse_attr_endpoint_url(attr: TokenStream) -> String {
    let attr_arg = syn::parse2::<LitStr>(attr).expect_or_abort("Expected string endpoint url");
    let endpoint_url = attr_arg.value();
    if !endpoint_url.starts_with('/') {
        abort!(attr_arg, "Endpoint url should start with a '/'")
    }
    endpoint_url
}

fn analyze_method_ir(ir: &MethodIR) {
    // Analyze attributes
    let arg_names = &ir.args.iter().filter_map(ArgIR::name).collect::<Vec<_>>();
    for attr in &ir.attrs {
        match attr {
            AttrIR::Query(AttrQueryIR { old_name, new_name }) => {
                if !arg_names.contains(&old_name) {
                    abort!(
                        old_name,
                        "Cannot find argument with name `{}`",
                        old_name.to_string()
                    )
                }
                if new_name.value().is_empty() || *old_name == new_name.value() {
                    abort!(new_name, "Invalid query name")
                }
            }
            _ => continue,
        }
    }
    // Analyze args
    // TODO: ensure only one body
}

pub fn method(method: Method, attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parsing
    let ir: MethodIR = syn::parse2(item).unwrap_or_abort();
    let endpoint_url = parse_attr_endpoint_url(attr);
    analyze_method_ir(&ir);
    // Codegen
    codegen_fn_impl(ir, &endpoint_url, method)
}

/// Generate impelmentation for the method from its IR
fn codegen_fn_impl(ir: MethodIR, endpoint_url: &str, method: Method) -> TokenStream {
    let name = &ir.name;
    let (type_definitions, type_bounds) = codegen_type_bounds(&ir);
    let args = codegen_fn_args(&ir);
    let return_type = match &ir.return_type {
        ReturnTypeIR::Typed(t) => quote!(#t),
        ReturnTypeIR::RawResponse => quote!(::reqwest::Response),
    };
    let fn_code_block = codegen_fn_code_block(&ir, endpoint_url, method);

    quote! {
        pub async fn #name #type_definitions ( #args ) -> ::restix::Result<#return_type>
        #type_bounds {
            #fn_code_block
        }
    }
}

/// Generate type definitions and bounds for the method
fn codegen_type_bounds(ir: &MethodIR) -> (TokenStream, TokenStream) {
    let mut type_definitions = vec![];
    let mut type_bounds = vec![];
    for arg in &ir.args {
        if let ArgIR::Typed { r#type, .. } = arg {
            let type_ident = r#type.as_ident();
            match r#type {
                ArgTypeIR::Query { .. } => type_bounds
                    .push(quote!(#type_ident: ::core::convert::AsRef<::std::primitive::str>)),
                ArgTypeIR::Path { .. } => {
                    type_bounds.push(quote!(#type_ident: ::std::fmt::Display))
                }
                ArgTypeIR::Body => type_bounds.push(quote!(#type_ident: ::serde::Serialize)),
            }
            type_definitions.push(quote!(#type_ident));
        }
    }

    if !type_bounds.is_empty() {
        (
            quote!(<#( #type_definitions ),*>),
            quote! {
                where #( #type_bounds ),*
            },
        )
    } else {
        (quote!(), quote!())
    }
}

/// Restore method args from IR
fn codegen_fn_args(ir: &MethodIR) -> TokenStream {
    let args = ir.args.iter().map(|arg| match arg {
        ArgIR::Receiver => quote!(&self),
        ArgIR::Typed { name, r#type } => {
            let type_path = r#type.as_ident();
            match r#type {
                ArgTypeIR::Query { optional: true, .. } => {
                    quote!(#name: ::std::option::Option<#type_path>)
                }
                _ => quote!(#name: #type_path),
            }
        }
    });
    quote! { #( #args ),* }
}

/// Merge codegen results into a single code block for method implementation
fn codegen_fn_code_block(ir: &MethodIR, endpoint_url: &str, method: Method) -> TokenStream {
    let format_url = codegen_format_url(ir, endpoint_url);
    let client_execution = codegen_client_execution(ir, method);
    quote! {
        #format_url
        #client_execution
    }
}

/// Generate `let full_url = format!(...)` statement
fn codegen_format_url(ir: &MethodIR, endpoint_url: &str) -> TokenStream {
    let paths = &ir
        .args
        .iter()
        .filter_map(ArgIR::as_path)
        .map(|path| {
            let key = path.to_string().unraw().as_ident();
            quote!(#key = #path)
        })
        .collect::<Vec<_>>();
    let full_url = format!("{{base_url}}{endpoint_url}");
    quote! {
        let full_url = ::std::format!(
            #full_url,
            base_url = &self.base_url,
            #( #paths ),*
        );
    }
}

/// Generate client execution statement
fn codegen_client_execution(ir: &MethodIR, method: Method) -> TokenStream {
    let method = match method {
        Method::Get => quote!(::restix::Method::Get),
        Method::Post => quote!(::restix::Method::Post),
    };
    let renamings = ir
        .attrs
        .iter()
        .filter_map(|attr| match attr {
            AttrIR::Query(AttrQueryIR { old_name, new_name }) => {
                Some((old_name.to_owned(), new_name.to_owned()))
            }
            _ => None,
        })
        .collect::<HashMap<Ident, LitStr>>();
    let queries = &ir
        .args
        .iter()
        .filter_map(ArgIR::as_required_query)
        .map(|query| {
            let key = match renamings.get(query) {
                Some(new_query) => new_query.value(),
                None => query.to_string(),
            };
            quote! { (#key, #query.as_ref()) }
        })
        .collect::<Vec<_>>();

    let opt_queries = &ir
        .args
        .iter()
        .filter_map(ArgIR::as_optional_query)
        .map(|query| {
            let key = match renamings.get(query) {
                Some(new_query) => new_query.value(),
                None => query.to_string(),
            };
            quote! {
                if let ::std::option::Option::Some(q) = #query.as_ref() {
                    queries.push( (#key, q.as_ref()) );
                }
            }
        })
        .collect::<Vec<_>>();

    let body = if let Some(body) = ir.args.iter().find_map(ArgIR::as_body) {
        quote! { ::std::option::Option::Some(#body) }
    } else {
        quote! { ::std::option::Option::<()>::None }
    };
    let execute_fn: Ident = match &ir.return_type {
        ReturnTypeIR::Typed(_) => syn::parse_quote!(execute_with_serde),
        ReturnTypeIR::RawResponse => syn::parse_quote!(execute_raw),
    };
    let response_mapping = ir
        .attrs
        .iter()
        .find_map(|attr| match attr {
            AttrIR::MapResponseWith(AttrMapResponseWithIR { ident }) => Some(quote!(.map(#ident))),
            _ => None,
        })
        .unwrap_or_else(|| quote!());

    quote! {
        let mut queries = ::std::vec![#( #queries ),*];

        #( #opt_queries )*

        self.client.#execute_fn(
            #method,
            &full_url,
            queries,
            #body,
        ).await #response_mapping
    }
}
