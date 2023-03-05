use proc_macro2::{Ident, Span};
use proc_macro_error::ResultExt;

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
