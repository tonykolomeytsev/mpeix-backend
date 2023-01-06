use std::hash::Hash;
use std::num::NonZeroUsize;

use chrono::{DateTime, Duration, Local};
use lru::LruCache;

/// # InMemoryCache
///
/// In-Memory Cache implementation based on LRU (last recent used) cache.
///
/// Supports expiration policies:
/// - By creation time:
///   ```rust
///   let mut cache = InMemoryCache::with_capacity(1000)
///       .expires_after_creation(Duration::hours(1));
///   ```
/// - By last access time:
///   ```rust
///   let mut cache = InMemoryCache::with_capacity(3000)
///       .expires_after_access(Duration::minutes(5));
///   ```
///
/// ### Example:
/// ```rust
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
}

/// # InMemoryCache.Entry
///
/// For internal use only.
/// The `Entry` wraps stored value and holds info about creation and access time.
struct Entry<V> {
    value: V,
    created_at: Option<DateTime<Local>>,
    accessed_at: DateTime<Local>,
}

impl<K: Eq + Hash, V> InMemoryCache<K, V> {
    /// Create in-memory cache instance with specified capacity.
    ///
    /// ### Example:
    /// ```rust
    /// let mut cache = InMemoryCache::with_capacity(3000);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: LruCache::new(
                NonZeroUsize::new(capacity).expect("Shall be correct by method contract"),
            ),
            expires_after_creation: None,
            expires_after_access: None,
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

    /// Insert value into the cache
    pub fn insert(&mut self, key: K, value: V) {
        self.insert_entry(
            key,
            Entry {
                value,
                created_at: self.expires_after_creation.map(|_| Local::now()),
                accessed_at: Local::now(),
            },
        );
    }

    fn insert_entry(&mut self, key: K, entry: Entry<V>) {
        self.entries.put(key, entry);
    }

    /// Get value from the cache.
    ///
    /// Exactly at the moment of a method call there are checks on expiration.
    pub fn get(&mut self, key: &K) -> Option<&'_ V> {
        let entry = self.entries.get(key);
        // Check 'created_at' expiration policy
        if match (self.expires_after_creation, &entry) {
            (Some(ref duration), Some(entry)) => is_expired(&entry.created_at, duration),
            (_, _) => false,
        } {
            self.entries.pop(key);
            return None;
        }
        // Check 'accessed_at' expiration policy
        if match (self.expires_after_access, &entry) {
            (Some(ref duration), Some(entry)) => is_expired(&Some(entry.accessed_at), duration),
            (_, _) => false,
        } {
            self.entries.pop(key);
            return None;
        }
        // Modify last access date
        if let Some(entry) = self.entries.get_mut(key) {
            entry.accessed_at = Local::now();
        };
        // Return entry
        self.entries.get(key).map(|entry| &entry.value)
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
            created_at: Some(
                Local::now()
                    .checked_sub_signed(Duration::minutes(5))
                    .unwrap(),
            ),
            accessed_at: Local::now(),
        };
        let not_expired_entry = Entry {
            value: 3,
            created_at: Some(
                Local::now()
                    .checked_sub_signed(Duration::minutes(4))
                    .unwrap(),
            ),
            accessed_at: Local::now(),
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
            created_at: None,
        };
        let not_expired_entry = Entry {
            value: "NotExpired",
            accessed_at: Local::now()
                .checked_sub_signed(Duration::minutes(4))
                .unwrap(),
            created_at: None,
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
