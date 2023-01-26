use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

/// # PersistentCache
///
/// Persistent cache implementation based on storing records in files.
/// This implementation also uses tokio runtime (with "fs" feature)
/// to prevent reading files from blocking other tasks.
pub struct PersistentCache {
    cache_dir: PathBuf,
}

/// The error type for persistent cache `insert`/`get` operations
pub enum Error {
    IOError(std::io::Error),
    DeserializationError(serde_json::Error),
}

impl PersistentCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Insert value into the cache
    ///
    /// Returns `IOError` if an error occurs while working with the file system:
    /// - [tokio::fs::create_dir_all]
    /// - [tokio::fs::File] (create)
    /// - [tokio::io::util::AsyncWriteExt::write_all]
    ///
    /// Panics if cannot serialize `value` (see [serde_json::to_string]).
    ///
    /// **Note:** This method creates all directories from `key`, if don't exist.
    pub async fn insert<K, V>(&mut self, key: K, value: V) -> Result<(), Error>
    where
        K: AsRef<Path>,
        V: Serialize,
    {
        let cache_entry_path = self.cache_dir.join(key);
        if let Some(parent_dir_path) = cache_entry_path.parent() {
            if !parent_dir_path.exists() {
                tokio::fs::create_dir_all(parent_dir_path).await?;
            }
        }
        let mut file = File::create(cache_entry_path).await?;
        let serialized_value =
            serde_json::to_string(&value).expect("Error while serializing internal model");
        file.write_all(serialized_value.as_bytes()).await?;
        Ok(())
    }

    /// Get value from the cache
    ///
    /// Returns `IOError` if an error occurs while working with the file system:
    /// - [tokio::fs::File] (open)
    /// - [tokio::io::util::AsyncReadExt::read_to_string]
    ///
    /// Returns `DeserializationError` if [serde_json::from_str] cannot get its work done.
    pub async fn get<K, V>(&mut self, key: K) -> Result<Option<V>, Error>
    where
        K: AsRef<Path>,
        V: DeserializeOwned,
    {
        let cache_entry_path = self.cache_dir.join(key);
        if !cache_entry_path.exists() {
            return Ok(None);
        }
        let mut file = File::open(cache_entry_path).await?;
        let mut serialized_value = String::with_capacity(8192);
        file.read_to_string(&mut serialized_value).await?;
        let deserialized_value: V = serde_json::from_str(&serialized_value)?;
        Ok(Some(deserialized_value))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IOError(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::DeserializationError(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(e) => writeln!(f, "Persistent cache IO error: {}", e),
            Error::DeserializationError(e) => {
                writeln!(f, "Persistent cache deserialization error: {}", e)
            }
        }
    }
}
