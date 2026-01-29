// Password Manager with Encryption
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use pbkdf2::pbkdf2;
use rand::{rngs::OsRng, RngCore};
use ring::digest;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Encrypted password entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordEntry {
    pub id: usize,
    pub website: String,
    pub username: String,
    pub encrypted_password: Vec<u8>,
    pub iv: [u8; 12], // Nonce for AES-GCM
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Password manager for securely storing credentials
pub struct PasswordManager {
    db: Arc<Mutex<Db>>,
    master_key: Option<[u8; 32]>,
    entries: Arc<Mutex<HashMap<usize, PasswordEntry>>>,
}

impl PasswordManager {
    /// Create a new password manager
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
        
        let db = sled::open(db_path)?;
        
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            master_key: None,
            entries: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Unlock the password manager with master password
    pub fn unlock(&mut self, master_password: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let salt = self.get_or_create_salt()?;
        let mut key = [0u8; 32];
        
        // Derive key from password using PBKDF2
        pbkdf2(
            digest::SHA256,
            master_password.as_bytes(),
            &salt,
            100_000,
            &mut key,
        )
        .map_err(|_| "Key derivation failed")?;
        
        self.master_key = Some(key);
        
        // Load existing entries
        self.load_entries()?;
        
        Ok(true)
    }

    /// Lock the password manager
    pub fn lock(&mut self) {
        self.master_key = None;
        self.entries.lock().unwrap().clear();
    }

    /// Check if password manager is unlocked
    pub fn is_unlocked(&self) -> bool {
        self.master_key.is_some()
    }

    /// Add a new password entry
    pub fn add_password(
        &self,
        website: String,
        username: String,
        password: String,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let key = self.master_key.ok_or("Password manager is locked")?;
        
        // Generate random IV/nonce
        let mut iv = [0u8; 12];
        OsRng.fill_bytes(&mut iv);
        
        // Encrypt password
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| "Invalid key")?;
        let nonce = Nonce::from_slice(&iv);
        let encrypted_password = cipher
            .encrypt(nonce, password.as_bytes())
            .map_err(|_| "Encryption failed")?;
        
        let id = {
            let entries = self.entries.lock().unwrap();
            entries.len() + 1
        };
        
        let entry = PasswordEntry {
            id,
            website,
            username,
            encrypted_password,
            iv,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        // Save to database
        self.save_entry(&entry)?;
        
        // Add to memory
        self.entries.lock().unwrap().insert(id, entry);
        
        Ok(id)
    }

    /// Get password entry by ID
    pub fn get_password(&self, id: usize) -> Result<Option<(String, String, String)>, Box<dyn std::error::Error>> {
        let key = self.master_key.ok_or("Password manager is locked")?;
        let entries = self.entries.lock().unwrap();
        
        if let Some(entry) = entries.get(&id) {
            // Decrypt password
            let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| "Invalid key")?;
            let nonce = Nonce::from_slice(&entry.iv);
            let decrypted_password = cipher
                .decrypt(nonce, entry.encrypted_password.as_slice())
                .map_err(|_| "Decryption failed")?;
            
            let password = String::from_utf8(decrypted_password)
                .map_err(|_| "Invalid UTF-8 in decrypted password")?;
            
            Ok(Some((
                entry.website.clone(),
                entry.username.clone(),
                password,
            )))
        } else {
            Ok(None)
        }
    }

    /// Update password entry
    pub fn update_password(
        &self,
        id: usize,
        website: Option<String>,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let key = self.master_key.ok_or("Password manager is locked")?;
        let mut entries = self.entries.lock().unwrap();
        
        if let Some(entry) = entries.get_mut(&id) {
            if let Some(new_website) = website {
                entry.website = new_website;
            }
            
            if let Some(new_username) = username {
                entry.username = new_username;
            }
            
            if let Some(new_password) = password {
                // Generate new IV
                let mut iv = [0u8; 12];
                OsRng.fill_bytes(&mut iv);
                
                // Encrypt new password
                let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| "Invalid key")?;
                let nonce = Nonce::from_slice(&iv);
                let encrypted_password = cipher
                    .encrypt(nonce, new_password.as_bytes())
                    .map_err(|_| "Encryption failed")?;
                
                entry.encrypted_password = encrypted_password;
                entry.iv = iv;
            }
            
            entry.updated_at = chrono::Utc::now();
            
            // Save to database
            self.save_entry(entry)?;
            
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Delete password entry
    pub fn delete_password(&self, id: usize) -> Result<bool, Box<dyn std::error::Error>> {
        let db = self.db.lock().unwrap();
        let key = format!("password_{}", id);
        
        if db.contains_key(&key)? {
            db.remove(&key)?;
            self.entries.lock().unwrap().remove(&id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get all password entries (without decrypted passwords)
    pub fn list_passwords(&self) -> Vec<(usize, String, String)> {
        let entries = self.entries.lock().unwrap();
        entries
            .values()
            .map(|entry| (entry.id, entry.website.clone(), entry.username.clone()))
            .collect()
    }

    /// Find passwords by website
    pub fn find_passwords(&self, website_query: &str) -> Vec<(usize, String, String)> {
        let entries = self.entries.lock().unwrap();
        entries
            .values()
            .filter(|entry| entry.website.to_lowercase().contains(&website_query.to_lowercase()))
            .map(|entry| (entry.id, entry.website.clone(), entry.username.clone()))
            .collect()
    }

    /// Generate a strong password
    pub fn generate_password(&self, length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789\
                                !@#$%^&*()_+-=[]{}|;:,.<>?";
        
        let mut rng = OsRng;
        let mut password = String::with_capacity(length);
        
        for _ in 0..length {
            let idx = (rng.next_u32() as usize) % CHARSET.len();
            password.push(CHARSET[idx] as char);
        }
        
        password
    }

    // Private helper methods
    
    fn get_or_create_salt(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let db = self.db.lock().unwrap();
        let salt_key = "master_salt";
        
        if let Some(salt_bytes) = db.get(salt_key)? {
            Ok(salt_bytes.to_vec())
        } else {
            // Generate new salt
            let mut salt = [0u8; 32];
            OsRng.fill_bytes(&mut salt);
            
            db.insert(salt_key, &salt)?;
            Ok(salt.to_vec())
        }
    }
    
    fn save_entry(&self, entry: &PasswordEntry) -> Result<(), Box<dyn std::error::Error>> {
        let db = self.db.lock().unwrap();
        let key = format!("password_{}", entry.id);
        let serialized = serde_json::to_vec(entry)?;
        db.insert(key, serialized)?;
        Ok(())
    }
    
    fn load_entries(&self) -> Result<(), Box<dyn std::error::Error>> {
        let db = self.db.lock().unwrap();
        let mut entries = self.entries.lock().unwrap();
        entries.clear();
        
        for result in db.iter() {
            let (key, value) = result?;
            let key_str = String::from_utf8(key.to_vec())?;
            
            if key_str.starts_with("password_") {
                let entry: PasswordEntry = serde_json::from_slice(&value)?;
                entries.insert(entry.id, entry);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_password_manager_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = PasswordManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Unlock with master password
        assert!(manager.unlock("test_master_password").unwrap());
        assert!(manager.is_unlocked());
        
        // Add a password
        let id = manager
            .add_password(
                "example.com".to_string(),
                "user@example.com".to_string(),
                "mypassword123".to_string(),
            )
            .unwrap();
        
        assert!(id > 0);
        
        // Retrieve the password
        let result = manager.get_password(id).unwrap().unwrap();
        assert_eq!(result.0, "example.com");
        assert_eq!(result.1, "user@example.com");
        assert_eq!(result.2, "mypassword123");
        
        // List passwords
        let list = manager.list_passwords();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].1, "example.com");
        
        // Lock the manager
        manager.lock();
        assert!(!manager.is_unlocked());
    }

    #[test]
    fn test_password_generation() {
        let manager = PasswordManager::new(None).unwrap();
        let password = manager.generate_password(16);
        assert_eq!(password.len(), 16);
        
        let password2 = manager.generate_password(20);
        assert_eq!(password2.len(), 20);
    }
}