//! AequiHash - GPU-Optimized ASIC-Resistant PoW Algorithm
//!
//! Trust-based GPU optimization for consumer hardware fairness
//! - Memory-hard but GPU-friendly (2-4GB VRAM)
//! - Auto-detects optimal settings for ANY graphics card
//! - ASIC-resistant through algorithmic complexity
//! - Optimized for RTX series, AMD RDNA, Intel Arc, and integrated graphics

use blake3::Hasher;
use sha3::{Digest, Keccak256};
use std::arch::x86_64::_mm256_shuffle_epi8;
use super::gpu_config::GpuConfig;
use std::mem;

/// Epoch length in blocks (changes DAG every ~2 hours)
pub const EPOCH_LENGTH: u64 = 240;

/// DAG size in bytes (~4GB for RTX 3060 compatibility)
pub const DAG_SIZE: usize = 4 * 1024 * 1024 * 1024; // 4 GB

/// Cache size for light verification
pub const CACHE_SIZE: usize = 64 * 1024 * 1024; // 64 MB

/// Number of mixing rounds
pub const MIX_ROUNDS: usize = 64;

/// Mix output size in 32-bit words
pub const MIX_WORDS: usize = 32;

/// Number of dataset accesses per hash
pub const DATASET_ACCESSES: usize = 64;

/// Algorithm variants for random program
#[derive(Clone, Copy, Debug)]
pub enum MathOp {
    Add,
    Mul,
    Sub,
    Xor,
    RotL,
    RotR,
    And,
    Or,
}

impl MathOp {
    /// Get operation from seed byte
    pub fn from_seed(seed: u8) -> Self {
        match seed % 8 {
            0 => MathOp::Add,
            1 => MathOp::Mul,
            2 => MathOp::Sub,
            3 => MathOp::Xor,
            4 => MathOp::RotL,
            5 => MathOp::RotR,
            6 => MathOp::And,
            _ => MathOp::Or,
        }
    }

    /// Execute the operation
    pub fn execute(&self, a: u32, b: u32) -> u32 {
        match self {
            MathOp::Add => a.wrapping_add(b),
            MathOp::Mul => a.wrapping_mul(b),
            MathOp::Sub => a.wrapping_sub(b),
            MathOp::Xor => a ^ b,
            MathOp::RotL => a.rotate_left((b % 32) as u32),
            MathOp::RotR => a.rotate_right((b % 32) as u32),
            MathOp::And => a & b,
            MathOp::Or => a | b,
        }
    }
}

/// AequiHash algorithm instance
pub struct AequiHash {
    /// Current epoch
    epoch: u64,

    /// Cached seed for the epoch
    seed: [u8; 32],

    /// Random number generator for the epoch
    rng: ChaCha20Rng,

    /// Precomputed operation sequence for the epoch
    operations: Vec<MathOp>,
}

impl AequiHash {
    /// Create a new AequiHash instance with GPU-optimized initialization
    pub fn new(epoch: u64) -> Self {
        let seed = Self::compute_epoch_seed(epoch);

        // GPU-optimized operation sequence generation
        let operations = Self::generate_gpu_optimized_ops(seed);

        Self {
            epoch,
            seed,
            // Use deterministic but GPU-friendly RNG
            rng: ChaCha20Rng::from_seed(seed),
            operations,
        }
    }

    /// Generate GPU-optimized operation sequence
    fn generate_gpu_optimized_ops(seed: [u8; 32]) -> Vec<MathOp> {
        let mut ops = Vec::with_capacity(MIX_ROUNDS);

        // Create GPU-friendly sequence patterns
        for i in 0..MIX_ROUNDS {
            // Use predictable but effective pattern for GPU optimization
            let seed_byte = seed[i % 32];
            let cycle_pattern = (i / 8) % 4;

            let op = match cycle_pattern {
                0 => MathOp::Add,  // Addition-heavy sequences
                1 => MathOp::Mul,  // Multiplication phases
                2 => MathOp::Xor,  // XOR mixing
                3 => MathOp::RotL, // Rotation for diffusion
                _ => MathOp::Add,
            };

            ops.push(op);
        }

        ops
    }

    /// Compute the seed for an epoch
    pub fn compute_epoch_seed(epoch: u64) -> [u8; 32] {
        let mut hasher = Keccak256::new();
        hasher.update(b"AequiHash Epoch Seed");
        hasher.update(&epoch.to_le_bytes());
        hasher.finalize().into()
    }

    /// Get epoch from block height
    pub fn epoch_from_height(height: u64) -> u64 {
        height / EPOCH_LENGTH
    }

    /// GPU-optimized hash computation with automatic hardware adaptation
    pub fn hash_light_optimized(&self, header_hash: &[u8; 32], nonce: u64, cache: &[u32]) -> [u8; 32] {
        // Auto-detect optimal parameters
        let gpu_config = GpuConfig::detect();
        
        // Use GPU-optimized mixing strategy
        let mut mix = Self::gpu_optimized_initial_mix(header_hash, nonce, &gpu_config);
        
        // GPU-parallelizable memory mixing
        Self::gpu_parallel_mix(&mut mix, cache, &self.operations, &gpu_config);
        
        // GPU-friendly final compression
        Self::gpu_final_compression(header_hash, nonce, &mix, &gpu_config)
    }
    
    /// GPU-optimized initial mixing phase
    fn gpu_optimized_initial_mix(header_hash: &[u8; 32], nonce: u64, gpu_config: &GpuConfig) -> [u32; MIX_WORDS] {
        let mut mix = [0u32; MIX_WORDS];
        
        // Combine header and nonce for seed
        let mut seed_hasher = Keccak256::new();
        seed_hasher.update(header_hash);
        seed_hasher.update(&nonce.to_le_bytes());
        let seed_hash = seed_hasher.finalize();
        
        // GPU-friendly parallel initialization
        for i in 0..8 {
            mix[i] = u32::from_le_bytes([
                seed_hash[i * 4],
                seed_hash[i * 4 + 1], 
                seed_hash[i * 4 + 2],
                seed_hash[i * 4 + 3]
            ]);
            
            // Apply GPU-optimized constants
            if gpu_config.supports_avx2 {
                mix[i + 8] = mix[i].wrapping_mul(0x1000183);
                mix[i + 16] = mix[i].wrapping_mul(0x6be03b46);
            } else {
                mix[i + 8] = mix[i].wrapping_mul(0x85ebca6b);
                mix[i + 24] = mix[i].wrapping_mul(0xc2b2ae35);
            }
        }
        
        mix
    }
    
    /// GPU-parallelizable memory mixing
    fn gpu_parallel_mix(mix: &mut [u32; MIX_WORDS], cache: &[u32], operations: &[MathOp], gpu_config: &GpuConfig) {
        let batch_size = gpu_config.optimal_batch_size();
        
        for round in 0..MIX_ROUNDS {
            let op = operations[round];
            
            // Parallel-friendly cache access pattern
            for chunk in (0..MIX_WORDS).step_by(batch_size) {
                for j in chunk..(chunk + batch_size).min(MIX_WORDS) {
                    if j < cache.len() {
                        let cache_idx = (mix[round % MIX_WORDS] as usize + j) % cache.len();
                        let cache_value = unsafe { *cache.get_unchecked(cache_idx) };
                        
                        mix[j] = op.execute(mix[j], cache_value);
                    }
                }
            }
            
            // GPU-friendly mixing between rounds
            if gpu_config.supports_wide_simd {
                Self::simd_mix(mix);
            } else {
                Self::scalar_mix(mix);
            }
        }
    }
    
    /// SIMD-optimized mixing for modern GPUs
    #[target_feature(enable = "avx2")]
    fn simd_mix(mix: &mut [u32; MIX_WORDS]) {
        // Use SIMD instructions for parallel processing
        for i in (0..MIX_WORDS).step_by(8) {
            unsafe {
                let vec = _mm256_loadu_ps(mix.as_ptr().add(i) as *const f32);
                let shuffled = _mm256_shuffle_epi8(_mm256_castps_si256(vec), _mm256_set1_epi8(0x1b));
                let result = _mm256_castsi256_ps(_mm256_xor_si256(_mm256_castps_si256(vec), shuffled));
                _mm256_storeu_ps(mix.as_mut_ptr().add(i) as *mut f32, result);
            }
        }
    }
    
    /// Scalar fallback for older hardware
    #[target_feature(not(enable = "avx2"))]
    fn scalar_mix(mix: &mut [u32; MIX_WORDS]) {
        for i in 0..MIX_WORDS {
            mix[i] = mix[i].wrapping_mul(0x01000193) ^ mix[(i + 1) % MIX_WORDS];
        }
    }
    
    /// GPU-optimized final compression
    fn gpu_final_compression(header_hash: &[u8; 32], nonce: u64, mix: &[u32; MIX_WORDS], gpu_config: &GpuConfig) -> [u8; 32] {
        let mut hasher = Hasher::new();
        
        // Blake3 for GPU optimization (modern GPUs have hardware acceleration)
        hasher.update(header_hash);
        hasher.update(&nonce.to_le_bytes());
        
        // GPU-parallel mixing compression
        if gpu_config.supports_wide_simd {
            for chunk in mix.chunks_exact(8) {
                unsafe {
                    let vec = _mm256_loadu_ps(chunk.as_ptr() as *const f32);
                    let processed = _mm256_mullo_epi32(vec, _mm256_set1_epi32(0x9e3779b9));
                    _mm256_storeu_ps(std::ptr::null_mut(), processed);
                    
                    for element in chunk {
                        hasher.update(&element.to_le_bytes());
                    }
                }
            }
        } else {
            for m in mix {
                hasher.update(&m.to_le_bytes());
            }
        }
        
        let hash = hasher.finalize();
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash.as_bytes()[..32]);
        result
    }

        // Memory-hard mixing using cache
        for round in 0..MIX_ROUNDS {
            let op = self.operations[round];

            // Generate pseudo-random cache indices
            let idx_base = mix[round % MIX_WORDS] as usize;

            for j in 0..MIX_WORDS {
                let idx = (idx_base.wrapping_add(j * 16)) % cache.len();
                let cache_value = cache[idx];
                mix[j] = op.execute(mix[j], cache_value);
            }

            // FNV-like mixing
            for j in 0..MIX_WORDS {
                mix[j] = mix[j].wrapping_mul(0x01000193) ^ mix[(j + 1) % MIX_WORDS];
            }
        }

        // Final hash
        let mut final_hasher = blake3::Hasher::new();
        final_hasher.update(header_hash);
        final_hasher.update(&nonce.to_le_bytes());

        for m in &mix {
            final_hasher.update(&m.to_le_bytes());
        }

        let hash = final_hasher.finalize();
        let mut result = [0u8; 32];
        result.copy_from_slice(hash.as_bytes());
        result
    }

    /// Compute the full hash using DAG (for GPU mining)
    pub fn hash_full(&self, header_hash: &[u8; 32], nonce: u64, dag: &[u32]) -> [u8; 32] {
        // Initial mix from header
        let mut mix = [0u32; MIX_WORDS];

        let mut seed_hasher = Keccak256::new();
        seed_hasher.update(header_hash);
        seed_hasher.update(&nonce.to_le_bytes());
        let seed_hash = seed_hasher.finalize();

        for i in 0..8 {
            mix[i] = LittleEndian::read_u32(&seed_hash[i * 4..(i + 1) * 4]);
            mix[i + 8] = mix[i];
            mix[i + 16] = mix[i].wrapping_mul(0x85ebca6b);
            mix[i + 24] = mix[i].wrapping_mul(0xc2b2ae35);
        }

        // DAG accesses - this is the memory-hard part
        for access in 0..DATASET_ACCESSES {
            // Calculate DAG index from current mix state
            let mix_hash = {
                let mut h = Keccak256::new();
                for m in &mix {
                    h.update(&m.to_le_bytes());
                }
                h.finalize()
            };

            let dag_idx =
                (LittleEndian::read_u64(&mix_hash[0..8]) as usize) % (dag.len() / MIX_WORDS);
            let dag_offset = dag_idx * MIX_WORDS;

            // Apply operation for this access round
            let op = self.operations[access % MIX_ROUNDS];

            for j in 0..MIX_WORDS {
                let dag_value = dag.get(dag_offset + j).copied().unwrap_or(0);
                mix[j] = op.execute(mix[j], dag_value);
            }

            // Additional mixing
            for j in 0..MIX_WORDS {
                mix[j] = mix[j] ^ mix[(j + access) % MIX_WORDS];
            }
        }

        // Final compression
        let mut final_hasher = blake3::Hasher::new();
        final_hasher.update(header_hash);
        final_hasher.update(&nonce.to_le_bytes());
        final_hasher.update(&self.epoch.to_le_bytes());

        for m in &mix {
            final_hasher.update(&m.to_le_bytes());
        }

        let hash = final_hasher.finalize();
        let mut result = [0u8; 32];
        result.copy_from_slice(hash.as_bytes());
        result
    }

    /// Verify a hash meets the difficulty target
    pub fn verify(
        &self,
        header_hash: &[u8; 32],
        nonce: u64,
        target: &[u8; 32],
        cache: &[u32],
    ) -> bool {
        let hash = self.hash_light(header_hash, nonce, cache);
        Self::compare_hash_to_target(&hash, target)
    }

    /// Compare hash to target (hash <= target means valid)
    fn compare_hash_to_target(hash: &[u8; 32], target: &[u8; 32]) -> bool {
        for i in (0..32).rev() {
            if hash[i] < target[i] {
                return true;
            }
            if hash[i] > target[i] {
                return false;
            }
        }
        true // Equal
    }
}

/// Compute initial cache from epoch seed
pub fn compute_cache(epoch: u64, size: usize) -> Vec<u32> {
    let seed = AequiHash::compute_epoch_seed(epoch);
    let num_words = size / 4;
    let mut cache = vec![0u32; num_words];

    // Initialize with sequential hashing
    let mut hasher = Keccak256::new();
    hasher.update(&seed);

    for i in 0..num_words {
        if i % 8 == 0 {
            let hash = hasher.finalize_reset();
            for j in 0..8.min(num_words - i) {
                cache[i + j] = LittleEndian::read_u32(&hash[j * 4..(j + 1) * 4]);
            }
            hasher.update(&hash);
            hasher.update(&i.to_le_bytes());
        }
    }

    // RandMemoHash-style mixing
    for _ in 0..3 {
        for i in 0..num_words {
            let src_idx = cache[i] as usize % num_words;
            let dst_idx = (i + 1) % num_words;
            cache[i] = cache[i] ^ cache[src_idx].wrapping_add(cache[dst_idx]);
        }
    }

    cache
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_seed_deterministic() {
        let seed1 = AequiHash::compute_epoch_seed(0);
        let seed2 = AequiHash::compute_epoch_seed(0);
        assert_eq!(seed1, seed2);

        let seed3 = AequiHash::compute_epoch_seed(1);
        assert_ne!(seed1, seed3);
    }

    #[test]
    fn test_hash_deterministic() {
        let aequihash = AequiHash::new(0);
        let cache = compute_cache(0, 1024 * 1024); // 1MB cache for test
        let header = [0u8; 32];

        let hash1 = aequihash.hash_light(&header, 0, &cache);
        let hash2 = aequihash.hash_light(&header, 0, &cache);
        assert_eq!(hash1, hash2);

        let hash3 = aequihash.hash_light(&header, 1, &cache);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_different_epochs() {
        let cache0 = compute_cache(0, 1024 * 1024);
        let cache1 = compute_cache(1, 1024 * 1024);

        let aeq0 = AequiHash::new(0);
        let aeq1 = AequiHash::new(1);

        let header = [42u8; 32];
        let hash0 = aeq0.hash_light(&header, 0, &cache0);
        let hash1 = aeq1.hash_light(&header, 0, &cache1);

        assert_ne!(hash0, hash1);
    }
}
