use std::{collections::HashMap, fmt::Debug};

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{quote, ToTokens};
use syn::{ExprAssign, ExprParen, FnArg, ImplItemMethod, LitStr, Pat, PatType, ReturnType, Type};

use crate::{
    commons::{parse_assign_left_ident, parse_assign_right_litstr, StringExt},
    Method,
};

/// Intermediate representation of an Method definition.
struct MethodIR {
    method: Method,
    fn_name: String,
    endpoint_url: String,
    queries: Option<Vec<String>>,
    query_rename: Option<HashMap<String, String>>,
    paths: Option<Vec<String>>,
    body: Option<String>,
    fn_args: Vec<FnArgIR>,
    fn_return_type: FnReturnTypeIR,
}

impl MethodIR {
    fn new(method: Method) -> Self {
        Self {
            method,
            fn_name: String::new(),
            endpoint_url: String::new(),
            queries: None,
            query_rename: None,
            paths: None,
            body: None,
            fn_args: vec![],
            fn_return_type: FnReturnTypeIR::Typed(syn::parse2(quote!(!)).unwrap()),
        }
    }
}

#[derive(Debug)]
enum FnArgIR {
    Receiver,
    Typed { name: String, r#type: ArgTypeIR },
}

#[derive(Debug)]
enum ArgTypeIR {
    Query(usize),
    Path(usize),
    Body,
}

impl ArgTypeIR {
    fn as_ident(&self) -> Ident {
        match self {
            Self::Query(i) => format!("Query{i}").as_ident(),
            Self::Path(i) => format!("Path{i}").as_ident(),
            Self::Body => "Body".as_ident(),
        }
    }
}

enum FnReturnTypeIR {
    RawResponse,
    Typed(Type),
}

pub fn method(method: Method, attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parsing
    let fn_definition = syn::parse2::<ImplItemMethod>(item)
        .expect_or_abort("Method attributes should only be used for trait methods definition");
    let mut ir = MethodIR::new(method);
    parse_attr_endpoint_url(attr, &mut ir);
    parse_fn_name(&fn_definition, &mut ir);
    parse_fn_signature(&fn_definition, &mut ir);
    parse_attr_query(&fn_definition, &mut ir);
    // Codegen
    codegen_fn_impl(ir)
}

/// Parse and validate endroint url arg of attribute macro
fn parse_attr_endpoint_url(attr: TokenStream, ir: &mut MethodIR) {
    let attr_arg = syn::parse2::<LitStr>(attr).expect_or_abort("Expected string endpoint url");
    let endpoint_url = attr_arg.value();
    if !endpoint_url.starts_with('/') {
        abort!(attr_arg, "Endpoint url should start with a '/'")
    }
    ir.endpoint_url = endpoint_url
}

/// Parse and validate attribute `#[query(old = "new")]` if exists
fn parse_attr_query(fn_definition: &ImplItemMethod, ir: &mut MethodIR) {
    let fn_arg_names = fn_definition
        .sig
        .inputs
        .iter()
        .filter_map(|fn_arg| match fn_arg {
            FnArg::Typed(pat_type) => Some(pat_type.to_owned()),
            _ => None,
        })
        .filter_map(|pat_type| match *pat_type.pat {
            Pat::Ident(ident) => Some(ident.ident.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();

    for attr in &fn_definition.attrs {
        let attr_ident = attr.path.get_ident().map(|it| it.to_string());
        if let Some("query") = attr_ident.as_deref() {
            let assn = syn::parse2::<ExprParen>(attr.tokens.to_owned())
                .and_then(|paren| syn::parse2::<ExprAssign>(paren.expr.to_token_stream()))
                .unwrap_or_else(|_| {
                    abort!(
                        attr,
                        "Expected `old_name = \"new_name\"` in query attribute"
                    )
                });

            let query_ident =
                parse_assign_left_ident(&assn, || "Expected identifier (fn argument name)");
            let query_litstr =
                parse_assign_right_litstr(&assn, || "Expected new name value (string literal)");

            if !fn_arg_names.contains(&query_ident.to_string()) {
                abort!(
                    query_ident,
                    "Cannot find argument with name `{}`",
                    &query_ident
                )
            }
            if query_litstr.value().trim().is_empty() {
                abort!(query_litstr, "Invalid new name value");
            }
            ir.query_rename
                .get_or_insert_with(HashMap::new)
                .insert(query_ident.to_string(), query_litstr.value());
        }
    }
}

/// Parse method name
fn parse_fn_name(fn_definition: &ImplItemMethod, ir: &mut MethodIR) {
    ir.fn_name = fn_definition.sig.ident.to_string()
}

/// Parse and validate method signature (args and return type)
fn parse_fn_signature(fn_definition: &ImplItemMethod, ir: &mut MethodIR) {
    parse_fn_args(fn_definition, ir);
    parse_fn_return_type(fn_definition, ir);
}

/// Parse method args and validate its types
fn parse_fn_args(fn_definition: &ImplItemMethod, ir: &mut MethodIR) {
    let mut query_number = 1;
    let mut path_number = 1;

    for (i, arg) in (&fn_definition.sig.inputs).into_iter().enumerate() {
        match arg {
            FnArg::Receiver(r) => match (i == 0, &r.mutability, &r.reference) {
                (false, _, _) => abort!(r, "First parameter should be &self parameter"),
                (_, Some(_), _) => abort!(r, "Parameter &self should be immutable"),
                (_, _, None) => abort!(r, "Parameter &self should be a reference"),
                _ => ir.fn_args.push(FnArgIR::Receiver),
            },
            FnArg::Typed(pat_type) => {
                if i == 0 {
                    abort!(pat_type, "First parameter should be &self parameter");
                } else {
                    parse_typed_arg(pat_type, ir, &mut query_number, &mut path_number);
                }
            }
        }
    }
}

/// Parse and validate specific method argument
fn parse_typed_arg(
    pat_type: &PatType,
    ir: &mut MethodIR,
    query_number: &mut usize,
    path_number: &mut usize,
) {
    let arg_name = match &*pat_type.pat {
        Pat::Ident(ident) => ident.ident.to_string(),
        _ => abort!(pat_type, "Only identifier arguments are supported"),
    };
    let arg_type = match &*pat_type.ty {
        Type::Path(path) => Some(path.path.to_owned()),
        _ => None,
    }
    .and_then(|path| {
        if path.segments.len() == 1 {
            Some(path.segments[0].ident.to_owned())
        } else {
            None
        }
    })
    .map(|ident| ident.to_string());

    match arg_type.as_deref() {
        Some("Path") => {
            ir.fn_args.push(FnArgIR::Typed {
                name: arg_name.to_owned(),
                r#type: ArgTypeIR::Path(*path_number),
            });
            ir.paths.get_or_insert_with(Vec::new).push(arg_name);
            *path_number += 1;
        }
        Some("Query") => {
            ir.fn_args.push(FnArgIR::Typed {
                name: arg_name.to_owned(),
                r#type: ArgTypeIR::Query(*query_number),
            });
            ir.queries.get_or_insert_with(Vec::new).push(arg_name);
            *query_number += 1;
        }
        Some("Body") => {
            if ir.body.is_none() {
                ir.fn_args.push(FnArgIR::Typed {
                    name: arg_name.to_owned(),
                    r#type: ArgTypeIR::Body,
                });
                ir.body = Some(arg_name)
            } else {
                abort!(pat_type, "Request can have only one body");
            }
        }
        _ => abort!(
            pat_type,
            "Parameter type must be one of the following types: Path, Query, Body"
        ),
    }
}

/// Parse method return type
fn parse_fn_return_type(fn_definition: &ImplItemMethod, ir: &mut MethodIR) {
    ir.fn_return_type = match &fn_definition.sig.output {
        ReturnType::Default => FnReturnTypeIR::Typed(syn::parse2(quote!(())).unwrap()),
        ReturnType::Type(_, t) => match t.as_ref() {
            Type::Path(type_path) => {
                if type_path.path.is_ident("Response") {
                    FnReturnTypeIR::RawResponse
                } else {
                    FnReturnTypeIR::Typed(*t.to_owned())
                }
            }
            _ => FnReturnTypeIR::Typed(*t.to_owned()),
        },
    }
}

/// Generate impelmentation for the method from its IR
fn codegen_fn_impl(ir: MethodIR) -> TokenStream {
    let fn_name = ir.fn_name.as_ident();
    let (type_definitions, type_bounds) = codegen_type_bounds(&ir);
    let args = codegen_fn_args(&ir);
    let fn_return_type = match &ir.fn_return_type {
        FnReturnTypeIR::Typed(t) => quote!(#t),
        FnReturnTypeIR::RawResponse => quote!(::reqwest::Response),
    };
    let fn_code_block = codegen_fn_code_block(&ir);

    quote! {
        pub async fn #fn_name #type_definitions ( #args ) -> ::restix::Result<#fn_return_type>
        #type_bounds {
            #fn_code_block
        }
    }
}

/// Generate type definitions and bounds for the method
fn codegen_type_bounds(ir: &MethodIR) -> (TokenStream, TokenStream) {
    let mut type_definitions = vec![];
    let mut type_bounds = vec![];
    for arg in &ir.fn_args {
        if let FnArgIR::Typed { r#type, .. } = arg {
            let type_ident = r#type.as_ident();
            match r#type {
                ArgTypeIR::Query(_) => type_bounds
                    .push(quote!(#type_ident: ::core::convert::AsRef<::std::primitive::str>)),
                ArgTypeIR::Path(_) => type_bounds.push(quote!(#type_ident: ::std::fmt::Display)),
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
    let args = ir.fn_args.iter().map(|arg| match arg {
        FnArgIR::Receiver => quote!(&self),
        FnArgIR::Typed { name, r#type } => {
            let name = name.as_ident();
            let r#type = r#type.as_ident();
            quote!(#name: #r#type)
        }
    });
    quote! { #( #args ),* }
}

/// Merge codegen results into a single code block for method implementation
fn codegen_fn_code_block(ir: &MethodIR) -> TokenStream {
    let format_url = codegen_format_url(ir);
    let client_execution = codegen_client_execution(ir);
    quote! {
        #format_url
        #client_execution
    }
}

/// Generate `let full_url = format!(...)` statement
fn codegen_format_url(ir: &MethodIR) -> TokenStream {
    let paths = match &ir.paths {
        None => vec![],
        Some(paths) => paths
            .iter()
            .map(|path| {
                let key = path.unraw().as_ident();
                let value = path.as_ident();
                quote!(#key = #value)
            })
            .collect(),
    };
    let full_url = format!("{{base_url}}{}", ir.endpoint_url);
    quote! {
        let full_url = ::std::format!(
            #full_url,
            base_url = &self.base_url,
            #( #paths ),*
        );
    }
}

/// Generate client execution statement
fn codegen_client_execution(ir: &MethodIR) -> TokenStream {
    let method = match &ir.method {
        Method::Get => quote!(::restix::Method::Get),
        Method::Post => quote!(::restix::Method::Post),
    };
    let queries = match &ir.queries {
        None => vec![],
        Some(queries) => queries
            .iter()
            .map(|query| {
                let renaming = ir.query_rename.as_ref().and_then(|map| map.get(query));
                let key = match &renaming {
                    Some(new_query) => new_query.unraw(),
                    None => query.unraw(),
                };
                let ident = query.as_ident();
                quote! { (#key, #ident.as_ref()) }
            })
            .collect(),
    };
    let queries = if queries.is_empty() {
        quote! { ::std::option::Option::None }
    } else {
        quote! { ::std::option::Option::Some(::std::vec![#( #queries ),*]) }
    };
    let body = if let Some(body) = &ir.body {
        let ident = body.as_ident();
        quote! { ::std::option::Option::Some(#ident) }
    } else {
        quote! { ::std::option::Option::<()>::None }
    };
    let execute_fn: Ident = match &ir.fn_return_type {
        FnReturnTypeIR::Typed(_) => syn::parse_quote!(execute_with_serde),
        FnReturnTypeIR::RawResponse => syn::parse_quote!(execute_raw),
    };

    quote! {
        self.client.#execute_fn(
            #method,
            &full_url,
            #queries,
            #body,
        ).await
    }
}
