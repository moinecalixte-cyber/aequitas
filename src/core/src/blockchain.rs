//! Blockchain state management
//!
//! Manages the chain of blocks and UTXO set.

use crate::address::Address;
use crate::block::{Block, BlockError, GENESIS_REWARD};
use crate::difficulty::{Difficulty, DIFFICULTY_AVERAGING_WINDOW, TARGET_BLOCK_TIME};
use crate::transaction::{Transaction, TxError, TxOutput};
use std::collections::HashMap;

/// Halving interval in blocks (~2 years at 30 second blocks)
pub const HALVING_INTERVAL: u64 = 2_100_000;

/// Maximum supply (210 million AEQ with 9 decimals)
pub const MAX_SUPPLY: u64 = 210_000_000_000_000_000;

/// Treasury (Dev) percentage (1%)
pub const TREASURY_PERCENTAGE: u64 = 1;

/// Solidarity (Small Miners) percentage (1%)
pub const SOLIDARITY_PERCENTAGE: u64 = 1;

/// UTXO identifier (transaction hash + output index)
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct UtxoId {
    pub tx_hash: [u8; 32],
    pub output_index: u32,
}

impl UtxoId {
    pub fn new(tx_hash: [u8; 32], output_index: u32) -> Self {
        Self {
            tx_hash,
            output_index,
        }
    }
}

/// The main blockchain structure
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Blockchain {
    blocks: HashMap<[u8; 32], Block>,
    height_index: HashMap<u64, [u8; 32]>,
    tip: [u8; 32],
    height: u64,
    utxos: HashMap<UtxoId, TxOutput>,
    block_times: Vec<(u64, i64)>,
    treasury_address: Address,
    current_difficulty: u64,
}

impl Blockchain {
    /// Create a new blockchain with genesis block
    pub fn new() -> Self {
        let genesis = Block::genesis();
        let genesis_hash = genesis.hash();

        let mut blocks = HashMap::new();
        let mut height_index = HashMap::new();
        let mut utxos = HashMap::new();

        // Add genesis block
        blocks.insert(genesis_hash, genesis.clone());
        height_index.insert(0, genesis_hash);

        // Add genesis UTXOs
        for (idx, output) in genesis.transactions[0].outputs.iter().enumerate() {
            let utxo_id = UtxoId::new(genesis.transactions[0].hash(), idx as u32);
            utxos.insert(utxo_id, output.clone());
        }

        let block_times = vec![(0, genesis.header.timestamp.timestamp())];

        Self {
            blocks,
            height_index,
            tip: genesis_hash,
            height: 0,
            utxos,
            block_times,
            treasury_address: Address::genesis_address(),
            current_difficulty: genesis.header.difficulty,
        }
    }

    /// Load from file
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read(path)?;
        let chain: Self = bincode::deserialize(&content)?;
        Ok(chain)
    }

    /// Save to file
    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = bincode::serialize(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the current height
    pub fn height(&self) -> u64 {
        self.height
    }

    /// Get the current tip hash
    pub fn tip(&self) -> [u8; 32] {
        self.tip
    }

    /// Get current difficulty
    pub fn difficulty(&self) -> u64 {
        self.current_difficulty
    }

    /// Calculate reward for a given height
    pub fn reward_for_height(height: u64) -> u64 {
        let halvings = height / HALVING_INTERVAL;
        if halvings >= 64 {
            return 0;
        }
        GENESIS_REWARD >> halvings
    }

    /// Calculate miner, treasury, and solidarity rewards
    pub fn rewards_for_height(&self, height: u64) -> (u64, u64, u64) {
        let total = Self::reward_for_height(height);
        let treasury = (total * TREASURY_PERCENTAGE) / 100;
        let solidarity = (total * SOLIDARITY_PERCENTAGE) / 100;
        let miner = total - treasury - solidarity;
        (miner, treasury, solidarity)
    }

    /// Find the smallest miner (lowest balance) among the last 100 blocks to receive solidarity
    pub fn find_smallest_beneficiary(&self) -> Address {
        let start_height = self.height.saturating_sub(100);
        let mut miner_balances: HashMap<Address, u64> = HashMap::new();

        for h in start_height..=self.height {
            if let Some(block) = self.get_block_at_height(h) {
                if let Some(coinbase) = block.transactions.get(0) {
                    // Assuming the first output of coinbase is the miner
                    if let Some(output) = coinbase.outputs.get(0) {
                        let addr = output.recipient.clone();
                        let balance = self.get_balance(&addr);
                        miner_balances.insert(addr, balance);
                    }
                }
            }
        }

        // Return the address with the lowest balance.
        // If no miners found (unlikely), return treasury as fallback.
        miner_balances
            .into_iter()
            .min_by_key(|&(_, balance)| balance)
            .map(|(addr, _)| addr)
            .unwrap_or_else(|| self.treasury_address.clone())
    }

    /// Get block by hash
    pub fn get_block(&self, hash: &[u8; 32]) -> Option<&Block> {
        self.blocks.get(hash)
    }

    /// Get block by height
    pub fn get_block_at_height(&self, height: u64) -> Option<&Block> {
        self.height_index
            .get(&height)
            .and_then(|hash| self.blocks.get(hash))
    }

    /// Get the tip block
    pub fn tip_block(&self) -> &Block {
        self.blocks.get(&self.tip).expect("Tip block must exist")
    }

    /// Check if a UTXO exists
    pub fn get_utxo(&self, utxo_id: &UtxoId) -> Option<&TxOutput> {
        self.utxos.get(utxo_id)
    }

    /// Get balance for an address
    pub fn get_balance(&self, address: &Address) -> u64 {
        self.utxos
            .values()
            .filter(|output| &output.recipient == address)
            .map(|output| output.amount)
            .sum()
    }

    /// Get UTXOs for an address
    pub fn get_utxos_for_address(&self, address: &Address) -> Vec<(UtxoId, TxOutput)> {
        self.utxos
            .iter()
            .filter(|(_, output)| &output.recipient == address)
            .map(|(id, output)| (id.clone(), output.clone()))
            .collect()
    }

    /// Calculate next difficulty
    pub fn next_difficulty(&self) -> u64 {
        Difficulty::calculate_next(self.current_difficulty, &self.block_times)
    }

    /// Validate and add a new block
    pub fn add_block(&mut self, block: Block) -> Result<(), ChainError> {
        // Check previous hash
        if block.header.prev_hash != self.tip {
            return Err(ChainError::InvalidPrevHash);
        }

        // Check height
        if block.header.height != self.height + 1 {
            return Err(ChainError::InvalidHeight);
        }

        // Check difficulty
        let expected_diff = self.next_difficulty();
        if block.header.difficulty != expected_diff {
            return Err(ChainError::InvalidDifficulty);
        }

        // Validate block structure
        block.validate()?;

        // Validate transactions
        self.validate_block_transactions(&block)?;

        // Apply block
        let block_hash = block.hash();
        let timestamp = block.header.timestamp.timestamp();

        // Update UTXO set
        for tx in &block.transactions {
            // Remove spent UTXOs
            for input in &tx.inputs {
                let utxo_id = UtxoId::new(input.prev_tx_hash, input.output_index);
                self.utxos.remove(&utxo_id);
            }

            // Add new UTXOs
            let tx_hash = tx.hash();
            for (idx, output) in tx.outputs.iter().enumerate() {
                let utxo_id = UtxoId::new(tx_hash, idx as u32);
                self.utxos.insert(utxo_id, output.clone());
            }
        }

        // Update chain state
        self.blocks.insert(block_hash, block);
        self.height_index.insert(self.height + 1, block_hash);
        self.tip = block_hash;
        self.height += 1;

        // Update block times for difficulty calculation
        self.block_times.push((self.height, timestamp));
        if self.block_times.len() > DIFFICULTY_AVERAGING_WINDOW as usize * 2 {
            self.block_times.remove(0);
        }

        // Update difficulty
        self.current_difficulty = self.next_difficulty();

        Ok(())
    }

    /// Validate all transactions in a block
    fn validate_block_transactions(&self, block: &Block) -> Result<(), ChainError> {
        if block.transactions.is_empty() {
            return Err(ChainError::NoTransactions);
        }

        // First transaction must be coinbase
        if block.transactions[0].tx_type != crate::transaction::TxType::Coinbase {
            return Err(ChainError::NoCoinbase);
        }

        // Validate coinbase amount
        let (miner_reward, treasury_reward, solidarity_reward) =
            self.rewards_for_height(block.header.height);
        let coinbase = &block.transactions[0];

        // Coinbase should have outputs for Miner, Dev Fund, and Solidarity Fund
        // Allow flexibility for genesis block which may have only 1 output
        if coinbase.outputs.len() < 3 && block.header.height > 0 {
            return Err(ChainError::InvalidCoinbaseAmount);
        }

        let total_reward = miner_reward + treasury_reward + solidarity_reward;
        let coinbase_amount: u64 = coinbase.outputs.iter().map(|o| o.amount).sum();

        if coinbase_amount > total_reward {
            return Err(ChainError::InvalidCoinbaseAmount);
        }

        // Verify Solidarity recipient is actually the smallest miner
        // Only enforce for blocks with more than 1 output (not genesis)
        if coinbase.outputs.len() > 2 {
            let expected_solidarity_recipient = self.find_smallest_beneficiary();
            let actual_solidarity_recipient = &coinbase.outputs[2].recipient;

            if actual_solidarity_recipient != &expected_solidarity_recipient
                && block.header.height > 0
            {
                log::warn!(
                    "Solidarity reward sent to wrong recipient. Expected: {}, Got: {}",
                    expected_solidarity_recipient,
                    actual_solidarity_recipient
                );
                // Allow flexibility for initial blocks
                if block.header.height > 100 {
                    return Err(ChainError::InvalidSolidarityRecipient);
                }
            }
        }

        // Validate other transactions
        let mut spent_in_block = HashMap::new();

        for (i, tx) in block.transactions.iter().enumerate() {
            if i == 0 {
                continue; // Skip coinbase
            }

            // Check for double-spends within block
            for input in &tx.inputs {
                let utxo_id = UtxoId::new(input.prev_tx_hash, input.output_index);

                if spent_in_block.contains_key(&utxo_id) {
                    return Err(ChainError::DoubleSpend);
                }

                // Check UTXO exists
                if !self.utxos.contains_key(&utxo_id) {
                    return Err(ChainError::MissingUtxo);
                }

                spent_in_block.insert(utxo_id, true);
            }

            // Validate transaction
            tx.validate()?;
        }

        Ok(())
    }

    /// Get total circulating supply
    pub fn circulating_supply(&self) -> u64 {
        self.utxos.values().map(|o| o.amount).sum()
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

/// Blockchain errors
#[derive(Debug, thiserror::Error)]
pub enum ChainError {
    #[error("Invalid previous hash")]
    InvalidPrevHash,

    #[error("Invalid block height")]
    InvalidHeight,

    #[error("Invalid difficulty")]
    InvalidDifficulty,

    #[error("Block validation failed: {0}")]
    BlockError(#[from] BlockError),

    #[error("Transaction validation failed: {0}")]
    TxError(#[from] TxError),

    #[error("Block has no transactions")]
    NoTransactions,

    #[error("First transaction must be coinbase")]
    NoCoinbase,

    #[error("Invalid coinbase amount")]
    InvalidCoinbaseAmount,

    #[error("Double spend detected")]
    DoubleSpend,

    #[error("Referenced UTXO not found")]
    MissingUtxo,

    #[error("Invalid solidarity recipient")]
    InvalidSolidarityRecipient,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_blockchain() {
        let chain = Blockchain::new();
        assert_eq!(chain.height(), 0);
        assert!(chain.circulating_supply() > 0);
    }

    #[test]
    fn test_reward_halving() {
        assert_eq!(Blockchain::reward_for_height(0), GENESIS_REWARD);
        assert_eq!(
            Blockchain::reward_for_height(HALVING_INTERVAL),
            GENESIS_REWARD / 2
        );
        assert_eq!(
            Blockchain::reward_for_height(HALVING_INTERVAL * 2),
            GENESIS_REWARD / 4
        );
    }

    #[test]
    fn test_treasury_reward() {
        let (miner, treasury) = Blockchain::rewards_for_height(0);
        assert_eq!(miner + treasury, GENESIS_REWARD);
        assert_eq!(treasury, GENESIS_REWARD * 2 / 100);
    }
}
