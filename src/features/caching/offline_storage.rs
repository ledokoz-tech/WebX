// Offline Storage for Web Pages
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Offline page manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineManifest {
    pub url: String,
    pub title: String,
    pub saved_at: chrono::DateTime<chrono::Utc>,
    pub resources: Vec<ResourceInfo>,
    pub main_content_hash: String,
    pub version: u32,
}

/// Resource information for offline storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub url: String,
    pub content_type: String,
    pub file_path: String,
    pub size: usize,
    pub hash: String,
}

/// Offline storage manager
pub struct OfflineStorage {
    storage_dir: PathBuf,
    manifests: HashMap<String, OfflineManifest>,
    max_storage_size: usize,
    current_size: usize,
}

impl OfflineStorage {
    /// Create new offline storage
    pub fn new(storage_dir: Option<PathBuf>, max_size_mb: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let storage_dir = storage_dir.unwrap_or_else(|| {
            let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("webx");
            path.push("offline");
            path
        });

        // Create storage directory
        fs::create_dir_all(&storage_dir)?;

        let mut storage = Self {
            storage_dir,
            manifests: HashMap::new(),
            max_storage_size: max_size_mb * 1024 * 1024,
            current_size: 0,
        };

        // Load existing manifests
        storage.load_manifests()?;

        Ok(storage)
    }

    /// Save a web page for offline viewing
    pub fn save_page(
        &mut self,
        url: &str,
        title: &str,
        html_content: &str,
        resources: Vec<(String, String, Vec<u8>)>, // (url, content_type, data)
    ) -> Result<String, Box<dyn std::error::Error>> {
        let page_id = format!("page_{}", uuid::Uuid::new_v4());
        let page_dir = self.storage_dir.join(&page_id);
        fs::create_dir_all(&page_dir)?;

        // Save main HTML content
        let html_path = page_dir.join("index.html");
        fs::write(&html_path, html_content)?;

        // Save resources
        let mut resource_infos = Vec::new();
        for (resource_url, content_type, data) in resources {
            let filename = self.generate_filename(&resource_url, &content_type);
            let file_path = page_dir.join(&filename);
            
            fs::write(&file_path, &data)?;
            
            let hash = self.calculate_hash(&data);
            resource_infos.push(ResourceInfo {
                url: resource_url,
                content_type,
                file_path: filename,
                size: data.len(),
                hash,
            });
        }

        // Create manifest
        let manifest = OfflineManifest {
            url: url.to_string(),
            title: title.to_string(),
            saved_at: chrono::Utc::now(),
            resources: resource_infos,
            main_content_hash: self.calculate_hash(html_content.as_bytes()),
            version: 1,
        };

        // Save manifest
        let manifest_path = page_dir.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(manifest_path, manifest_json)?;

        self.manifests.insert(url.to_string(), manifest);
        self.save_manifest_index()?;
        self.update_storage_size()?;

        // Check storage limits
        self.enforce_storage_limits()?;

        Ok(page_id)
    }

    /// Load offline page
    pub fn load_page(&self, url: &str) -> Result<Option<OfflinePage>, Box<dyn std::error::Error>> {
        if let Some(manifest) = self.manifests.get(url) {
            let page_dir = self.storage_dir.join(self.get_page_id_from_url(url)?);
            
            // Load HTML content
            let html_path = page_dir.join("index.html");
            let html_content = fs::read_to_string(&html_path)?;
            
            // Load resources
            let mut resources = HashMap::new();
            for resource in &manifest.resources {
                let file_path = page_dir.join(&resource.file_path);
                let data = fs::read(&file_path)?;
                resources.insert(resource.url.clone(), (resource.content_type.clone(), data));
            }
            
            Ok(Some(OfflinePage {
                url: manifest.url.clone(),
                title: manifest.title.clone(),
                html_content,
                resources,
                saved_at: manifest.saved_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Check if page is available offline
    pub fn is_page_offline(&self, url: &str) -> bool {
        self.manifests.contains_key(url)
    }

    /// List all offline pages
    pub fn list_pages(&self) -> Vec<OfflinePageInfo> {
        self.manifests
            .values()
            .map(|manifest| OfflinePageInfo {
                url: manifest.url.clone(),
                title: manifest.title.clone(),
                saved_at: manifest.saved_at,
                resource_count: manifest.resources.len(),
                total_size: manifest.resources.iter().map(|r| r.size).sum(),
            })
            .collect()
    }

    /// Delete offline page
    pub fn delete_page(&mut self, url: &str) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(manifest) = self.manifests.remove(url) {
            let page_id = self.get_page_id_from_url(url)?;
            let page_dir = self.storage_dir.join(&page_id);
            
            // Remove directory
            if page_dir.exists() {
                fs::remove_dir_all(&page_dir)?;
            }
            
            self.save_manifest_index()?;
            self.update_storage_size()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get storage statistics
    pub fn stats(&self) -> OfflineStorageStats {
        OfflineStorageStats {
            page_count: self.manifests.len(),
            total_size_mb: self.current_size as f64 / (1024.0 * 1024.0),
            max_size_mb: self.max_storage_size as f64 / (1024.0 * 1024.0),
            oldest_page: self.manifests.values().map(|m| m.saved_at).min(),
            newest_page: self.manifests.values().map(|m| m.saved_at).max(),
        }
    }

    /// Clear all offline storage
    pub fn clear_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        fs::remove_dir_all(&self.storage_dir)?;
        fs::create_dir_all(&self.storage_dir)?;
        self.manifests.clear();
        self.current_size = 0;
        Ok(())
    }

    // Private helper methods
    
    fn load_manifests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let index_path = self.storage_dir.join("manifests.json");
        if index_path.exists() {
            let content = fs::read_to_string(&index_path)?;
            self.manifests = serde_json::from_str(&content)?;
        }
        self.update_storage_size()?;
        Ok(())
    }

    fn save_manifest_index(&self) -> Result<(), Box<dyn std::error::Error>> {
        let index_path = self.storage_dir.join("manifests.json");
        let content = serde_json::to_string_pretty(&self.manifests)?;
        fs::write(index_path, content)?;
        Ok(())
    }

    fn update_storage_size(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.current_size = 0;
        for manifest in self.manifests.values() {
            self.current_size += manifest.resources.iter().map(|r| r.size).sum::<usize>();
            // Add approximate size for HTML content and manifest
            self.current_size += 1024 * 10; // ~10KB for HTML + manifest
        }
        Ok(())
    }

    fn enforce_storage_limits(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while self.current_size > self.max_storage_size && !self.manifests.is_empty() {
            // Remove oldest page
            if let Some(oldest_url) = self.manifests
                .iter()
                .min_by_key(|(_, manifest)| manifest.saved_at)
                .map(|(url, _)| url.clone())
            {
                self.delete_page(&oldest_url)?;
            } else {
                break;
            }
        }
        Ok(())
    }

    fn generate_filename(&self, url: &str, content_type: &str) -> String {
        let extension = self.get_extension_from_content_type(content_type);
        let hash = format!("{:x}", md5::compute(url));
        format!("{}.{}", &hash[..16], extension)
    }

    fn get_extension_from_content_type(&self, content_type: &str) -> &str {
        match content_type {
            ct if ct.starts_with("image/") => {
                if ct.contains("jpeg") || ct.contains("jpg") {
                    "jpg"
                } else if ct.contains("png") {
                    "png"
                } else if ct.contains("gif") {
                    "gif"
                } else if ct.contains("webp") {
                    "webp"
                } else {
                    "bin"
                }
            }
            ct if ct.starts_with("text/css") => "css",
            ct if ct.starts_with("application/javascript") => "js",
            ct if ct.starts_with("font/") => "woff2",
            _ => "bin",
        }
    }

    fn calculate_hash(&self, data: &[u8]) -> String {
        format!("{:x}", md5::compute(data))
    }

    fn get_page_id_from_url(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.manifests
            .get(url)
            .map(|manifest| {
                // Extract page ID from manifest path or regenerate
                format!("page_{:x}", md5::compute(&manifest.url))
            })
            .ok_or_else(|| "Page not found".into())
    }
}

/// Offline page data
#[derive(Debug, Clone)]
pub struct OfflinePage {
    pub url: String,
    pub title: String,
    pub html_content: String,
    pub resources: HashMap<String, (String, Vec<u8>)>, // url -> (content_type, data)
    pub saved_at: chrono::DateTime<chrono::Utc>,
}

/// Information about offline page
#[derive(Debug, Clone)]
pub struct OfflinePageInfo {
    pub url: String,
    pub title: String,
    pub saved_at: chrono::DateTime<chrono::Utc>,
    pub resource_count: usize,
    pub total_size: usize,
}

/// Offline storage statistics
#[derive(Debug, Clone)]
pub struct OfflineStorageStats {
    pub page_count: usize,
    pub total_size_mb: f64,
    pub max_size_mb: f64,
    pub oldest_page: Option<chrono::DateTime<chrono::Utc>>,
    pub newest_page: Option<chrono::DateTime<chrono::Utc>>,
}