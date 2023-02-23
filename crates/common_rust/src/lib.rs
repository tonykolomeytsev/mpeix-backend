pub mod env {
    use std::str::FromStr;

    /// Get environment variable value if exists.
    /// In case of errors return [Option::None].
    pub fn get(key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    /// Get environment variable value if exists.
    /// In case of errors return `default`.
    pub fn get_or<S: AsRef<str>>(key: &str, default: S) -> String {
        std::env::var(key).unwrap_or_else(|_| default.as_ref().to_owned())
    }

    /// Get environment variable value if exists, then parse it to necessary type.
    /// In case of errors return [Option::None].
    pub fn get_parsed<T: FromStr>(key: &str) -> Option<T> {
        match std::env::var(key) {
            Ok(var) => var.parse::<T>().ok(),
            Err(_) => None,
        }
    }

    /// Get environment variable value if exists, then parse it to necessary type.
    /// In case of errors return `default`.
    pub fn get_parsed_or<T: FromStr>(key: &str, default: T) -> T {
        match std::env::var(key) {
            Ok(var) => var.parse::<T>().unwrap_or(default),
            Err(_) => default,
        }
    }

    /// Get environment variable value if exists,
    /// or panic with readable description.
    pub fn required(key: &str) -> String {
        std::env::var(key).expect(&format!("Environment variable {key} not provided"))
    }
}
