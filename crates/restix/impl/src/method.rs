use std::{collections::HashMap, fmt::Debug};

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{quote, ToTokens};
use syn::{
    parse::Parse, ExprAssign, ExprParen, FnArg, ImplItemMethod, LitStr, Pat, PatType, ReturnType,
    Type,
};

use crate::{commons::StringExt, Method};

/// Intermediate representation of an Method definition.
struct MethodIR {
    method: Method,
    fn_name: String,
    endpoint_url: String,
    query_rename: HashMap<String, String>,
    fn_args: Vec<FnArgIR>,
    fn_return_type: FnReturnTypeIR,
}

impl MethodIR {
    fn new(method: Method) -> Self {
        Self {
            method,
            fn_name: String::new(),
            endpoint_url: String::new(),
            query_rename: HashMap::with_capacity(0),
            fn_args: Vec::with_capacity(0),
            fn_return_type: FnReturnTypeIR::Typed(syn::parse2(quote!(!)).unwrap()),
        }
    }
}

#[derive(Debug)]
enum FnArgIR {
    Receiver,
    Typed {
        name: String,
        r#type: ArgTypeIR,
        optional: bool,
    },
}

macro_rules! fn_arg_as {
    ($n:ident -> $p:pat) => {
        fn $n(&self) -> Option<&String> {
            match self {
                Self::Typed {
                    name, r#type: $p, ..
                } => Some(name),
                _ => None,
            }
        }
    };
}

impl FnArgIR {
    fn path(name: String, num: usize) -> FnArgIR {
        FnArgIR::Typed {
            name,
            r#type: ArgTypeIR::Path(num),
            optional: false,
        }
    }

    fn query(name: String, num: usize, optional: bool) -> FnArgIR {
        FnArgIR::Typed {
            name,
            r#type: ArgTypeIR::Query(num),
            optional,
        }
    }

    fn body(name: String) -> FnArgIR {
        FnArgIR::Typed {
            name,
            r#type: ArgTypeIR::Body,
            optional: false,
        }
    }

    fn is_optional(&self) -> bool {
        matches!(self, FnArgIR::Typed { optional: true, .. })
    }

    fn_arg_as!(as_path -> ArgTypeIR::Path(_));
    fn_arg_as!(as_query -> ArgTypeIR::Query(_));
    fn_arg_as!(as_body -> ArgTypeIR::Body);
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

struct AttrQueryIR {
    old_name: Ident,
    new_name: LitStr,
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
            let attr_query_ir: AttrQueryIR = syn::parse2(attr.tokens.to_owned()).unwrap_or_abort();
            let query_ident = attr_query_ir.old_name;
            let query_litstr = attr_query_ir.new_name;

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
        Type::Path(type_path) => type_path.to_owned(),
        _ => abort!(
            pat_type,
            "Parameter type must be one of the following types: Path, Query, Option<Query>, Body"
        ),
    };

    if arg_type == syn::parse_quote!(Path) {
        ir.fn_args.push(FnArgIR::path(arg_name, *path_number));
        *path_number += 1;
    } else if arg_type == syn::parse_quote!(Query) {
        ir.fn_args
            .push(FnArgIR::query(arg_name, *query_number, false));
        *query_number += 1;
    } else if arg_type == syn::parse_quote!(Option<Query>) {
        ir.fn_args
            .push(FnArgIR::query(arg_name, *query_number, true));
        *query_number += 1;
    } else if arg_type == syn::parse_quote!(Body) {
        match ir.fn_args.iter().find_map(FnArgIR::as_body) {
            Some(_) => abort!(pat_type, "Request can have only one body"),
            None => ir.fn_args.push(FnArgIR::body(arg_name)),
        }
    } else {
        abort!(
            pat_type,
            "Parameter type must be one of the following types: Path, Query, Option<Query>, Body"
        )
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
        FnArgIR::Typed {
            name,
            r#type,
            optional,
        } => {
            let name = name.as_ident();
            let r#type = r#type.as_ident();
            if *optional {
                quote!(#name: ::std::option::Option<#r#type>)
            } else {
                quote!(#name: #r#type)
            }
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
    let paths = &ir
        .fn_args
        .iter()
        .filter_map(FnArgIR::as_path)
        .map(|path| {
            let key = path.unraw().as_ident();
            let value = path.as_ident();
            quote!(#key = #value)
        })
        .collect::<Vec<_>>();
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
    let queries = &ir
        .fn_args
        .iter()
        .filter(|arg| !arg.is_optional())
        .filter_map(FnArgIR::as_query)
        .map(|query| {
            let renaming = ir.query_rename.get(query);
            let key = match &renaming {
                Some(new_query) => new_query.unraw(),
                None => query.unraw(),
            };
            let ident = query.as_ident();
            quote! { (#key, #ident.as_ref()) }
        })
        .collect::<Vec<_>>();

    let opt_queries = &ir
        .fn_args
        .iter()
        .filter(|arg| arg.is_optional())
        .filter_map(FnArgIR::as_query)
        .map(|query| {
            let renaming = ir.query_rename.get(query);
            let key = match &renaming {
                Some(new_query) => new_query.unraw(),
                None => query.unraw(),
            };
            let ident = query.as_ident();
            quote! {
                if let ::std::option::Option::Some(q) = #ident.as_ref() {
                    queries.push( (#key, q.as_ref()) );
                }
            }
        })
        .collect::<Vec<_>>();

    let body = if let Some(body) = ir.fn_args.iter().find_map(FnArgIR::as_body) {
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
        let mut queries = ::std::vec![#( #queries ),*];

        #( #opt_queries )*

        self.client.#execute_fn(
            #method,
            &full_url,
            queries,
            #body,
        ).await
    }
}
