// Password Manager UI Components
use serde::{Deserialize, Serialize};

/// Password manager UI state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordUI {
    pub is_unlocked: bool,
    pub master_password_strength: PasswordStrength,
    pub entries_count: usize,
    pub recent_entries: Vec<PasswordEntryPreview>,
}

/// Password strength indicator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PasswordStrength {
    VeryWeak,
    Weak,
    Medium,
    Strong,
    VeryStrong,
}

/// Preview of password entry for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordEntryPreview {
    pub id: usize,
    pub website: String,
    pub username: String,
    pub last_used: Option<String>,
}

impl PasswordUI {
    /// Create new password UI state
    pub fn new() -> Self {
        Self {
            is_unlocked: false,
            master_password_strength: PasswordStrength::VeryWeak,
            entries_count: 0,
            recent_entries: Vec::new(),
        }
    }

    /// Update UI state when unlocked
    pub fn set_unlocked(&mut self, unlocked: bool) {
        self.is_unlocked = unlocked;
    }

    /// Update password strength indicator
    pub fn update_password_strength(&mut self, password: &str) {
        self.master_password_strength = Self::calculate_strength(password);
    }

    /// Update entries count
    pub fn set_entries_count(&mut self, count: usize) {
        self.entries_count = count;
    }

    /// Update recent entries
    pub fn set_recent_entries(&mut self, entries: Vec<PasswordEntryPreview>) {
        self.recent_entries = entries;
    }

    /// Calculate password strength
    pub fn calculate_strength(password: &str) -> PasswordStrength {
        let length = password.len();
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digits = password.chars().any(|c| c.is_numeric());
        let has_symbols = password.chars().any(|c| !c.is_alphanumeric());
        
        let score = if length >= 12 && has_uppercase && has_lowercase && has_digits && has_symbols {
            5
        } else if length >= 10 && has_uppercase && has_lowercase && (has_digits || has_symbols) {
            4
        } else if length >= 8 && has_uppercase && has_lowercase {
            3
        } else if length >= 6 {
            2
        } else {
            1
        };
        
        match score {
            5 => PasswordStrength::VeryStrong,
            4 => PasswordStrength::Strong,
            3 => PasswordStrength::Medium,
            2 => PasswordStrength::Weak,
            _ => PasswordStrength::VeryWeak,
        }
    }

    /// Generate password suggestions
    pub fn generate_suggestions(count: usize) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        for _ in 0..count {
            let password = Self::generate_secure_password(16);
            suggestions.push(password);
        }
        
        suggestions
    }

    /// Generate a secure random password
    fn generate_secure_password(length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";
        
        let _rng = rand::thread_rng();
        let mut password = String::with_capacity(length);
        
        for _ in 0..length {
            let idx = (rand::random::<u32>() as usize) % CHARSET.len();
            password.push(CHARSET[idx] as char);
        }
        
        password
    }
    
    /// Show the password manager UI
    pub fn show(&self) {
        println!("Password Manager UI:");
        println!("  Unlocked: {}", self.is_unlocked);
        println!("  Master Password Strength: {:?}", self.master_password_strength);
        println!("  Entries Count: {}", self.entries_count);
        println!("  Recent Entries: {}", self.recent_entries.len());
    }
}