//! Transaction structures for Aequitas
//!
//! Defines transactions, inputs, and outputs with signature verification.

use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use crate::address::Address;

/// Transaction input referencing a previous output
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxInput {
    /// Hash of the transaction containing the output being spent
    pub prev_tx_hash: [u8; 32],
    
    /// Index of the output in the previous transaction
    pub output_index: u32,
    
    /// Signature proving ownership
    pub signature: Vec<u8>,
    
    /// Public key for verification
    pub public_key: Vec<u8>,
}

impl TxInput {
    /// Create a new transaction input
    pub fn new(prev_tx_hash: [u8; 32], output_index: u32) -> Self {
        Self {
            prev_tx_hash,
            output_index,
            signature: Vec::new(),
            public_key: Vec::new(),
        }
    }
    
    /// Sign the input with a private key
    pub fn sign(&mut self, signing_key: &SigningKey, message: &[u8]) {
        let signature = signing_key.sign(message);
        self.signature = signature.to_bytes().to_vec();
        self.public_key = signing_key.verifying_key().to_bytes().to_vec();
    }
    
    /// Verify the signature
    pub fn verify(&self, message: &[u8]) -> Result<(), TxError> {
        if self.public_key.len() != 32 {
            return Err(TxError::InvalidPublicKey);
        }
        
        let pk_bytes: [u8; 32] = self.public_key.clone().try_into()
            .map_err(|_| TxError::InvalidPublicKey)?;
        let verifying_key = VerifyingKey::from_bytes(&pk_bytes)
            .map_err(|_| TxError::InvalidPublicKey)?;
        
        if self.signature.len() != 64 {
            return Err(TxError::InvalidSignature);
        }
        
        let sig_bytes: [u8; 64] = self.signature.clone().try_into()
            .map_err(|_| TxError::InvalidSignature)?;
        let signature = Signature::from_bytes(&sig_bytes);
        
        verifying_key.verify(message, &signature)
            .map_err(|_| TxError::InvalidSignature)
    }
}

/// Transaction output specifying recipient and amount
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxOutput {
    /// Amount in smallest unit (1 AEQ = 10^9 units)
    pub amount: u64,
    
    /// Recipient address
    pub recipient: Address,
}

impl TxOutput {
    /// Create a new output
    pub fn new(recipient: Address, amount: u64) -> Self {
        Self { amount, recipient }
    }
}

/// Transaction types
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TxType {
    /// Regular transfer transaction
    Transfer,
    
    /// Coinbase (mining reward)
    Coinbase,
    
    /// Governance vote
    Vote,
    
    /// Governance proposal
    Proposal,
}

/// A complete transaction
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction version
    pub version: u32,
    
    /// Type of transaction
    pub tx_type: TxType,
    
    /// Transaction inputs
    pub inputs: Vec<TxInput>,
    
    /// Transaction outputs
    pub outputs: Vec<TxOutput>,
    
    /// Timestamp
    pub timestamp: i64,
    
    /// Optional memo/data field (max 256 bytes)
    pub memo: Vec<u8>,
}

impl Transaction {
    /// Create a new transfer transaction
    pub fn new_transfer(inputs: Vec<TxInput>, outputs: Vec<TxOutput>) -> Self {
        Self {
            version: 1,
            tx_type: TxType::Transfer,
            inputs,
            outputs,
            timestamp: chrono::Utc::now().timestamp(),
            memo: Vec::new(),
        }
    }
    
    /// Create a coinbase transaction (mining reward)
    pub fn coinbase(recipient: Address, reward: u64, height: u64) -> Self {
        // Coinbase has no inputs, only outputs
        // Include height in memo to ensure unique hash
        let memo = format!("Aequitas Block {}", height).into_bytes();
        
        Self {
            version: 1,
            tx_type: TxType::Coinbase,
            inputs: Vec::new(),
            outputs: vec![TxOutput::new(recipient, reward)],
            timestamp: chrono::Utc::now().timestamp(),
            memo,
        }
    }
    
    /// Compute transaction hash
    pub fn hash(&self) -> [u8; 32] {
        let serialized = bincode::serialize(self).expect("Failed to serialize tx");
        let mut hasher = Keccak256::new();
        hasher.update(&serialized);
        hasher.finalize().into()
    }
    
    /// Get the message to sign (excludes signatures)
    pub fn signing_message(&self) -> Vec<u8> {
        let mut msg = Vec::new();
        msg.extend_from_slice(&self.version.to_le_bytes());
        
        for input in &self.inputs {
            msg.extend_from_slice(&input.prev_tx_hash);
            msg.extend_from_slice(&input.output_index.to_le_bytes());
        }
        
        for output in &self.outputs {
            msg.extend_from_slice(&output.amount.to_le_bytes());
            msg.extend_from_slice(&output.recipient.to_bytes());
        }
        
        msg.extend_from_slice(&self.timestamp.to_le_bytes());
        msg.extend_from_slice(&self.memo);
        
        msg
    }
    
    /// Validate the transaction
    pub fn validate(&self) -> Result<(), TxError> {
        // Coinbase transactions have special rules
        if self.tx_type == TxType::Coinbase {
            if !self.inputs.is_empty() {
                return Err(TxError::CoinbaseWithInputs);
            }
            if self.outputs.is_empty() {
                return Err(TxError::NoOutputs);
            }
            return Ok(());
        }
        
        // Regular transactions must have inputs and outputs
        if self.inputs.is_empty() {
            return Err(TxError::NoInputs);
        }
        if self.outputs.is_empty() {
            return Err(TxError::NoOutputs);
        }
        
        // Verify all signatures
        let message = self.signing_message();
        for input in &self.inputs {
            input.verify(&message)?;
        }
        
        // Check memo size
        if self.memo.len() > 256 {
            return Err(TxError::MemoTooLarge);
        }
        
        Ok(())
    }
    
    /// Calculate total output amount
    pub fn total_output(&self) -> u64 {
        self.outputs.iter().map(|o| o.amount).sum()
    }
}

/// Transaction validation errors
#[derive(Debug, thiserror::Error)]
pub enum TxError {
    #[error("Transaction has no inputs")]
    NoInputs,
    
    #[error("Transaction has no outputs")]
    NoOutputs,
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Invalid public key")]
    InvalidPublicKey,
    
    #[error("Memo too large (max 256 bytes)")]
    MemoTooLarge,
    
    #[error("Coinbase transaction should not have inputs")]
    CoinbaseWithInputs,
    
    #[error("Insufficient funds")]
    InsufficientFunds,
    
    #[error("Double spend detected")]
    DoubleSpend,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_coinbase_transaction() {
        let addr = Address::genesis_address();
        let tx = Transaction::coinbase(addr, 50_000_000_000, 0);
        
        assert_eq!(tx.tx_type, TxType::Coinbase);
        assert!(tx.inputs.is_empty());
        assert_eq!(tx.outputs.len(), 1);
        assert!(tx.validate().is_ok());
    }
    
    #[test]
    fn test_transaction_hash_deterministic() {
        let addr = Address::genesis_address();
        let tx = Transaction::coinbase(addr, 50_000_000_000, 0);
        
        let hash1 = tx.hash();
        let hash2 = tx.hash();
        assert_eq!(hash1, hash2);
    }
}
