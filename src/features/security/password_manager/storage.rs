// Password Storage Backend
use sled::Db;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Password storage manager
pub struct PasswordStorage {
    db: Arc<Mutex<Db>>,
    db_path: PathBuf,
}

impl PasswordStorage {
    /// Create new password storage
    pub fn new(db_path: Option<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = db_path.unwrap_or_else(|| {
            let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("webx");
            path.push("passwords.db");
            path
        });
        
        // Create parent directories
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let db = sled::open(&db_path)?;
        
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            db_path,
        })
    }

    /// Save password entry
    pub fn save_password(
        &self,
        url: &str,
        username: &str,
        encrypted_password: &(Vec<u8>, [u8; 12]),
    ) -> Result<(), Box<dyn std::error::Error>> {
        let id = self.generate_id(url, username);
        let (ref encrypted_data, ref iv) = encrypted_password;
        
        self.store_password(
            id,
            url,
            username,
            encrypted_data,
            iv,
        )
    }
    
    /// Get password entry
    pub fn get_password(
        &self,
        url: &str,
        username: &str,
    ) -> Result<Option<(Vec<u8>, [u8; 12])>, Box<dyn std::error::Error>> {
        let id = self.generate_id(url, username);
        
        if let Some(encrypted_data) = self.get_password_by_id(id)? {
            // Parse the stored data to extract encrypted password and IV
            let entry: serde_json::Value = serde_json::from_slice(&encrypted_data)?;
            
            if let (Some(encrypted_password), Some(iv_array)) = (
                entry.get("encrypted_password").and_then(|v| v.as_array()),
                entry.get("iv").and_then(|v| v.as_array()),
            ) {
                let encrypted: Vec<u8> = encrypted_password.iter()
                    .filter_map(|v| v.as_u64().map(|n| n as u8))
                    .collect();
                
                let iv: [u8; 12] = iv_array.iter()
                    .take(12)
                    .filter_map(|v| v.as_u64().map(|n| n as u8))
                    .collect::<Vec<u8>>()
                    .try_into()
                    .unwrap_or([0u8; 12]);
                
                Ok(Some((encrypted, iv)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    
    /// Generate ID from URL and username
    fn generate_id(&self, url: &str, username: &str) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        username.hash(&mut hasher);
        hasher.finish() as usize
    }
    
    /// Get password by ID (internal helper)
    fn get_password_by_id(&self, id: usize) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let db = self.db.lock().unwrap();
        let key = format!("password_{}", id);
        
        if let Some(value) = db.get(key)? {
            Ok(Some(value.to_vec()))
        } else {
            Ok(None)
        }
    }

    /// Store encrypted password entry (internal use)
    fn store_password(
        &self,
        id: usize,
        website: &str,
        username: &str,
        encrypted_password: &[u8],
        iv: &[u8; 12],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let db = self.db.lock().unwrap();
        let key = format!("password_{}", id);
        
        let entry = serde_json::json!({
            "id": id,
            "website": website,
            "username": username,
            "encrypted_password": encrypted_password,
            "iv": iv,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "updated_at": chrono::Utc::now().to_rfc3339()
        });
        
        let serialized = serde_json::to_vec(&entry)?;
        db.insert(key, serialized)?;
        Ok(())
    }
    pub fn get_password(&self, id: usize) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let db = self.db.lock().unwrap();
        let key = format!("password_{}", id);
        
        if let Some(value) = db.get(key)? {
            Ok(Some(value.to_vec()))
        } else {
            Ok(None)
        }
    }

    /// List all password entries (metadata only)
    pub fn list_passwords(&self) -> Result<Vec<(usize, String, String)>, Box<dyn std::error::Error>> {
        let db = self.db.lock().unwrap();
        let mut entries = Vec::new();
        
        for result in db.iter() {
            let (key, value) = result?;
            let key_str = String::from_utf8(key.to_vec())?;
            
            if key_str.starts_with("password_") {
                if let Ok(entry) = serde_json::from_slice::<serde_json::Value>(&value) {
                    if let (Some(id), Some(website), Some(username)) = (
                        entry.get("id").and_then(|v| v.as_u64()),
                        entry.get("website").and_then(|v| v.as_str()),
                        entry.get("username").and_then(|v| v.as_str()),
                    ) {
                        entries.push((id as usize, website.to_string(), username.to_string()));
                    }
                }
            }
        }
        
        Ok(entries)
    }

    /// Delete password entry
    pub fn delete_password(&self, id: usize) -> Result<bool, Box<dyn std::error::Error>> {
        let db = self.db.lock().unwrap();
        let key = format!("password_{}", id);
        
        if db.contains_key(&key)? {
            db.remove(&key)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Store master password salt
    pub fn store_salt(&self, salt: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let db = self.db.lock().unwrap();
        db.insert("master_salt", salt)?;
        Ok(())
    }

    /// Retrieve master password salt
    pub fn get_salt(&self) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let db = self.db.lock().unwrap();
        if let Some(salt) = db.get("master_salt")? {
            Ok(Some(salt.to_vec()))
        } else {
            Ok(None)
        }
    }

    /// Get database path
    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }
}