#![doc = include_str!("../README.md")]

pub use restix_impl::*;
pub use restix_macro::*;

#[cfg(all(not(feature = "reqwest"), feature = "json"))]
compile_error!(r#"The "reqwest" feature must be enabled if the "json" feature is enabled"#);

#[cfg(all(not(feature = "json"), not(feature = "reqwest")))]
compile_error!(
    r#"At least one "reqwest" feature must be enabled in order to use the restix library"#
);

pub trait AsQuery<T> {
    fn push_to_vec<'a>(&self, key: &'a str, vec: &mut std::vec::Vec<(&'a str, String)>);
}

impl<T> AsQuery<T> for T
where
    T: std::fmt::Display,
{
    fn push_to_vec<'a>(&self, key: &'a str, vec: &mut std::vec::Vec<(&'a str, String)>) {
        vec.push((key, format!("{self}")));
    }
}

impl<T> AsQuery<T> for Option<T>
where
    T: std::fmt::Display,
{
    fn push_to_vec<'a>(&self, key: &'a str, vec: &mut std::vec::Vec<(&'a str, String)>) {
        if let Some(value) = self {
            value.push_to_vec(key, vec);
        }
    }
}
