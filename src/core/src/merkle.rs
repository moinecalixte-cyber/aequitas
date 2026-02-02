//! Merkle tree implementation for Aequitas
//!
//! Computes merkle roots for transaction sets in blocks.

use sha3::{Digest, Keccak256};
use crate::transaction::Transaction;

/// Compute the merkle root of a list of transactions
pub fn compute_merkle_root(transactions: &[Transaction]) -> [u8; 32] {
    if transactions.is_empty() {
        return [0u8; 32];
    }
    
    // Get hashes of all transactions
    let mut hashes: Vec<[u8; 32]> = transactions
        .iter()
        .map(|tx| tx.hash())
        .collect();
    
    // Build merkle tree
    while hashes.len() > 1 {
        let mut next_level = Vec::new();
        
        for chunk in hashes.chunks(2) {
            let combined = if chunk.len() == 2 {
                hash_pair(&chunk[0], &chunk[1])
            } else {
                // Odd number of nodes: duplicate the last one
                hash_pair(&chunk[0], &chunk[0])
            };
            next_level.push(combined);
        }
        
        hashes = next_level;
    }
    
    hashes[0]
}

/// Hash two 32-byte values together
fn hash_pair(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(a);
    hasher.update(b);
    hasher.finalize().into()
}

/// Merkle proof for a transaction
#[derive(Clone, Debug)]
pub struct MerkleProof {
    /// The leaf hash (transaction hash)
    pub leaf: [u8; 32],
    
    /// Proof path (sibling hashes)
    pub path: Vec<([u8; 32], bool)>, // (hash, is_right)
}

impl MerkleProof {
    /// Verify the proof against a root
    pub fn verify(&self, root: &[u8; 32]) -> bool {
        let mut current = self.leaf;
        
        for (sibling, is_right) in &self.path {
            current = if *is_right {
                hash_pair(&current, sibling)
            } else {
                hash_pair(sibling, &current)
            };
        }
        
        &current == root
    }
}

/// Build a merkle proof for a transaction at a given index
pub fn build_merkle_proof(transactions: &[Transaction], index: usize) -> Option<MerkleProof> {
    if index >= transactions.len() {
        return None;
    }
    
    let mut hashes: Vec<[u8; 32]> = transactions
        .iter()
        .map(|tx| tx.hash())
        .collect();
    
    let leaf = hashes[index];
    let mut path = Vec::new();
    let mut current_index = index;
    
    while hashes.len() > 1 {
        let mut next_level = Vec::new();
        let mut next_index = current_index / 2;
        
        for i in (0..hashes.len()).step_by(2) {
            let left = hashes[i];
            let right = if i + 1 < hashes.len() {
                hashes[i + 1]
            } else {
                hashes[i]
            };
            
            // If this pair contains our current index, record the sibling
            if i == current_index || i + 1 == current_index {
                if i == current_index {
                    path.push((right, true));
                } else {
                    path.push((left, false));
                }
            }
            
            next_level.push(hash_pair(&left, &right));
        }
        
        hashes = next_level;
        current_index = next_index;
    }
    
    Some(MerkleProof { leaf, path })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::Address;
    
    #[test]
    fn test_empty_merkle_root() {
        let root = compute_merkle_root(&[]);
        assert_eq!(root, [0u8; 32]);
    }
    
    #[test]
    fn test_single_transaction_merkle() {
        let addr = Address::genesis_address();
        let tx = Transaction::coinbase(addr, 50_000_000_000, 0);
        let root = compute_merkle_root(&[tx.clone()]);
        
        assert_eq!(root, tx.hash());
    }
    
    #[test]
    fn test_merkle_proof() {
        let addr = Address::genesis_address();
        let txs: Vec<Transaction> = (0..4)
            .map(|i| Transaction::coinbase(addr.clone(), 50_000_000_000, i))
            .collect();
        
        let root = compute_merkle_root(&txs);
        
        for i in 0..txs.len() {
            let proof = build_merkle_proof(&txs, i).unwrap();
            assert!(proof.verify(&root));
        }
    }
}
