//! Address management for Aequitas
//!
//! Addresses are derived from Ed25519 public keys using Keccak256 hashing.

use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

/// Address prefix for Aequitas mainnet
pub const ADDRESS_PREFIX: &str = "aeq";

/// Address length (20 bytes + 4 byte checksum)
pub const ADDRESS_LENGTH: usize = 24;

/// An Aequitas address
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address {
    /// Raw address bytes (20 bytes)
    bytes: [u8; 20],
}

impl Address {
    /// Create an address from raw bytes
    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        Self { bytes }
    }
    
    /// Create an address from a public key
    pub fn from_public_key(public_key: &VerifyingKey) -> Self {
        let pk_bytes = public_key.to_bytes();
        let mut hasher = Keccak256::new();
        hasher.update(&pk_bytes);
        let hash = hasher.finalize();
        
        // Take last 20 bytes of hash
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&hash[12..32]);
        
        Self { bytes }
    }
    
    /// Get the genesis address (for genesis block reward)
    pub fn genesis_address() -> Self {
        // Deterministic genesis address
        let mut bytes = [0u8; 20];
        let genesis_hash = Keccak256::digest(b"Aequitas Genesis 2026");
        bytes.copy_from_slice(&genesis_hash[12..32]);
        Self { bytes }
    }
    
    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.bytes.to_vec()
    }
    
    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.bytes
    }
    
    /// Compute checksum for the address
    fn checksum(&self) -> [u8; 4] {
        let mut hasher = Keccak256::new();
        hasher.update(&self.bytes);
        let hash = hasher.finalize();
        [hash[0], hash[1], hash[2], hash[3]]
    }
    
    /// Convert to human-readable string format
    /// Format: aeq1<base58_of_bytes_and_checksum>
    pub fn to_string_format(&self) -> String {
        let mut full_bytes = [0u8; ADDRESS_LENGTH];
        full_bytes[..20].copy_from_slice(&self.bytes);
        full_bytes[20..24].copy_from_slice(&self.checksum());
        
        format!("{}1{}", ADDRESS_PREFIX, bs58::encode(&full_bytes).into_string())
    }
    
    /// Parse from string format
    pub fn from_string(s: &str) -> Result<Self, AddressError> {
        if !s.starts_with(&format!("{}1", ADDRESS_PREFIX)) {
            return Err(AddressError::InvalidPrefix);
        }
        
        let encoded = &s[4..]; // Skip "aeq1"
        let decoded = bs58::decode(encoded)
            .into_vec()
            .map_err(|_| AddressError::InvalidEncoding)?;
        
        if decoded.len() != ADDRESS_LENGTH {
            return Err(AddressError::InvalidLength);
        }
        
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&decoded[..20]);
        
        let addr = Self { bytes };
        
        // Verify checksum
        let expected_checksum = addr.checksum();
        if decoded[20..24] != expected_checksum {
            return Err(AddressError::InvalidChecksum);
        }
        
        Ok(addr)
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_format())
    }
}

/// A keypair for signing transactions
#[derive(Clone)]
pub struct Keypair {
    /// The signing (private) key
    signing_key: SigningKey,
    
    /// The verifying (public) key
    verifying_key: VerifyingKey,
}

impl Keypair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        
        Self {
            signing_key,
            verifying_key,
        }
    }
    
    /// Create from existing signing key bytes
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, AddressError> {
        let signing_key = SigningKey::from_bytes(bytes);
        let verifying_key = signing_key.verifying_key();
        
        Ok(Self {
            signing_key,
            verifying_key,
        })
    }
    
    /// Get the address for this keypair
    pub fn address(&self) -> Address {
        Address::from_public_key(&self.verifying_key)
    }
    
    /// Get the signing key
    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }
    
    /// Get the verifying key
    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying_key
    }
    
    /// Export private key bytes
    pub fn to_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }
}

/// Address-related errors
#[derive(Debug, thiserror::Error)]
pub enum AddressError {
    #[error("Invalid address prefix")]
    InvalidPrefix,
    
    #[error("Invalid address encoding")]
    InvalidEncoding,
    
    #[error("Invalid address length")]
    InvalidLength,
    
    #[error("Invalid checksum")]
    InvalidChecksum,
    
    #[error("Invalid private key")]
    InvalidPrivateKey,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_address_generation() {
        let keypair = Keypair::generate();
        let address = keypair.address();
        
        let string_form = address.to_string_format();
        assert!(string_form.starts_with("aeq1"));
        
        let parsed = Address::from_string(&string_form).unwrap();
        assert_eq!(address, parsed);
    }
    
    #[test]
    fn test_genesis_address() {
        let addr1 = Address::genesis_address();
        let addr2 = Address::genesis_address();
        assert_eq!(addr1, addr2);
    }
    
    #[test]
    fn test_keypair_deterministic() {
        let bytes = [42u8; 32];
        let kp1 = Keypair::from_bytes(&bytes).unwrap();
        let kp2 = Keypair::from_bytes(&bytes).unwrap();
        assert_eq!(kp1.address(), kp2.address());
    }
}
