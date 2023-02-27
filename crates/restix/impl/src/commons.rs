use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, ResultExt};
use syn::{Expr, ExprAssign, Lit, LitStr};

pub(crate) fn parse_assign_left_ident<'a, F: FnOnce() -> &'a str>(
    assn: &'a ExprAssign,
    err_msg: F,
) -> &'a Ident {
    match assn.left.as_ref() {
        Expr::Path(expr) => Some(expr),
        _ => None,
    }
    .and_then(|expr| expr.path.get_ident())
    .unwrap_or_else(|| abort!(assn.left, err_msg()))
}

pub(crate) fn parse_assign_right_litstr<'a, F: FnOnce() -> &'a str>(
    assn: &'a ExprAssign,
    err_msg: F,
) -> &'a LitStr {
    match assn.right.as_ref() {
        Expr::Lit(expr) => Some(expr),
        _ => None,
    }
    .and_then(|expr| match &expr.lit {
        Lit::Str(s) => Some(s),
        _ => None,
    })
    .unwrap_or_else(|| abort!(assn.right, err_msg()))
}

pub trait StringExt {
    fn as_ident(&self) -> Ident;
    fn unraw(&self) -> &str;
}

impl<S: AsRef<str>> StringExt for S {
    fn as_ident(&self) -> Ident {
        if self.as_ref().starts_with("r#") {
            syn::parse_str(self.as_ref()).expect_or_abort(&format!(
                "Something went wrong when parsing identifier `{}`",
                self.as_ref()
            ))
        } else {
            Ident::new(self.as_ref(), Span::call_site())
        }
    }

    fn unraw(&self) -> &str {
        self.as_ref().trim_start_matches("r#")
    }
}
