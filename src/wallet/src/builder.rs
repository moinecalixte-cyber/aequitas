//! Transaction builder
//!
//! Fluent API for constructing and signing transactions.

use aequitas_core::{Address, Transaction, TxInput, TxOutput, Blockchain};
use aequitas_core::blockchain::UtxoId;
use aequitas_core::transaction::TxType;
use ed25519_dalek::SigningKey;

/// Minimum transaction fee (in smallest units)
pub const MIN_FEE: u64 = 1000; // 0.000001 AEQ

/// Fee per byte (for fee estimation)
pub const FEE_PER_BYTE: u64 = 10;

/// Transaction builder
pub struct TransactionBuilder {
    /// Sender address
    from: Option<Address>,
    
    /// Recipients and amounts
    outputs: Vec<(Address, u64)>,
    
    /// Explicit fee (if not set, calculated automatically)
    fee: Option<u64>,
    
    /// Memo data
    memo: Vec<u8>,
}

impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new() -> Self {
        Self {
            from: None,
            outputs: Vec::new(),
            fee: None,
            memo: Vec::new(),
        }
    }
    
    /// Set sender address
    pub fn from(mut self, address: Address) -> Self {
        self.from = Some(address);
        self
    }
    
    /// Add recipient
    pub fn to(mut self, address: Address, amount: u64) -> Self {
        self.outputs.push((address, amount));
        self
    }
    
    /// Add multiple recipients
    pub fn to_many(mut self, recipients: Vec<(Address, u64)>) -> Self {
        self.outputs.extend(recipients);
        self
    }
    
    /// Set explicit fee
    pub fn fee(mut self, fee: u64) -> Self {
        self.fee = Some(fee);
        self
    }
    
    /// Set memo
    pub fn memo(mut self, memo: Vec<u8>) -> Self {
        self.memo = memo;
        self
    }
    
    /// Set memo from string
    pub fn memo_str(mut self, memo: &str) -> Self {
        self.memo = memo.as_bytes().to_vec();
        self
    }
    
    /// Calculate total output amount
    pub fn total_output(&self) -> u64 {
        self.outputs.iter().map(|(_, a)| a).sum()
    }
    
    /// Estimate transaction size
    pub fn estimate_size(&self) -> usize {
        // Base tx size + inputs + outputs
        100 + self.outputs.len() * 40 + self.memo.len()
    }
    
    /// Estimate fee for this transaction
    pub fn estimate_fee(&self) -> u64 {
        let size = self.estimate_size();
        let calculated = (size as u64 * FEE_PER_BYTE).max(MIN_FEE);
        self.fee.unwrap_or(calculated)
    }
    
    /// Build and sign the transaction
    pub fn build_and_sign(
        self,
        signing_key: &SigningKey,
        chain: &Blockchain,
    ) -> anyhow::Result<Transaction> {
        let from = self.from.ok_or_else(|| anyhow::anyhow!("Sender address not set"))?;
        
        if self.outputs.is_empty() {
            anyhow::bail!("No recipients specified");
        }
        
        // Get UTXOs for sender
        let utxos = chain.get_utxos_for_address(&from);
        if utxos.is_empty() {
            anyhow::bail!("No UTXOs found for sender");
        }
        
        // Calculate required amount (outputs + fee)
        let fee = self.estimate_fee();
        let total_needed = self.total_output() + fee;
        
        // Select UTXOs (simple greedy selection)
        let mut selected_utxos: Vec<(UtxoId, TxOutput)> = Vec::new();
        let mut selected_amount: u64 = 0;
        
        for (utxo_id, output) in utxos {
            selected_utxos.push((utxo_id, output.clone()));
            selected_amount += output.amount;
            
            if selected_amount >= total_needed {
                break;
            }
        }
        
        if selected_amount < total_needed {
            anyhow::bail!(
                "Insufficient funds: have {} but need {}",
                selected_amount,
                total_needed
            );
        }
        
        // Create inputs
        let mut inputs: Vec<TxInput> = selected_utxos.iter()
            .map(|(utxo_id, _)| TxInput::new(utxo_id.tx_hash, utxo_id.output_index))
            .collect();
        
        // Create outputs
        let mut tx_outputs: Vec<TxOutput> = self.outputs.iter()
            .map(|(addr, amount)| TxOutput::new(addr.clone(), *amount))
            .collect();
        
        // Add change output if needed
        let change = selected_amount - total_needed;
        if change > 0 {
            tx_outputs.push(TxOutput::new(from.clone(), change));
        }
        
        // Create transaction
        let mut tx = Transaction {
            version: 1,
            tx_type: TxType::Transfer,
            inputs,
            outputs: tx_outputs,
            timestamp: chrono::Utc::now().timestamp(),
            memo: self.memo,
        };
        
        // Sign each input
        let message = tx.signing_message();
        for input in &mut tx.inputs {
            input.sign(signing_key, &message);
        }
        
        Ok(tx)
    }
    
    /// Build an unsigned transaction (for multi-sig or external signing)
    pub fn build_unsigned(
        self,
        chain: &Blockchain,
    ) -> anyhow::Result<UnsignedTransaction> {
        let from = self.from.ok_or_else(|| anyhow::anyhow!("Sender address not set"))?;
        
        if self.outputs.is_empty() {
            anyhow::bail!("No recipients specified");
        }
        
        let utxos = chain.get_utxos_for_address(&from);
        let fee = self.estimate_fee();
        let total_needed = self.total_output() + fee;
        
        // Select UTXOs
        let mut selected_utxos: Vec<(UtxoId, TxOutput)> = Vec::new();
        let mut selected_amount: u64 = 0;
        
        for (utxo_id, output) in utxos {
            selected_utxos.push((utxo_id, output.clone()));
            selected_amount += output.amount;
            
            if selected_amount >= total_needed {
                break;
            }
        }
        
        if selected_amount < total_needed {
            anyhow::bail!("Insufficient funds");
        }
        
        // Build transaction
        let inputs: Vec<TxInput> = selected_utxos.iter()
            .map(|(utxo_id, _)| TxInput::new(utxo_id.tx_hash, utxo_id.output_index))
            .collect();
        
        let mut tx_outputs: Vec<TxOutput> = self.outputs.iter()
            .map(|(addr, amount)| TxOutput::new(addr.clone(), *amount))
            .collect();
        
        let change = selected_amount - total_needed;
        if change > 0 {
            tx_outputs.push(TxOutput::new(from.clone(), change));
        }
        
        let tx = Transaction {
            version: 1,
            tx_type: TxType::Transfer,
            inputs,
            outputs: tx_outputs,
            timestamp: chrono::Utc::now().timestamp(),
            memo: self.memo,
        };
        
        Ok(UnsignedTransaction {
            transaction: tx,
            signing_message: Vec::new(), // Will be computed when signing
        })
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Unsigned transaction for external signing
pub struct UnsignedTransaction {
    /// The unsigned transaction
    pub transaction: Transaction,
    
    /// Message to sign
    pub signing_message: Vec<u8>,
}

impl UnsignedTransaction {
    /// Get the message that needs to be signed
    pub fn get_signing_message(&self) -> Vec<u8> {
        self.transaction.signing_message()
    }
    
    /// Add a signature to an input
    pub fn add_signature(&mut self, input_index: usize, signature: Vec<u8>, public_key: Vec<u8>) {
        if let Some(input) = self.transaction.inputs.get_mut(input_index) {
            input.signature = signature;
            input.public_key = public_key;
        }
    }
    
    /// Check if fully signed
    pub fn is_fully_signed(&self) -> bool {
        self.transaction.inputs.iter().all(|i| !i.signature.is_empty())
    }
    
    /// Convert to signed transaction
    pub fn into_transaction(self) -> Transaction {
        self.transaction
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_builder_creation() {
        let builder = TransactionBuilder::new()
            .fee(1000)
            .memo_str("Test");
        
        assert_eq!(builder.fee, Some(1000));
        assert_eq!(builder.memo, b"Test");
    }
    
    #[test]
    fn test_fee_estimation() {
        let builder = TransactionBuilder::new()
            .to(Address::genesis_address(), 1_000_000_000);
        
        let fee = builder.estimate_fee();
        assert!(fee >= MIN_FEE);
    }
}
