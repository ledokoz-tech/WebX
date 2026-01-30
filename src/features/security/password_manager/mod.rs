// Password Manager Module
pub mod encryption;
pub mod storage;
pub mod ui;

pub use encryption::PasswordEncryption;
pub use storage::PasswordStorage;
pub use ui::PasswordUI;

use std::sync::{Arc, Mutex};

/// Main Password Manager that coordinates all password functionality
pub struct PasswordManager {
    storage: Arc<Mutex<PasswordStorage>>,
    encryption: PasswordEncryption,
    ui: PasswordUI,
}

impl PasswordManager {
    /// Create new password manager
    pub fn new(master_password: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let storage = Arc::new(Mutex::new(PasswordStorage::new(None)?));
        let encryption = PasswordEncryption::new(master_password.unwrap_or("default"))?;
        let ui = PasswordUI::new();
        
        Ok(Self {
            storage,
            encryption,
            ui,
        })
    }
    
    /// Save a password
    pub fn save_password(
        &self,
        url: &str,
        username: &str,
        password: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let encrypted_password = self.encryption.encrypt(password)?;
        let mut storage = self.storage.lock().unwrap();
        storage.save_password(url, username, &encrypted_password)
    }
    
    /// Get a password
    pub fn get_password(
        &self,
        url: &str,
        username: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let storage = self.storage.lock().unwrap();
        if let Some(encrypted_password) = storage.get_password(url, username)? {
            let decrypted = self.encryption.decrypt(&encrypted_password)?;
            Ok(Some(decrypted))
        } else {
            Ok(None)
        }
    }
    
    /// Show password manager UI
    pub fn show_ui(&self) {
        self.ui.show();
    }
}

pub use self::PasswordManager;