//! Secure keystore with encryption
//!
//! Stores private keys encrypted with a password-derived key.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{Argon2, password_hash::SaltString};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;
use ed25519_dalek::SigningKey;
use aequitas_core::address::{Keypair, Address};
use std::path::Path;

/// Keystore version
pub const KEYSTORE_VERSION: u32 = 1;

/// Encrypted key entry
#[derive(Clone, Serialize, Deserialize)]
pub struct EncryptedKey {
    /// Address for this key
    pub address: String,
    
    /// Encrypted private key bytes
    pub ciphertext: Vec<u8>,
    
    /// Nonce used for encryption
    pub nonce: Vec<u8>,
    
    /// Salt for key derivation
    pub salt: String,
    
    /// Optional label
    pub label: Option<String>,
    
    /// Creation timestamp
    pub created_at: i64,
}

/// Keystore file format
#[derive(Clone, Serialize, Deserialize)]
pub struct KeystoreFile {
    /// Version
    pub version: u32,
    
    /// Encrypted keys
    pub keys: Vec<EncryptedKey>,
}

impl Default for KeystoreFile {
    fn default() -> Self {
        Self {
            version: KEYSTORE_VERSION,
            keys: Vec::new(),
        }
    }
}

/// Keystore manager
pub struct Keystore {
    /// Path to keystore file
    path: Option<std::path::PathBuf>,
    
    /// Keystore data
    data: KeystoreFile,
    
    /// Unlocked keys (in memory)
    unlocked: Vec<UnlockedKey>,
}

/// Unlocked key in memory
struct UnlockedKey {
    address: Address,
    keypair: Keypair,
}

impl Keystore {
    /// Create a new in-memory keystore
    pub fn new() -> Self {
        Self {
            path: None,
            data: KeystoreFile::default(),
            unlocked: Vec::new(),
        }
    }
    
    /// Load keystore from file
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let data: KeystoreFile = serde_json::from_str(&content)?;
        
        Ok(Self {
            path: Some(path.as_ref().to_path_buf()),
            data,
            unlocked: Vec::new(),
        })
    }
    
    /// Save keystore to file
    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(path) = &self.path {
            let content = serde_json::to_string_pretty(&self.data)?;
            std::fs::write(path, content)?;
        }
        Ok(())
    }
    
    /// Save keystore to a specific path
    pub fn save_to<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        self.path = Some(path.as_ref().to_path_buf());
        self.save()
    }
    
    /// Generate a new keypair and add to keystore
    pub fn generate_key(&mut self, password: &str, label: Option<String>) -> anyhow::Result<Address> {
        let keypair = Keypair::generate();
        let address = keypair.address();
        
        self.add_key(&keypair, password, label)?;
        
        Ok(address)
    }
    
    /// Import existing key
    pub fn import_key(
        &mut self,
        secret_bytes: &[u8; 32],
        password: &str,
        label: Option<String>,
    ) -> anyhow::Result<Address> {
        let keypair = Keypair::from_bytes(secret_bytes)?;
        let address = keypair.address();
        
        self.add_key(&keypair, password, label)?;
        
        Ok(address)
    }
    
    /// Add a keypair to the keystore
    fn add_key(&mut self, keypair: &Keypair, password: &str, label: Option<String>) -> anyhow::Result<()> {
        let address = keypair.address();
        let secret_bytes = keypair.to_bytes();
        
        // Derive encryption key from password
        let salt = SaltString::generate(&mut OsRng);
        let mut key_bytes = [0u8; 32];
        
        Argon2::default()
            .hash_password_into(
                password.as_bytes(),
                salt.as_str().as_bytes(),
                &mut key_bytes,
            )
            .map_err(|e| anyhow::anyhow!("Key derivation failed: {}", e))?;
        
        // Encrypt private key
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| anyhow::anyhow!("Cipher creation failed: {}", e))?;
        
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher
            .encrypt(nonce, secret_bytes.as_ref())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
        
        // Clean up sensitive data
        let mut key_bytes_clean = key_bytes;
        key_bytes_clean.zeroize();
        
        // Store encrypted key
        let encrypted = EncryptedKey {
            address: address.to_string(),
            ciphertext,
            nonce: nonce_bytes.to_vec(),
            salt: salt.to_string(),
            label,
            created_at: chrono::Utc::now().timestamp(),
        };
        
        self.data.keys.push(encrypted);
        
        Ok(())
    }
    
    /// Unlock a key with password
    pub fn unlock(&mut self, address: &Address, password: &str) -> anyhow::Result<()> {
        let address_str = address.to_string();
        
        let encrypted = self.data.keys.iter()
            .find(|k| k.address == address_str)
            .ok_or_else(|| anyhow::anyhow!("Key not found"))?
            .clone();
        
        // Derive decryption key
        let mut key_bytes = [0u8; 32];
        Argon2::default()
            .hash_password_into(
                password.as_bytes(),
                encrypted.salt.as_bytes(),
                &mut key_bytes,
            )
            .map_err(|e| anyhow::anyhow!("Key derivation failed: {}", e))?;
        
        // Decrypt private key
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| anyhow::anyhow!("Cipher creation failed: {}", e))?;
        
        let nonce = Nonce::from_slice(&encrypted.nonce);
        
        let secret_bytes = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|_| anyhow::anyhow!("Decryption failed - wrong password?"))?;
        
        // Clean up key
        let mut key_bytes_clean = key_bytes;
        key_bytes_clean.zeroize();
        
        // Create keypair from decrypted bytes
        let secret_array: [u8; 32] = secret_bytes.try_into()
            .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
        
        let keypair = Keypair::from_bytes(&secret_array)?;
        
        // Store unlocked key
        self.unlocked.push(UnlockedKey {
            address: address.clone(),
            keypair,
        });
        
        Ok(())
    }
    
    /// Lock all keys
    pub fn lock_all(&mut self) {
        self.unlocked.clear();
    }
    
    /// Check if an address is unlocked
    pub fn is_unlocked(&self, address: &Address) -> bool {
        self.unlocked.iter().any(|k| &k.address == address)
    }
    
    /// Get signing key for an address (must be unlocked)
    pub fn get_signing_key(&self, address: &Address) -> Option<&ed25519_dalek::SigningKey> {
        self.unlocked.iter()
            .find(|k| &k.address == address)
            .map(|k| k.keypair.signing_key())
    }
    
    /// List all addresses in keystore
    pub fn addresses(&self) -> Vec<String> {
        self.data.keys.iter().map(|k| k.address.clone()).collect()
    }
    
    /// Get key count
    pub fn key_count(&self) -> usize {
        self.data.keys.len()
    }
}

impl Default for Keystore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_and_unlock() {
        let mut keystore = Keystore::new();
        let password = "test_password_123";
        
        let address = keystore.generate_key(password, Some("Test".to_string())).unwrap();
        
        assert_eq!(keystore.key_count(), 1);
        assert!(!keystore.is_unlocked(&address));
        
        keystore.unlock(&address, password).unwrap();
        assert!(keystore.is_unlocked(&address));
        
        keystore.lock_all();
        assert!(!keystore.is_unlocked(&address));
    }
    
    #[test]
    fn test_wrong_password() {
        let mut keystore = Keystore::new();
        let password = "correct_password";
        
        let address = keystore.generate_key(password, None).unwrap();
        
        assert!(keystore.unlock(&address, "wrong_password").is_err());
    }
}
