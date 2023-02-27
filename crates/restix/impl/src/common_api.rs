use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    token::Async, Block, Expr, ExprAssign, ExprLit, ExprPath, ImplItem, ImplItemMethod, ItemImpl,
    ItemStruct, ItemTrait, Lit, LitStr, Signature, Visibility,
};

pub fn common_api(attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_def = syn::parse2::<ItemTrait>(item).unwrap();
    let struct_def = create_struct_def(&trait_def);
    let base_url = get_base_url_if_exists(attr);
    let impl_trait_for_struct = create_impl_struct_stubs(&trait_def, &struct_def);
    let impl_constructor = create_impl_constructor(&trait_def, base_url);

    quote! {
        #struct_def
        #impl_constructor
        #impl_trait_for_struct
    }
    .into()
}

/// Create `struct SomeApi { ... }` with feature dependent `common_api_macro::HttpClient`.
fn create_struct_def(trait_def: &ItemTrait) -> ItemStruct {
    let struct_ident = &trait_def.ident;
    let struct_def = quote! {
        pub struct #struct_ident {
            client: ::restix::HttpClient,
            base_url: ::std::string::String,
        }
    };
    syn::parse::<ItemStruct>(struct_def.into()).unwrap()
}

/// Create `impl SomeApi { ... }` block and copy here methods from the trait.
fn create_impl_struct_stubs(trait_def: &ItemTrait, struct_def: &ItemStruct) -> ItemImpl {
    let struct_ident = &struct_def.ident;
    let methods = create_stub_methods(&trait_def);
    let implementation = quote! {
        impl #struct_ident {
            #( #methods )*
        }
    };
    syn::parse::<ItemImpl>(implementation.into()).unwrap()
}

/// Copy methods from trait with the same signature, but make them `pub async`.
/// Copied methods has empty implementation with `todo!()` call for further processsing
/// with `get`/`post`/etc.. macro attributes.
fn create_stub_methods(trait_def: &ItemTrait) -> Vec<ImplItem> {
    let tobo_block = syn::parse2::<Block>(quote! { { todo!() } }).unwrap();
    let pub_vis = syn::parse2::<Visibility>(quote! { pub }).unwrap();

    trait_def
        .items
        .iter()
        .filter_map(|it| match it {
            syn::TraitItem::Method(m) => Some(m),
            _ => None,
        })
        .map(|method| {
            let func = ImplItem::Method(ImplItemMethod {
                attrs: method.attrs.to_owned(),
                vis: pub_vis.to_owned(),
                defaultness: None,
                sig: Signature {
                    asyncness: Some(Async {
                        span: Span::call_site(),
                    }),
                    ..method.sig.to_owned()
                },
                block: tobo_block.to_owned(),
            });
            func
        })
        .collect::<Vec<ImplItem>>()
}

/// Create constructor for the `*Api` struct.
///
/// If argument `base_url` is specified in macro attribute `#[common_api]`,
/// then the argument does not need to be passed to the constructor `*Api::new()`.
fn create_impl_constructor(trait_def: &ItemTrait, base_url: Option<String>) -> ItemImpl {
    let struct_ident = &trait_def.ident;
    let constructor = if let Some(base_url) = base_url {
        // create constructor without `base_url` argument
        let base_url = LitStr::new(&base_url, Span::call_site());
        syn::parse2::<ImplItemMethod>(quote! {
            pub fn new(client: ::restix::HttpClient) -> #struct_ident {
                return #struct_ident {
                    client,
                    base_url: #base_url.to_owned(),
                }
            }
        })
    } else {
        // create constructor with `base_url` argument
        syn::parse2::<ImplItemMethod>(quote! {
            pub fn new(
                client: ::common_api_macro::HttpClient,
                base_url: ::std::string::String,
            ) -> #struct_ident {
                return #struct_ident {
                    client,
                    base_url,
                }
            }
        })
    }
    .unwrap();

    let implementation = quote! {
        impl #struct_ident {
            #constructor
        }
    };

    syn::parse2::<ItemImpl>(implementation).unwrap()
}

/// Check for `base_url` argument in the macro: `#[common_api(base_url = "...")]`
/// If `base_url` exists, pass it as argument to the struct constructor.
fn get_base_url_if_exists(attr: TokenStream) -> Option<String> {
    if attr.is_empty() {
        return None;
    }
    let assignment = syn::parse2::<ExprAssign>(attr).expect("Expected 'base_url = \"...\"'");
    if let Expr::Path(ExprPath { path, .. }) = *assignment.left {
        if path.is_ident("base_url") {
            if let Expr::Lit(ExprLit { lit, .. }) = *assignment.right {
                if let Lit::Str(lit_str) = lit {
                    let base_url = lit_str.token().to_string();
                    let base_url = base_url.trim_matches('"');
                    if base_url.ends_with("/") {
                        panic!("base_url shouldn't end with a '/'");
                    }
                    return Some(base_url.to_owned());
                }
            }
        }
    }
    panic!("Expected 'base_url = \"...\"'");
}
