//! Hashrate classification and proportional solidarity rewards system
//!
//! Implements tier-based mining rewards that compensate
//! smaller miners proportionally to their real contribution,
//! while maintaining fair incentives for all participants.

use std::collections::HashMap;

/// Mining hash rate tiers for proportional rewards
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashrateTier {
    /// Small miners with consumer GPUs (< 50 GH/s)
    Petit,
    /// Medium miners with decent GPUs (50-200 GH/s)
    Moyen,
    /// Large miners with high-end GPUs (200-500 GH/s)
    Gros,
    /// Industrial mining operations (500+ GH/s)
    Industriel,
}

impl HashrateTier {
    /// Determine tier from hashrate in GH/s
    pub fn from_hashrate(hashrate_ghs: f64) -> Self {
        if hashrate_ghs < 50.0 {
            HashrateTier::Petit
        } else if hashrate_ghs < 200.0 {
            HashrateTier::Moyen
        } else if hashrate_ghs < 500.0 {
            HashrateTier::Gros
        } else if hashrate_ghs < 1000.0 {
            HashrateTier::Industriel
        } else {
            HashrateTier::UltraGros // Very large operations
        }
    }

    /// Get base mining percentage (vs 60% standard for large miners)
    pub fn base_mining_percentage(&self) -> f64 {
        match self {
            HashrateTier::Petit => 0.90,      // 90% of 60%
            HashrateTier::Moyen => 0.60,      // 60% of 60%
            HashrateTier::Gros => 0.35,       // 35% of 60%
            HashrateTier::Industriel => 0.25, // 25% of 60%
            HashrateTier::UltraGros => 0.15,  // 15% of 60%
        }
    }

    /// Get optimal solidarity percentage for this tier
    pub fn solidarity_percentage(&self) -> f64 {
        match self {
            HashrateTier::Petit => 0.35,      // 35% for strong protection
            HashrateTier::Moyen => 0.20,      // 20% for moderate support
            HashrateTier::Gros => 0.10,       // 10% for minimal support
            HashrateTier::Industriel => 0.05, // 5% for symbolic contribution
            HashrateTier::UltraGros => 0.02,  // 2% for very minimal
        }
    }

    /// Get treasury percentage for this tier
    pub fn treasury_percentage(&self) -> f64 {
        match self {
            HashrateTier::Petit => 0.05,      // 5% for development
            HashrateTier::Moyen => 0.10,      // 10% for ecosystem growth
            HashrateTier::Gros => 0.20,       // 20% for network security
            HashrateTier::Industriel => 0.25, // 25% for infrastructure
            HashrateTier::UltraGros => 0.28,  // 28% for operational costs
        }
    }
}

/// Proportional mining reward distribution
#[derive(Debug, Clone)]
pub struct ProportionalRewards {
    /// Mining percentage (adjusted by tier)
    pub mining_percentage: f64,
    /// Solidarity percentage (adjusted by tier)
    pub solidarity_percentage: f64,
    /// Treasury percentage (adjusted by tier)
    pub treasury_percentage: f64,
}

impl ProportionalRewards {
    /// Calculate optimal reward distribution based on hash rate tier
    pub fn for_hashrate(hashrate_ghs: f64) -> Self {
        let tier = HashrateTier::from_hashrate(hashrate_ghs);

        Self {
            mining_percentage: tier.base_mining_percentage(),
            solidarity_percentage: tier.solidarity_percentage(),
            treasury_percentage: tier.treasury_percentage(),
        }
    }

    /// Calculate individual rewards from block reward
    pub fn calculate_rewards(&self, block_reward: u64) -> (u64, u64, u64) {
        let miner_reward = (block_reward as f64 * self.mining_percentage) as u64;
        let solidarity_reward = (block_reward as f64 * self.solidarity_percentage) as u64;
        let treasury_reward = (block_reward as f64 * self.treasury_percentage) as u64;

        (miner_reward, solidarity_reward, treasury_reward)
    }

    /// Get tier description for display
    pub fn tier_description(&self) -> &'static str {
        match self {
            HashrateTier::Petit => "Petits mineurs - Protection renforcée",
            HashrateTier::Moyen => "Mineurs moyens - Support modéré",
            HashrateTier::Gros => "Gros mineurs - Inclusion financière",
            HashrateTier::Industriel => "Opérateurs industriels - Maintenance réseau",
            HashrateTier::UltraGros => "Très gros opérateurs - Sécurité infra",
        }
    }
}

/// Track miner contribution history for fair solidarity distribution
#[derive(Debug, Clone)]
pub struct MinerContribution {
    /// Miner identifier
    pub miner_id: String,

    /// Historical hash rate average
    pub avg_hashrate: f64,

    /// Current tier classification
    pub current_tier: HashrateTier,

    /// Blocks found in current window
    pub blocks_found: u64,

    /// Shares submitted in current window
    pub shares_submitted: u64,

    /// Last reward period timestamp
    pub last_reward_period: u64,
}

impl MinerContribution {
    /// Create new miner contribution tracker
    pub fn new(miner_id: String) -> Self {
        Self {
            miner_id,
            avg_hashrate: 0.0,
            current_tier: HashrateTier::Petit,
            blocks_found: 0,
            shares_submitted: 0,
            last_reward_period: 0,
        }
    }

    /// Update miner statistics
    pub fn update_stats(&mut self, hashrate_ghs: f64, blocks_found: u64, shares_submitted: u64) {
        // Update average hash rate with weighted average
        self.avg_hashrate = (self.avg_hashrate * 0.9) + (hashrate_ghs * 0.1);

        // Update current tier
        self.current_tier = HashrateTier::from_hashrate(self.avg_hashrate);

        // Update contribution metrics
        self.blocks_found += blocks_found;
        self.shares_submitted += shares_submitted;
    }

    /// Calculate solidarity score for fair distribution
    pub fn solidarity_score(&self) -> f64 {
        // Higher score for smaller miners with more consistent contribution
        let consistency_score = if self.blocks_found > 0 {
            (self.shares_submitted as f64 / self.blocks_found as f64).min(1.0)
        } else {
            0.0
        };

        // Tier bonus (smaller miners get higher bonus)
        let tier_bonus = match self.current_tier {
            HashrateTier::Petit => 1.0,
            HashrateTier::Moyen => 0.7,
            HashrateTier::Gros => 0.4,
            HashrateTier::Industriel => 0.2,
            HashrateTier::UltraGros => 0.1,
        };

        consistency_score * tier_bonus
    }
}

/// Global solidarity pool manager
pub struct SolidarityPool {
    /// Current period for solidarity distribution
    current_period: u64,

    /// Contributors in current period
    contributors: HashMap<String, MinerContribution>,

    /// Total hash rate in current period
    total_hashrate: f64,

    /// Small miner beneficiaries (updated each period)
    small_miner_beneficiaries: Vec<String>,
}

impl SolidarityPool {
    /// Create new solidarity pool
    pub fn new() -> Self {
        Self {
            current_period: 0,
            contributors: HashMap::new(),
            total_hashrate: 0.0,
            small_miner_beneficiaries: Vec::new(),
        }
    }

    /// Register miner contribution
    pub fn register_contributor(&mut self, miner_id: String) {
        let contribution = MinerContribution::new(miner_id);
        self.contributors.insert(miner_id, contribution);
    }

    /// Update all contributor statistics
    pub fn update_all_stats(&mut self) {
        let mut total_hashrate = 0.0;
        let mut new_small_miners = Vec::new();

        for (miner_id, contribution) in &self.contributors {
            total_hashrate += contribution.avg_hashrate;

            // Find the smallest contributing miners
            if contribution.current_tier == HashrateTier::Petit
                && contribution.solidarity_score() > 0.5
            {
                if !new_small_miners.contains(&miner_id) {
                    new_small_miners.push(miner_id.clone());
                }
            }
        }

        self.total_hashrate = total_hashrate;
        self.small_miner_beneficiaries = new_small_miners;
    }

    /// Calculate solidarity rewards for current period
    pub fn calculate_period_rewards(&self, block_reward: u64) -> Vec<(String, u64)> {
        let mut rewards = Vec::new();

        // Give 30% of solidarity pool to smallest contributing miners
        let solidarity_pool = (block_reward as f64 * 0.30) as u64;

        if !self.small_miner_beneficiaries.is_empty() {
            let reward_per_miner = solidarity_pool / self.small_miner_beneficiaries.len() as u64;

            for miner_id in &self.small_miner_beneficiaries {
                rewards.push((miner_id.clone(), reward_per_miner));
            }
        }

        rewards
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashrate_tier_classification() {
        assert_eq!(HashrateTier::from_hashrate(25.0), HashrateTier::Petit);
        assert_eq!(HashrateTier::from_hashrate(100.0), HashrateTier::Moyen);
        assert_eq!(HashrateTier::from_hashrate(300.0), HashrateTier::Gros);
        assert_eq!(HashrateTier::from_hashrate(800.0), HashrateTier::Industriel);
        assert_eq!(HashrateTier::from_hashrate(1500.0), HashrateTier::UltraGros);
    }

    #[test]
    fn test_proportional_rewards() {
        let rewards = ProportionalRewards::for_hashrate(50.0);

        assert_eq!(rewards.mining_percentage, 0.90); // Small miners get 90%
        assert_eq!(rewards.solidarity_percentage, 0.35); // Strong protection
        assert_eq!(rewards.treasury_percentage, 0.05); // Development fund
    }

    #[test]
    fn test_miner_contribution_tracking() {
        let contribution = MinerContribution::new("test_miner".to_string());
        contribution.update_stats(1000.0, 5, 500);

        assert_eq!(contribution.avg_hashrate, 900.0); // Weighted average
        assert_eq!(contribution.blocks_found, 5);
        assert_eq!(contribution.shares_submitted, 500);
        assert_eq!(contribution.solidarity_score(), 1.0); // Perfect consistency
    }
}
