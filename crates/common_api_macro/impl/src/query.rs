use syn::{ExprAssign, ItemFn, PatType, TraitItemMethod};

pub fn query(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let assn = syn::parse_macro_input!(attr as ExprAssign);
    let left = query_key(&assn);
    let right = query_value(&assn);
    let _ = right;
    let fn_inputs = syn::parse::<ItemFn>(item.clone())
        .map(|it| it.sig.inputs)
        .or_else(|_| syn::parse::<TraitItemMethod>(item.clone()).map(|it| it.sig.inputs))
        .expect("Method inputs");

    for fn_arg in fn_inputs.into_iter() {
        match fn_arg {
            syn::FnArg::Receiver(_) => continue,
            syn::FnArg::Typed(PatType { pat, .. }) => match &*pat {
                syn::Pat::Ident(ident) => {
                    if ident.ident.to_string() == left {
                        return item;
                    }
                }
                _ => continue,
            },
        }
    }
    panic!("Cannot find argument with name `{}`", left);
}

pub(crate) fn query_key(assn: &ExprAssign) -> String {
    match assn.left.as_ref() {
        syn::Expr::Path(p) => {
            let segments = &p.path.segments;
            if segments.len() == 1 {
                segments.into_iter().nth(0).unwrap().ident.to_string()
            } else {
                panic!("Left part of query attribute should be identifier")
            }
        }
        _ => panic!("Left part of query attribute should be identifier"),
    }
}

pub(crate) fn query_value(assn: &ExprAssign) -> String {
    match assn.right.as_ref() {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) => s.value(),
            _ => panic!("Right part of query attribute should be identifier string literal"),
        },
        _ => panic!("Right part of query attribute should be identifier string literal"),
    }
}
