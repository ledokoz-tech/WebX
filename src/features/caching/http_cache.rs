// HTTP Response Caching
use crate::features::caching::lru_cache::{LRUCache, CacheEntry};
use flate2::{Compression, write::GzEncoder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::time::Duration;

/// HTTP cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HTTPCacheEntry {
    pub url: String,
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub compressed_body: Option<Vec<u8>>,
    pub content_type: Option<String>,
    pub content_length: usize,
    pub cache_control: Option<String>,
    pub expires: Option<chrono::DateTime<chrono::Utc>>,
    pub etag: Option<String>,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
}

/// HTTP cache manager
pub struct HTTPCache {
    cache: LRUCache<String, HTTPCacheEntry>,
    default_ttl: Duration,
    compression_enabled: bool,
}

impl HTTPCache {
    /// Create new HTTP cache
    pub fn new(max_size_mb: usize, default_ttl_minutes: u64, compression_enabled: bool) -> Self {
        let max_size_bytes = max_size_mb * 1024 * 1024;
        Self {
            cache: LRUCache::new(max_size_bytes),
            default_ttl: Duration::from_secs(default_ttl_minutes * 60),
            compression_enabled,
        }
    }

    /// Store HTTP response in cache
    pub fn store_response(
        &mut self,
        url: String,
        status_code: u16,
        headers: HashMap<String, String>,
        body: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content_type = headers.get("content-type").cloned();
        let content_length = body.len();
        let cache_control = headers.get("cache-control").cloned();
        let etag = headers.get("etag").cloned();
        
        let last_modified = headers.get("last-modified").and_then(|lm| {
            chrono::DateTime::parse_from_rfc2822(lm).ok().map(|dt| dt.with_timezone(&chrono::Utc))
        });
        
        let expires = self.calculate_expires(&headers, chrono::Utc::now());

        let mut compressed_body = None;
        if self.compression_enabled && content_length > 1024 {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&body)?;
            compressed_body = Some(encoder.finish()?);
        }

        let entry = HTTPCacheEntry {
            url: url.clone(),
            status_code,
            headers,
            body,
            compressed_body,
            content_type,
            content_length,
            cache_control,
            expires,
            etag,
            last_modified,
        };

        let entry_size = self.calculate_entry_size(&entry);
        self.cache.put(url, entry, entry_size);

        Ok(())
    }

    /// Get cached HTTP response
    pub fn get_response(&mut self, url: &str) -> Option<HTTPCacheEntry> {
        let key = url.to_string();
        let entry = self.cache.get(&key)?.clone();
        
        // Check if expired
        if let Some(expires) = entry.expires {
            if chrono::Utc::now() > expires {
                self.cache.remove(&key);
                return None;
            }
        }
        
        Some(entry)
    }

    /// Check if response is cacheable
    pub fn is_cacheable(&self, status_code: u16, headers: &HashMap<String, String>) -> bool {
        // Check status code
        if ![200, 203, 204, 206, 300, 301, 404, 405, 410, 414, 501].contains(&status_code) {
            return false;
        }

        // Check cache-control header
        if let Some(cache_control) = headers.get("cache-control") {
            if cache_control.contains("no-store") || cache_control.contains("no-cache") {
                return false;
            }
        }

        // Check pragma header
        if let Some(pragma) = headers.get("pragma") {
            if pragma.contains("no-cache") {
                return false;
            }
        }

        true
    }

    /// Get cache statistics
    pub fn stats(&mut self) -> HTTPCacheStats {
        let lru_stats = self.cache.stats();
        HTTPCacheStats {
            entries: lru_stats.size,
            memory_usage_mb: lru_stats.current_memory_usage as f64 / (1024.0 * 1024.0),
            hit_count: lru_stats.hit_count,
            compression_ratio: self.calculate_compression_ratio(),
        }
    }

    /// Clear expired entries
    pub fn clear_expired(&mut self) {
        let now = chrono::Utc::now();
        let keys_to_check: Vec<String> = self
            .cache
            .keys_lru_first()
            .iter()
            .map(|key| (**key).clone())
            .collect();
            
        let mut expired_keys = Vec::new();
            
        for key in keys_to_check {
            if let Some(entry) = self.cache.get(&key) {
                if let Some(expires) = entry.expires {
                    if now > expires {
                        expired_keys.push(key);
                    }
                }
            }
        }
            
        for key in expired_keys {
            self.cache.remove(&key);
        }
    }

    /// Clear all cache entries
    pub fn clear_all(&mut self) {
        self.cache.clear();
    }

    // Private helper methods
    
    fn calculate_expires(
        &self,
        headers: &HashMap<String, String>,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Option<chrono::DateTime<chrono::Utc>> {
        // Check expires header
        if let Some(expires_str) = headers.get("expires") {
            if let Ok(expires) = chrono::DateTime::parse_from_rfc2822(expires_str) {
                return Some(expires.with_timezone(&chrono::Utc));
            }
        }

        // Check cache-control max-age
        if let Some(cache_control) = headers.get("cache-control") {
            if let Some(max_age_str) = self.extract_max_age(cache_control) {
                if let Ok(max_age) = max_age_str.parse::<u64>() {
                    return Some(now + chrono::Duration::seconds(max_age as i64));
                }
            }
        }

        // Use default TTL
        Some(now + chrono::Duration::from_std(self.default_ttl).unwrap())
    }

    fn extract_max_age<'a>(&self, cache_control: &'a str) -> Option<&'a str> {
        for directive in cache_control.split(',') {
            let trimmed = directive.trim();
            if trimmed.starts_with("max-age=") {
                return Some(&trimmed[8..]);
            }
        }
        None
    }

    fn calculate_entry_size(&self, entry: &HTTPCacheEntry) -> usize {
        let mut size = 0;
        size += entry.url.len();
        size += entry.body.len();
        size += entry.headers.values().map(|v| v.len()).sum::<usize>();
        size += entry.content_type.as_ref().map(|ct| ct.len()).unwrap_or(0);
        size += entry.cache_control.as_ref().map(|cc| cc.len()).unwrap_or(0);
        size += entry.etag.as_ref().map(|e| e.len()).unwrap_or(0);
        size
    }

    fn calculate_compression_ratio(&mut self) -> f64 {
        let mut total_original = 0;
        let mut total_compressed = 0;
        
        // Collect all entries first to avoid borrowing conflicts
        let entries: Vec<_> = self.cache
            .keys_mru_last()
            .iter()
            .filter_map(|key| self.cache.get(key).cloned())
            .collect();
        
        for cache_entry in entries {
            total_original += cache_entry.body.len();
            if let Some(compressed) = &cache_entry.compressed_body {
                total_compressed += compressed.len();
            } else {
                total_compressed += cache_entry.body.len();
            }
        }
        
        if total_original == 0 {
            1.0
        } else {
            total_compressed as f64 / total_original as f64
        }
    }
}

/// HTTP cache statistics
#[derive(Debug, Clone)]
pub struct HTTPCacheStats {
    pub entries: usize,
    pub memory_usage_mb: f64,
    pub hit_count: u64,
    pub compression_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_http_cache_basic_operations() {
        let mut cache = HTTPCache::new(10, 60, true);
        
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/html".to_string());
        headers.insert("cache-control".to_string(), "max-age=3600".to_string());
        
        let body = b"<html><body>Hello World</body></html>".to_vec();
        
        // Test storing response
        cache.store_response("https://example.com".to_string(), 200, headers.clone(), body.clone()).unwrap();
        
        // Test retrieving response
        let cached = cache.get_response("https://example.com").unwrap();
        assert_eq!(cached.status_code, 200);
        assert_eq!(cached.body, body);
        assert_eq!(cached.content_type, Some("text/html".to_string()));
        
        // Test cache statistics
        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
        assert!(stats.memory_usage_mb > 0.0);
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = HTTPCache::new(1, 60, false); // 1MB cache
        
        // Add large responses to trigger eviction
        for i in 0..100 {
            let url = format!("https://example.com/{}", i);
            let body = vec![0u8; 1024 * 50]; // 50KB each
            let headers = HashMap::new();
            
            cache.store_response(url, 200, headers, body).unwrap();
        }
        
        // Cache should have evicted older entries
        let stats = cache.stats();
        assert!(stats.entries < 100);
        assert!(stats.memory_usage_mb <= 1.0);
    }
}