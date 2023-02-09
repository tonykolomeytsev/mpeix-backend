pub mod env {
    use std::str::FromStr;

    pub fn get(key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    pub fn get_or<S: AsRef<str>>(key: &str, default: S) -> String {
        std::env::var(key).unwrap_or_else(|_| default.as_ref().to_owned())
    }

    pub fn get_parsed<T: FromStr>(key: &str) -> Option<T> {
        match std::env::var(key) {
            Ok(var) => var.parse::<T>().ok(),
            Err(_) => None,
        }
    }

    pub fn get_parsed_or<T: FromStr>(key: &str, default: T) -> T {
        match std::env::var(key) {
            Ok(var) => var.parse::<T>().unwrap_or(default),
            Err(_) => default,
        }
    }
}
