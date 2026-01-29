// Password Encryption Utilities
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use pbkdf2::pbkdf2;
use rand::{rngs::OsRng, RngCore};
use ring::digest;

/// Password encryption handler
pub struct PasswordEncryption;

impl PasswordEncryption {
    /// Derive encryption key from password using PBKDF2
    pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], Box<dyn std::error::Error>> {
        let mut key = [0u8; 32];
        pbkdf2(
            digest::SHA256,
            password.as_bytes(),
            salt,
            100_000,
            &mut key,
        )
        .map_err(|_| "Key derivation failed")?;
        Ok(key)
    }

    /// Generate random salt
    pub fn generate_salt() -> [u8; 32] {
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    /// Generate random IV for AES-GCM
    pub fn generate_iv() -> [u8; 12] {
        let mut iv = [0u8; 12];
        OsRng.fill_bytes(&mut iv);
        iv
    }

    /// Encrypt password using AES-256-GCM
    pub fn encrypt_password(
        password: &str,
        key: &[u8; 32],
        iv: &[u8; 12],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| "Invalid key")?;
        let nonce = Nonce::from_slice(iv);
        let encrypted = cipher
            .encrypt(nonce, password.as_bytes())
            .map_err(|_| "Encryption failed")?;
        Ok(encrypted)
    }

    /// Decrypt password using AES-256-GCM
    pub fn decrypt_password(
        encrypted: &[u8],
        key: &[u8; 32],
        iv: &[u8; 12],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| "Invalid key")?;
        let nonce = Nonce::from_slice(iv);
        let decrypted = cipher
            .decrypt(nonce, encrypted)
            .map_err(|_| "Decryption failed")?;
        let password = String::from_utf8(decrypted)
            .map_err(|_| "Invalid UTF-8 in decrypted password")?;
        Ok(password)
    }

    /// Verify password against encrypted data
    pub fn verify_password(
        password: &str,
        encrypted: &[u8],
        key: &[u8; 32],
        iv: &[u8; 12],
    ) -> bool {
        match Self::decrypt_password(encrypted, key, iv) {
            Ok(decrypted) => decrypted == password,
            Err(_) => false,
        }
    }
}