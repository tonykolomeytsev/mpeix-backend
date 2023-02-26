use std::collections::HashMap;

use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{
    Attribute, Block, ExprAssign, ExprParen, FnArg, ImplItemMethod, ItemFn, LitStr, Pat, Path,
    Receiver, Signature, TraitItemMethod, Type,
};

use crate::query::{query_key, query_value};

pub fn method(
    method: &str,
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let endpoint_url = get_endpoint_url(attr);
    let fn_sig = get_fn_signature(&item);
    let args = get_arguments(&fn_sig);
    let fn_attrs = get_fn_attributes(&item);
    let query_rename_rules = get_query_rename_rules(fn_attrs);

    let mut method_body = syn::parse::<ImplItemMethod>(item).unwrap();
    method_body.block = create_fn_block(method, &endpoint_url, args, &query_rename_rules);

    quote! {
        #method_body
    }
    .into()
}

fn get_endpoint_url(attr: proc_macro::TokenStream) -> String {
    let endpoint_url = syn::parse::<LitStr>(attr).expect("Expected url string literal");
    let endpoint_url = endpoint_url.value();
    if !endpoint_url.starts_with("/") {
        panic!("Url part should start with a '/'");
    }
    endpoint_url
}

fn get_fn_signature(item: &proc_macro::TokenStream) -> Signature {
    syn::parse::<ItemFn>(item.clone())
        .map(|it| it.sig)
        .or_else(|_| syn::parse::<TraitItemMethod>(item.clone()).map(|it| it.sig))
        .expect("Cannot get method signature. Maybe you use this macro attr in wrong context")
}

fn get_fn_attributes(item: &proc_macro::TokenStream) -> Vec<Attribute> {
    syn::parse::<ItemFn>(item.clone())
        .map(|it| it.attrs)
        .or_else(|_| syn::parse::<TraitItemMethod>(item.clone()).map(|it| it.attrs))
        .expect("Cannot get methodattributes. Maybe you use this macro attr in wrong context")
}

struct Args {
    paths: Vec<String>,
    queries: Vec<String>,
    body: Option<String>,
}

fn get_arguments(sig: &Signature) -> Args {
    let mut i: usize = 0;
    let mut args = Args {
        paths: vec![],
        queries: vec![],
        body: None,
    };
    // inspect arg types
    for arg in &sig.inputs {
        match arg {
            FnArg::Receiver(receiver) => inspect_receiver(receiver, &i),
            FnArg::Typed(pat_type) => {
                assert!(i > 0, "&self should be the first paramter");
                let arg_name = if let Pat::Ident(ident) = &*pat_type.pat {
                    ident.ident.to_string()
                } else {
                    panic!("Only identifier arguments are supported");
                };
                if let Type::Path(path) = &*pat_type.ty {
                    inspect_type(&path.path, arg_name, &mut args);
                } else {
                    panic!(
                        "Argument `{arg_name}` has unsupported type.\nMust be one of the following types: Path, Query, Body"
                    )
                }
            }
        }
        i += 1;
    }
    args
}

fn inspect_receiver(receiver: &Receiver, i: &usize) {
    assert!(i == &0, "&self should be the first paramter");
    assert!(
        receiver.mutability.is_none(),
        "Parameter &self should be immutable"
    );
    assert!(
        receiver.reference.is_some(),
        "Parameter &self should be a reference"
    );
}

fn inspect_type(path: &Path, arg_name: String, args: &mut Args) {
    let ident =
        if path.segments.len() == 2 && path.segments[0].ident.to_string() == "common_api_macro" {
            Some(path.segments[1].ident.to_owned())
        } else if path.segments.len() == 1 {
            Some(path.segments[0].ident.to_owned())
        } else {
            None
        };
    match ident.map(|it| it.to_string()).as_deref() {
        Some("Path") => args.paths.push(arg_name),
        Some("Query") => args.queries.push(arg_name),
        Some("Body") => {
            if args.body.is_none() {
                args.body = Some(arg_name)
            } else {
                panic!("Request can have only one body");
            }
        }
        _ => panic!(
            "Argument `{arg_name}` has unsupported type.\nMust be one of the following types: Path, Query, Body"
        )
    }
}

fn get_query_rename_rules(attrs: Vec<Attribute>) -> HashMap<String, String> {
    let mut rename_rules = HashMap::<String, String>::new();
    for attr in attrs {
        let attr_ident = attr.path.get_ident().map(|it| it.to_string());
        if let Some("query") = attr_ident.as_deref() {
            let assn = syn::parse2::<ExprParen>(attr.tokens)
                .expect("Expected `key = \"value\"` in query attribute");
            let assn = syn::parse2::<ExprAssign>(assn.expr.to_token_stream())
                .expect("Expected `key = \"value\"` in query attribute");
            let left = query_key(&assn);
            let right = query_value(&assn);
            rename_rules.insert(left, right);
        }
    }
    rename_rules
}

fn create_fn_block(
    method: &str,
    endpoint_url: &str,
    args: Args,
    query_rename_rules: &HashMap<String, String>,
) -> Block {
    // Create path substitutions for further use in `format!()` macro
    let paths = args
        .paths
        .into_iter()
        .map(|it| {
            let key = Ident::new(it.unraw(), Span::call_site());
            let value = ident(&it);
            quote! { #key=#value }
        })
        .collect::<Vec<_>>();

    // Create query substitutions for further use as `client.execute` argument
    let queries = if args.queries.is_empty() {
        quote! { ::std::option::Option::None }
    } else {
        let queries = args
            .queries
            .into_iter()
            .map(|it| {
                let key = if let Some(key) = query_rename_rules.get(&it) {
                    LitStr::new(key.unraw(), Span::call_site())
                } else {
                    LitStr::new(&it.unraw(), Span::call_site())
                };
                let value = ident(&it);
                quote! { (#key, #value.as_ref()) }
            })
            .collect::<Vec<_>>();
        quote! { ::std::option::Option::Some(::std::vec![#( #queries ),*]) }
    };

    let full_url_lit = LitStr::new(&format!("{{base_url}}{endpoint_url}"), Span::call_site());
    let method = LitStr::new(method, Span::call_site());
    let body = if let Some(body) = args.body {
        let ident = ident(&body);
        quote! { ::std::option::Option::Some(#ident) }
    } else {
        quote! { ::std::option::Option::<::common_api_macro::Body<()>>::None }
    };

    let block = quote! {
        fn stub()
        {
            let full_url = ::std::format!(
                #full_url_lit,
                base_url = &self.base_url,
                #( #paths ),*
            );

            self.client.execute(
                #method,
                &full_url,
                #queries,
                #body,
            ).await
        }
    };
    *syn::parse2::<ItemFn>(block)
        .expect("Statement expected")
        .block
}

fn ident(name: &str) -> Ident {
    if name.starts_with("r#") {
        Ident::new_raw(name.unraw(), Span::call_site())
    } else {
        Ident::new(name, Span::call_site())
    }
}

trait Unraw {
    fn unraw(&self) -> &str;
}

impl<S: AsRef<str>> Unraw for S {
    fn unraw(&self) -> &str {
        self.as_ref().trim_start_matches("r#")
    }
}
