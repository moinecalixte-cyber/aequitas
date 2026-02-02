//! Wallet implementation
//!
//! High-level wallet interface for managing keys and creating transactions.

use aequitas_core::{Address, Transaction, TxInput, TxOutput, Blockchain};
use aequitas_core::blockchain::UtxoId;
use crate::keystore::Keystore;
use crate::builder::TransactionBuilder;
use std::path::Path;

/// Wallet for managing keys and transactions
pub struct Wallet {
    /// Keystore for encrypted key storage
    keystore: Keystore,
    
    /// Default address for receiving
    default_address: Option<Address>,
}

impl Wallet {
    /// Create a new wallet
    pub fn new() -> Self {
        Self {
            keystore: Keystore::new(),
            default_address: None,
        }
    }
    
    /// Create wallet from existing keystore
    pub fn from_keystore(keystore: Keystore) -> Self {
        Self {
            keystore,
            default_address: None,
        }
    }
    
    /// Load wallet from file
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let keystore = Keystore::load(path)?;
        Ok(Self::from_keystore(keystore))
    }
    
    /// Save wallet to file
    pub fn save(&self) -> anyhow::Result<()> {
        self.keystore.save()
    }
    
    /// Save wallet to specific path
    pub fn save_to<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        self.keystore.save_to(path)
    }
    
    /// Generate a new address
    pub fn new_address(&mut self, password: &str, label: Option<String>) -> anyhow::Result<Address> {
        let addr = self.keystore.generate_key(password, label)?;
        
        if self.default_address.is_none() {
            self.default_address = Some(addr.clone());
        }
        
        Ok(addr)
    }
    
    /// Import a private key (hex string)
    pub fn import_private_key(
        &mut self,
        hex_key: &str,
        password: &str,
        label: Option<String>,
    ) -> anyhow::Result<Address> {
        let key_bytes = hex::decode(hex_key)?;
        if key_bytes.len() != 32 {
            anyhow::bail!("Invalid key length");
        }
        
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&key_bytes);
        
        self.keystore.import_key(&bytes, password, label)
    }
    
    /// Get default address
    pub fn default_address(&self) -> Option<&Address> {
        self.default_address.as_ref()
    }
    
    /// Set default address
    pub fn set_default_address(&mut self, address: Address) {
        self.default_address = Some(address);
    }
    
    /// Get all addresses
    pub fn addresses(&self) -> Vec<String> {
        self.keystore.addresses()
    }
    
    /// Unlock an address
    pub fn unlock(&mut self, address: &Address, password: &str) -> anyhow::Result<()> {
        self.keystore.unlock(address, password)
    }
    
    /// Lock all addresses
    pub fn lock(&mut self) {
        self.keystore.lock_all()
    }
    
    /// Check if address is unlocked
    pub fn is_unlocked(&self, address: &Address) -> bool {
        self.keystore.is_unlocked(address)
    }
    
    /// Get balance for an address from blockchain
    pub fn get_balance(&self, address: &Address, chain: &Blockchain) -> u64 {
        chain.get_balance(address)
    }
    
    /// Get total balance across all addresses
    pub fn total_balance(&self, chain: &Blockchain) -> u64 {
        self.addresses()
            .iter()
            .filter_map(|addr_str| Address::from_string(addr_str).ok())
            .map(|addr| chain.get_balance(&addr))
            .sum()
    }
    
    /// Create a transaction
    pub fn create_transaction(
        &self,
        from: &Address,
        to: &Address,
        amount: u64,
        chain: &Blockchain,
    ) -> anyhow::Result<Transaction> {
        if !self.keystore.is_unlocked(from) {
            anyhow::bail!("Address is not unlocked");
        }
        
        let signing_key = self.keystore.get_signing_key(from)
            .ok_or_else(|| anyhow::anyhow!("Signing key not found"))?;
        
        TransactionBuilder::new()
            .from(from.clone())
            .to(to.clone(), amount)
            .build_and_sign(signing_key, chain)
    }
    
    /// Create a transaction with custom fee
    pub fn create_transaction_with_fee(
        &self,
        from: &Address,
        to: &Address,
        amount: u64,
        fee: u64,
        chain: &Blockchain,
    ) -> anyhow::Result<Transaction> {
        if !self.keystore.is_unlocked(from) {
            anyhow::bail!("Address is not unlocked");
        }
        
        let signing_key = self.keystore.get_signing_key(from)
            .ok_or_else(|| anyhow::anyhow!("Signing key not found"))?;
        
        TransactionBuilder::new()
            .from(from.clone())
            .to(to.clone(), amount)
            .fee(fee)
            .build_and_sign(signing_key, chain)
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

/// Wallet balance info
#[derive(Clone, Debug)]
pub struct BalanceInfo {
    /// Confirmed balance
    pub confirmed: u64,
    
    /// Unconfirmed balance (pending transactions)
    pub unconfirmed: u64,
    
    /// Total balance
    pub total: u64,
    
    /// Number of UTXOs
    pub utxo_count: usize,
}

impl BalanceInfo {
    /// Create from blockchain state
    pub fn from_chain(address: &Address, chain: &Blockchain) -> Self {
        let utxos = chain.get_utxos_for_address(address);
        let confirmed: u64 = utxos.iter().map(|(_, o)| o.amount).sum();
        
        Self {
            confirmed,
            unconfirmed: 0, // TODO: Track mempool
            total: confirmed,
            utxo_count: utxos.len(),
        }
    }
}

/// Format balance for display (9 decimal places)
pub fn format_balance(amount: u64) -> String {
    let whole = amount / 1_000_000_000;
    let frac = amount % 1_000_000_000;
    
    if frac == 0 {
        format!("{} AEQ", whole)
    } else {
        format!("{}.{:09} AEQ", whole, frac)
    }
}

/// Parse balance from string
pub fn parse_balance(s: &str) -> anyhow::Result<u64> {
    let s = s.trim().to_lowercase().replace(" aeq", "").replace("aeq", "");
    
    if let Some(dot_pos) = s.find('.') {
        let whole: u64 = s[..dot_pos].parse()?;
        let frac_str = &s[dot_pos + 1..];
        let frac_len = frac_str.len().min(9);
        let frac: u64 = frac_str[..frac_len].parse()?;
        let multiplier = 10u64.pow(9 - frac_len as u32);
        
        Ok(whole * 1_000_000_000 + frac * multiplier)
    } else {
        let whole: u64 = s.parse()?;
        Ok(whole * 1_000_000_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_balance() {
        assert_eq!(format_balance(0), "0 AEQ");
        assert_eq!(format_balance(1_000_000_000), "1 AEQ");
        assert_eq!(format_balance(50_000_000_000), "50 AEQ");
        assert_eq!(format_balance(1_500_000_000), "1.500000000 AEQ");
    }
    
    #[test]
    fn test_parse_balance() {
        assert_eq!(parse_balance("1 AEQ").unwrap(), 1_000_000_000);
        assert_eq!(parse_balance("50").unwrap(), 50_000_000_000);
        assert_eq!(parse_balance("1.5").unwrap(), 1_500_000_000);
        assert_eq!(parse_balance("0.000000001").unwrap(), 1);
    }
    
    #[test]
    fn test_wallet_creation() {
        let mut wallet = Wallet::new();
        let password = "test123";
        
        let addr = wallet.new_address(password, Some("Main".to_string())).unwrap();
        assert!(wallet.addresses().len() == 1);
        
        wallet.unlock(&addr, password).unwrap();
        assert!(wallet.is_unlocked(&addr));
    }
}
