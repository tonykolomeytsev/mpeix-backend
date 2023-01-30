use std::hash::Hash;
use std::num::NonZeroUsize;

use chrono::{DateTime, Duration, Local};
use lru::LruCache;
use serde::{Deserialize, Serialize};

/// # InMemoryCache
///
/// In-Memory Cache implementation based on LRU (last recent used) cache.
///
/// Supports expiration policies:
/// - By creation time:
///   ```ignore
///   let mut cache = InMemoryCache::with_capacity(1000)
///       .expires_after_creation(Duration::hours(1));
///   ```
/// - By last access time:
///   ```ignore
///   let mut cache = InMemoryCache::with_capacity(3000)
///       .expires_after_access(Duration::minutes(5));
///   ```
/// - By number of hits (number of accesses):
///   ```ignore
///   let mut cache = InMemoryCache::with_capacity(500)
///       .max_hits(10);
///   ```
///
/// ### Example:
/// ```rust
/// use common_in_memory_cache::InMemoryCache;
///
/// let mut cache = InMemoryCache::with_capacity(3);
/// cache.insert(1, "Lorem");
/// cache.insert(2, "Ipsum");
/// cache.insert(3, "Dolor");
/// cache.insert(4, "Sit");
/// cache.insert(5, "Amet");
/// assert_eq!(cache.get(&1), None);
/// assert_eq!(cache.get(&2), None);
/// assert_eq!(cache.get(&3), Some(&"Dolor"));
/// assert_eq!(cache.get(&4), Some(&"Sit"));
/// assert_eq!(cache.get(&5), Some(&"Amet"));
/// ```
pub struct InMemoryCache<K: Eq + Hash, V> {
    entries: LruCache<K, Entry<V>>,
    expires_after_creation: Option<Duration>,
    expires_after_access: Option<Duration>,
    max_hits: Option<u32>,
}

/// # InMemoryCache.Entry
///
/// The `Entry` wraps stored value and holds info about creation and access time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry<V> {
    value: V,
    created_at: DateTime<Local>,
    accessed_at: DateTime<Local>,
    hits: u32,
}

impl<V> Entry<V> {
    pub fn new(value: V) -> Self {
        let now = Local::now();
        Self {
            value,
            created_at: now.to_owned(),
            accessed_at: now,
            hits: 0,
        }
    }
}

impl<K: Eq + Hash, V> InMemoryCache<K, V> {
    /// Create in-memory cache instance with specified capacity.
    ///
    /// ### Example:
    /// ```ignore
    /// let mut cache = InMemoryCache::with_capacity(3000);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: LruCache::new(
                NonZeroUsize::new(capacity).expect("Shall be correct by method contract"),
            ),
            expires_after_creation: None,
            expires_after_access: None,
            max_hits: None,
        }
    }

    /// Set expiration policy by creation time.
    ///
    /// Value stored in the cache will be considered as expired
    /// if sum of its creation time and this policy `duration` is less than current time.
    pub fn expires_after_creation(mut self, duration: Duration) -> Self {
        self.expires_after_creation = Some(duration);
        self
    }

    /// Set expiration policy by access time.
    ///
    /// Value stored in the cache will be considered as expired
    /// if sum of its last access time and this policy `duration` is less than current time.
    pub fn expires_after_access(mut self, duration: Duration) -> Self {
        self.expires_after_access = Some(duration);
        self
    }

    /// Set expiration policy by access time.
    ///
    /// Value stored in the cache will be considered as expired
    /// if sum of its last access time and this policy `duration` is less than current time.
    pub fn max_hits(mut self, max_hits: u32) -> Self {
        self.max_hits = Some(max_hits);
        self
    }

    /// Insert value into the cache
    ///
    /// If an entry with key `k` already exists in the cache or another cache entry is removed
    /// (due to the lru's capacity), then it returns the old entry's key-value pair.
    /// Otherwise, returns `None`.
    pub fn insert(&mut self, key: K, value: V) -> Option<(K, Entry<V>)> {
        self.insert_entry(key, Entry::new(value))
    }

    /// Insert complete LRU cache entry into the cache
    ///
    /// Used for interaction with persistent cache. Because we can keep
    /// oldest items outside of the RAM. For example, in DB or in files.
    pub fn insert_entry(&mut self, key: K, entry: Entry<V>) -> Option<(K, Entry<V>)> {
        self.entries.push(key, entry)
    }

    /// Get value from the cache.
    ///
    /// Returns value if it is not expired, otherwise returns None.
    ///
    /// Exactly at the moment of a method call there are checks on expiration.
    /// Removes expired values from cache.
    pub fn get(&mut self, key: &K) -> Option<&'_ V> {
        self.get_entry(key, false).map(|entry| &entry.0.value)
    }

    /// Get value from the cache.
    ///
    /// Returns tuple:
    /// - (&value, false) - if value is not expired
    /// - (&value, true) - if value expired
    ///
    /// Exactly at the moment of a method call there are checks on expiration.
    /// Does not remove expired values from cache.
    pub fn peek(&mut self, key: &K) -> Option<(&'_ V, bool)> {
        self.get_entry(key, true)
            .map(|(entry, expired)| (&entry.value, expired))
    }

    /// For internal use only
    fn get_entry(&mut self, key: &K, keep_expired_value: bool) -> Option<(&'_ Entry<V>, bool)> {
        let entry = self.entries.get(key);
        // Check 'created_at' expiration policy
        let expired = match (self.expires_after_creation, &entry) {
            (Some(ref duration), Some(entry)) => is_expired(&Some(entry.created_at), duration),
            (_, _) => false,
        };
        // Check 'accessed_at' expiration policy
        let expired = expired
            || match (self.expires_after_access, &entry) {
                (Some(ref duration), Some(entry)) => is_expired(&Some(entry.accessed_at), duration),
                (_, _) => false,
            };
        // Check 'max_hits' expiration policy
        let expired = expired
            || match (self.max_hits, &entry) {
                (Some(max_hits), Some(entry)) => max_hits <= entry.hits,
                (_, _) => false,
            };

        if !keep_expired_value && expired {
            self.entries.pop(key);
            return None;
        }

        // Modify last access date and hits number
        if let Some(entry) = self.entries.get_mut(key) {
            entry.accessed_at = Local::now();
            entry.hits = entry.hits.saturating_add(1);
        };

        // Return entry
        self.entries.get(key).map(|entry| (entry, expired))
    }

    /// Returns a bool indicating whether the given key is in the cache.
    /// There are no any checks on expiration or cache modification
    /// during this call.
    pub fn contains(&self, key: &K) -> bool {
        self.entries.contains(key)
    }
}

fn is_expired(start: &Option<DateTime<Local>>, duration: &Duration) -> bool {
    start
        .and_then(|s| s.checked_add_signed(*duration))
        .filter(|&e| e <= Local::now())
        .is_some()
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Local};

    use super::{Entry, InMemoryCache};

    #[test]
    fn test_insert_then_get() {
        let mut cache = InMemoryCache::with_capacity(10);
        cache.insert("Hello", 1);
        cache.insert("World", 2);
        assert_eq!(cache.get(&"Hello"), Some(&1));
        assert_eq!(cache.get(&"World"), Some(&2));
    }

    #[test]
    fn test_insert_then_get_create_expired() {
        let mut cache =
            InMemoryCache::with_capacity(10).expires_after_creation(Duration::minutes(5));
        let expired_entry = Entry {
            value: 2,
            created_at: Local::now()
                .checked_sub_signed(Duration::minutes(5))
                .unwrap(),
            accessed_at: Local::now(),
            hits: 0,
        };
        let not_expired_entry = Entry {
            value: 3,
            created_at: Local::now()
                .checked_sub_signed(Duration::minutes(4))
                .unwrap(),
            accessed_at: Local::now(),
            hits: 0,
        };

        cache.insert_entry("Expired", expired_entry);
        cache.insert_entry("NotExpired", not_expired_entry);
        assert!(cache.get(&"Expired").is_none());
        assert!(cache.get(&"NotExpired").is_some());
    }

    #[test]
    fn test_insert_then_get_access_expired() {
        let mut cache = InMemoryCache::with_capacity(10).expires_after_access(Duration::minutes(5));
        let expired_entry = Entry {
            value: "Expired",
            accessed_at: Local::now()
                .checked_sub_signed(Duration::minutes(5))
                .unwrap(),
            created_at: Local::now(),
            hits: 0,
        };
        let not_expired_entry = Entry {
            value: "NotExpired",
            accessed_at: Local::now()
                .checked_sub_signed(Duration::minutes(4))
                .unwrap(),
            created_at: Local::now(),
            hits: 0,
        };

        cache.insert_entry(2, expired_entry);
        cache.insert_entry(3, not_expired_entry);
        assert!(cache.get(&2).is_none());
        assert!(cache.get(&3).is_some());
    }

    #[test]
    fn test_insert_then_get_max_hits_expired() {
        let mut cache = InMemoryCache::with_capacity(10).max_hits(10);
        let expired_entry = Entry {
            value: "Expired",
            accessed_at: Local::now(),
            created_at: Local::now(),
            hits: 10,
        };
        let not_expired_entry = Entry {
            value: "NotExpired",
            accessed_at: Local::now(),
            created_at: Local::now(),
            hits: 0,
        };

        cache.insert_entry(2, expired_entry);
        cache.insert_entry(3, not_expired_entry);
        assert!(cache.get(&2).is_none());
        assert!(cache.get(&3).is_some());
    }

    #[test]
    fn test_maximum_capacity() {
        let mut cache = InMemoryCache::with_capacity(3);
        cache.insert(1, "Lorem");
        cache.insert(2, "Ipsum");
        cache.insert(3, "Dolor");
        cache.insert(4, "Sit");
        cache.insert(5, "Amet");
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some(&"Dolor"));
        assert_eq!(cache.get(&4), Some(&"Sit"));
        assert_eq!(cache.get(&5), Some(&"Amet"));
    }
}
