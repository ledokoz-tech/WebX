// LRU (Least Recently Used) Cache Implementation
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

/// LRU Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub value: T,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
    pub size: usize,
}

/// LRU Cache implementation
pub struct LRUCache<K, V> {
    cache: HashMap<K, CacheEntry<V>>,
    access_order: Vec<K>, // Most recent at the end
    max_size: usize,
    current_size: usize,
}

impl<K, V> LRUCache<K, V>
where
    K: Clone + Eq + Hash + std::fmt::Debug,
    V: Clone + std::fmt::Debug,
{
    /// Create new LRU cache with maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            access_order: Vec::new(),
            max_size,
            current_size: 0,
        }
    }

    /// Insert a value into the cache
    pub fn put(&mut self, key: K, value: V, size: usize) -> Option<V> {
        let timestamp = chrono::Utc::now();
        
        // If key already exists, remove old entry
        if let Some(old_entry) = self.cache.remove(&key) {
            self.current_size -= old_entry.size;
            self.access_order.retain(|k| k != &key);
            return Some(old_entry.value);
        }

        // Evict entries if needed
        while self.current_size + size > self.max_size && !self.access_order.is_empty() {
            if let Some(lru_key) = self.access_order.first() {
                if let Some(entry) = self.cache.remove(lru_key) {
                    self.current_size -= entry.size;
                    self.access_order.remove(0);
                }
            }
        }

        // Insert new entry
        let entry = CacheEntry {
            value: value.clone(),
            timestamp,
            access_count: 1,
            size,
        };

        self.cache.insert(key.clone(), entry);
        self.access_order.push(key);
        self.current_size += size;

        None
    }

    /// Get a value from the cache
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(entry) = self.cache.get_mut(key) {
            entry.access_count += 1;
            entry.timestamp = chrono::Utc::now();

            // Move key to end of access order (most recent)
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
                self.access_order.push(key.clone());
            }

            Some(&entry.value)
        } else {
            None
        }
    }

    /// Check if cache contains key
    pub fn contains_key(&self, key: &K) -> bool {
        self.cache.contains_key(key)
    }

    /// Remove a key from cache
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.cache.remove(key) {
            self.current_size -= entry.size;
            self.access_order.retain(|k| k != key);
            Some(entry.value)
        } else {
            None
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.cache.len(),
            current_memory_usage: self.current_size,
            max_memory_usage: self.max_size,
            hit_count: self.cache.values().map(|e| e.access_count).sum(),
        }
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
        self.current_size = 0;
    }

    /// Get keys in access order (LRU first)
    pub fn keys_lru_first(&self) -> Vec<&K> {
        self.access_order.iter().collect()
    }

    /// Get keys in access order (MRU last)
    pub fn keys_mru_last(&self) -> Vec<&K> {
        self.access_order.iter().rev().collect()
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub size: usize,
    pub current_memory_usage: usize,
    pub max_memory_usage: usize,
    pub hit_count: u64,
}

impl CacheStats {
    /// Get hit ratio (0.0 to 1.0)
    pub fn hit_ratio(&self) -> f64 {
        if self.size == 0 {
            0.0
        } else {
            self.hit_count as f64 / self.size as f64
        }
    }

    /// Get memory usage percentage
    pub fn memory_usage_percent(&self) -> f64 {
        if self.max_memory_usage == 0 {
            0.0
        } else {
            (self.current_memory_usage as f64 / self.max_memory_usage as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_basic_operations() {
        let mut cache = LRUCache::new(100);
        
        // Test put and get
        cache.put("key1", "value1", 10);
        assert_eq!(cache.get(&"key1"), Some(&"value1"));
        
        cache.put("key2", "value2", 20);
        assert_eq!(cache.get(&"key2"), Some(&"value2"));
        
        // Test capacity and eviction
        cache.put("key3", "value3", 80); // This should evict key1
        assert_eq!(cache.get(&"key1"), None);
        assert_eq!(cache.get(&"key3"), Some(&"value3"));
        
        // Test stats
        let stats = cache.stats();
        assert_eq!(stats.size, 2);
        assert_eq!(stats.current_memory_usage, 100);
    }

    #[test]
    fn test_lru_eviction() {
        let mut cache = LRUCache::new(50);
        
        cache.put("a", "value_a", 10);
        cache.put("b", "value_b", 10);
        cache.put("c", "value_c", 10);
        
        // Access 'a' to make it MRU
        cache.get(&"a");
        
        // Add new item that should evict LRU (b)
        cache.put("d", "value_d", 30);
        
        assert_eq!(cache.get(&"a"), Some(&"value_a")); // Still exists
        assert_eq!(cache.get(&"b"), None);             // Evicted
        assert_eq!(cache.get(&"c"), Some(&"value_c")); // Still exists
        assert_eq!(cache.get(&"d"), Some(&"value_d")); // New item
    }
}