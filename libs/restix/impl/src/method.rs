use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{quote, ToTokens};
use syn::{
    parse::Parse, spanned::Spanned, Attribute, ExprParen, FnArg, ImplItemMethod, LitStr, PatType,
    ReturnType, Type, TypePath,
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
    MapResponseWith(AttrMapResponseWithIR),
}

struct AttrMapResponseWithIR {
    mapper: TypePath,
}

enum ArgIR {
    Receiver,
    Typed {
        name: Ident,
        r#type: Type,
        kind: ArgKindIR,
    },
}

#[derive(Default)]
struct ArgsCounter {
    common: usize,
}

enum ArgKindIR {
    Query(Option<Ident>),
    Path(Option<Ident>),
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
            Some("map_response_with") => AttrIR::MapResponseWith(syn::parse2(attr.tokens)?),
            _ => return Err(syn::Error::new(attr.span(), "Unknown attribute")),
        },
    )
}

impl Parse for AttrMapResponseWithIR {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mapper: TypePath = syn::parse2(input.parse::<ExprParen>()?.expr.to_token_stream())?;
        Ok(AttrMapResponseWithIR { mapper })
    }
}

fn parse_arg_ir(fn_arg: FnArg, counter: &mut ArgsCounter) -> syn::Result<ArgIR> {
    counter.common += 1;
    match &fn_arg {
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
                name: syn::parse2(pat_type.pat.to_token_stream())?,
                r#type: syn::parse2(pat_type.ty.to_token_stream())?,
                kind: parse_arg_kind_ir(pat_type)?,
            })
        }
    }
}

fn parse_arg_kind_ir(pat_type: &PatType) -> syn::Result<ArgKindIR> {
    let mut iter = pat_type.attrs.iter();
    let arg_kind = if let Some(attr) = iter.next() {
        let alt_name = if attr.tokens.is_empty() {
            None
        } else {
            let expr_paren = syn::parse2::<ExprParen>(attr.tokens.to_owned())?;
            Some(
                syn::parse2::<LitStr>(expr_paren.expr.into_token_stream())?
                    .value()
                    .as_ident(),
            )
        };
        match attr.path.get_ident().map(ToString::to_string).as_deref() {
            Some("path") => ArgKindIR::Path(alt_name),
            Some("query") => ArgKindIR::Query(alt_name),
            Some("body") => ArgKindIR::Body,
            _ => {
                return Err(syn::Error::new(
                    attr.path.span(),
                    "Unsupported attribute. Must be one of: `path`, `query`, `body`",
                ))
            }
        }
    } else {
        return Err(syn::Error::new(
            pat_type.span(),
            "Each argument must have attribute `#[path]`, `#[query]`, or #[body]",
        ));
    };
    if let Some(attr) = iter.next() {
        return Err(syn::Error::new(
            attr.span(),
            "An argument cannot have more than one attribute",
        ));
    }
    Ok(arg_kind)
}

impl ArgIR {
    fn as_query(&self) -> Option<(&Ident, &Ident)> {
        match self {
            Self::Typed {
                name,
                kind: ArgKindIR::Query(alt_name),
                ..
            } => Some((name, alt_name.as_ref().unwrap_or(name))),
            _ => None,
        }
    }

    fn as_path(&self) -> Option<(&Ident, &Ident)> {
        match self {
            Self::Typed {
                name,
                kind: ArgKindIR::Path(alt_name),
                ..
            } => Some((name, alt_name.as_ref().unwrap_or(name))),
            _ => None,
        }
    }

    fn as_body(&self) -> Option<&Ident> {
        match self {
            Self::Typed {
                name,
                kind: ArgKindIR::Body,
                ..
            } => Some(name),
            _ => None,
        }
    }
}

impl Parse for ReturnTypeIR {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let return_type: ReturnType = input.parse()?;
        Ok(match return_type {
            ReturnType::Default => ReturnTypeIR::RawResponse,
            ReturnType::Type(_, t) => ReturnTypeIR::Typed(*t),
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

pub fn method(method: Method, attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parsing
    let ir: MethodIR = syn::parse2(item).unwrap_or_abort();
    let endpoint_url = parse_attr_endpoint_url(attr);
    analyze_method_ir(&ir);
    // Codegen
    codegen_fn_impl(ir, &endpoint_url, method)
}

fn analyze_method_ir(ir: &MethodIR) {
    let body_args = ir
        .args
        .iter()
        .filter_map(ArgIR::as_body)
        .collect::<Vec<_>>();
    if body_args.len() > 1 {
        abort!(body_args[1], "Only one body argument is allowed");
    }
}

/// Generate impelmentation for the method from its IR
fn codegen_fn_impl(ir: MethodIR, endpoint_url: &str, method: Method) -> TokenStream {
    let name = &ir.name;
    let args = codegen_fn_args(&ir);
    let method_return_type = method_return_type(&ir);
    let fn_code_block = codegen_client_execution(&ir, endpoint_url, method);
    let client_result_type = client_result_type();

    quote! {
        pub async fn #name ( #args ) -> #client_result_type<#method_return_type>
        {
            #fn_code_block
        }
    }
}

fn method_return_type(ir: &MethodIR) -> TokenStream {
    match &ir.return_type {
        ReturnTypeIR::Typed(t) => quote!(#t),
        ReturnTypeIR::RawResponse => client_response_type(),
    }
}

#[cfg(feature = "reqwest")]
fn client_result_type() -> TokenStream {
    quote!(::reqwest::Result)
}

#[cfg(feature = "reqwest")]
fn client_response_type() -> TokenStream {
    quote!(::reqwest::Response)
}

/// Restore method args from IR
fn codegen_fn_args(ir: &MethodIR) -> TokenStream {
    let args = ir.args.iter().map(|arg| match arg {
        ArgIR::Receiver => quote!(&self),
        ArgIR::Typed { name, r#type, .. } => quote!(#name: #r#type),
    });
    quote! { #( #args ),* }
}

/// Generate client execution statement
#[cfg(feature = "reqwest")]
fn codegen_client_execution(ir: &MethodIR, endpoint_url: &str, method: Method) -> TokenStream {
    let format_url = codegen_format_url(ir, endpoint_url);
    let method_call: Ident = match method {
        Method::Get => syn::parse_quote!(get),
        Method::Post => syn::parse_quote!(post),
    };
    let queries = codegen_queries(ir);
    let body_call = if let Some(body) = ir.args.iter().find_map(ArgIR::as_body) {
        quote!(.body(#body))
    } else {
        quote!()
    };
    let deserialize_and_return = codegen_deserialize_and_return(ir);

    quote! {
        #format_url
        #queries

        let response = self.client
            .#method_call(&full_url)
            .query(&queries)
            #body_call
            .send()
            .await?;
        #deserialize_and_return
    }
}

/// Generate `let full_url = format!(...)` statement
fn codegen_format_url(ir: &MethodIR, endpoint_url: &str) -> TokenStream {
    let paths = &ir
        .args
        .iter()
        .filter_map(ArgIR::as_path)
        .map(|(name, alt_name)| {
            let key = alt_name.to_string().unraw().as_ident();
            quote!(#key = #name)
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

fn codegen_queries(ir: &MethodIR) -> TokenStream {
    let queries = &ir
        .args
        .iter()
        .filter_map(ArgIR::as_query)
        .map(|(name, alt_name)| {
            let key = alt_name.to_string();
            let key = key.unraw();
            quote! {
                #name.push_to_vec(#key, &mut queries);
            }
        })
        .collect::<Vec<_>>();
    let query_len = queries.len();

    quote! {
        use ::restix::AsQuery;
        let mut queries = ::std::vec::Vec::<(&::std::primitive::str, ::std::string::String)>::with_capacity(#query_len);
        #( #queries )*
    }
}

#[cfg(all(feature = "reqwest", feature = "json"))]
fn codegen_deserialize_and_return(ir: &MethodIR) -> TokenStream {
    let mapper = ir
        .attrs
        .iter()
        .map(|attr| match attr {
            AttrIR::MapResponseWith(AttrMapResponseWithIR { mapper }) => Some(quote!(#mapper)),
        })
        .next();
    match (mapper, &ir.return_type) {
        (Some(mapper), ReturnTypeIR::RawResponse) => {
            quote!(::std::result::Result::Ok(#mapper(response)))
        }
        (Some(mapper), _) => quote!(::std::result::Result::Ok(#mapper(response.json().await?))),
        (None, ReturnTypeIR::RawResponse) => quote!(::std::result::Result::Ok(response)),
        (None, _) => {
            let return_type = method_return_type(ir);
            quote!(response.json::<#return_type>().await)
        }
    }
}

#[cfg(all(feature = "reqwest", not(feature = "json")))]
fn codegen_deserialize_and_return(ir: &MethodIR) -> TokenStream {
    let mapper = ir
        .attrs
        .iter()
        .map(|attr| match attr {
            AttrIR::MapResponseWith(AttrMapResponseWithIR { mapper }) => Some(quote!(#mapper)),
        })
        .next();
    if let Some(mapper) = mapper {
        quote!(Ok(#mapper(response)))
    } else {
        quote!(response)
    }
}
