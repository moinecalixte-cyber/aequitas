//! Mining worker implementation

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use crossbeam_channel::{bounded, Receiver, Sender};
use aequitas_consensus::{AequiHash, DAG, ProofOfWork};
use aequitas_consensus::aequihash::EPOCH_LENGTH;
use aequitas_consensus::pow::{difficulty_to_target, hash_less_or_equal, WorkUnit};
use crate::config::MinerConfig;
use crate::stats::MiningStats;

/// Mining job
#[derive(Clone, Debug)]
pub struct MiningJob {
    /// Job ID
    pub job_id: String,
    
    /// Block header hash
    pub header_hash: [u8; 32],
    
    /// Difficulty target
    pub difficulty: u64,
    
    /// Block height
    pub height: u64,
    
    /// Epoch for this job
    pub epoch: u64,
}

impl MiningJob {
    /// Create from block info
    pub fn new(header_hash: [u8; 32], difficulty: u64, height: u64) -> Self {
        Self {
            job_id: hex::encode(&header_hash[..8]),
            header_hash,
            difficulty,
            height,
            epoch: height / EPOCH_LENGTH,
        }
    }
}

/// Mining result
#[derive(Clone, Debug)]
pub struct MiningResult {
    /// Job ID
    pub job_id: String,
    
    /// Winning nonce
    pub nonce: u64,
    
    /// Result hash
    pub hash: [u8; 32],
}

/// Worker thread control
struct WorkerControl {
    /// Stop flag
    stop: AtomicBool,
    
    /// Current job
    job_id: std::sync::RwLock<String>,
    
    /// Hash counter
    hash_count: AtomicU64,
}

/// CPU mining worker
pub struct CpuWorker {
    /// Worker ID
    id: usize,
    
    /// Control
    control: Arc<WorkerControl>,
    
    /// Thread handle
    handle: Option<thread::JoinHandle<()>>,
}

impl CpuWorker {
    /// Create a new CPU worker
    pub fn new(id: usize, control: Arc<WorkerControl>) -> Self {
        Self {
            id,
            control,
            handle: None,
        }
    }
    
    /// Start mining
    pub fn start(
        &mut self,
        job_rx: Receiver<MiningJob>,
        result_tx: Sender<MiningResult>,
        nonce_start: u64,
        nonce_range: u64,
    ) {
        let control = self.control.clone();
        let id = self.id;
        
        let handle = thread::spawn(move || {
            log::info!("CPU Worker {} started", id);
            
            let mut current_job: Option<MiningJob> = None;
            let mut dag: Option<(u64, Arc<Vec<u32>>)> = None;
            let mut aequihash: Option<AequiHash> = None;
            
            loop {
                // Check for new job
                match job_rx.try_recv() {
                    Ok(job) => {
                        log::debug!("Worker {} got new job: {}", id, job.job_id);
                        
                        // Update DAG if epoch changed
                        let need_new_dag = dag.as_ref()
                            .map(|(epoch, _)| *epoch != job.epoch)
                            .unwrap_or(true);
                        
                        if need_new_dag {
                            log::info!("Worker {} generating cache for epoch {}", id, job.epoch);
                            let cache = aequitas_consensus::aequihash::compute_cache(
                                job.epoch, 
                                16 * 1024 * 1024 // 16MB cache for CPU
                            );
                            dag = Some((job.epoch, Arc::new(cache)));
                            aequihash = Some(AequiHash::new(job.epoch));
                        }
                        
                        current_job = Some(job);
                    }
                    Err(_) => {}
                }
                
                // Check stop flag
                if control.stop.load(Ordering::Relaxed) {
                    break;
                }
                
                // Mine if we have a job
                if let (Some(job), Some((_, cache)), Some(aeq)) = 
                    (&current_job, &dag, &aequihash) 
                {
                    let target = difficulty_to_target(job.difficulty);
                    
                    // Mine a batch of nonces
                    let batch_size = 10000u64;
                    let nonce_offset = nonce_start + (id as u64 * nonce_range);
                    
                    for batch in 0..100 {
                        let start_nonce = nonce_offset + batch * batch_size;
                        
                        for nonce in start_nonce..start_nonce + batch_size {
                            // Check for new job periodically
                            if nonce % 10000 == 0 {
                                if job_rx.try_recv().is_ok() || control.stop.load(Ordering::Relaxed) {
                                    break;
                                }
                            }
                            
                            let hash = aeq.hash_light(&job.header_hash, nonce, cache);
                            control.hash_count.fetch_add(1, Ordering::Relaxed);
                            
                            if hash_less_or_equal(&hash, &target) {
                                log::info!("Worker {} found solution! Nonce: {}", id, nonce);
                                
                                let result = MiningResult {
                                    job_id: job.job_id.clone(),
                                    nonce,
                                    hash,
                                };
                                
                                let _ = result_tx.send(result);
                            }
                        }
                        
                        if control.stop.load(Ordering::Relaxed) {
                            break;
                        }
                    }
                } else {
                    // No job, wait a bit
                    thread::sleep(Duration::from_millis(100));
                }
            }
            
            log::info!("CPU Worker {} stopped", id);
        });
        
        self.handle = Some(handle);
    }
    
    /// Stop the worker
    pub fn stop(&mut self) {
        self.control.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Mining worker manager
pub struct MiningWorker {
    /// Configuration
    config: MinerConfig,
    
    /// Worker control
    control: Arc<WorkerControl>,
    
    /// CPU workers
    cpu_workers: Vec<CpuWorker>,
    
    /// Job sender
    job_tx: Option<Sender<MiningJob>>,
    
    /// Result receiver
    result_rx: Option<Receiver<MiningResult>>,
    
    /// Mining statistics
    stats: Arc<MiningStats>,
    
    /// Is running
    running: bool,
}

impl MiningWorker {
    /// Create a new mining worker
    pub fn new(config: MinerConfig) -> Self {
        let control = Arc::new(WorkerControl {
            stop: AtomicBool::new(false),
            job_id: std::sync::RwLock::new(String::new()),
            hash_count: AtomicU64::new(0),
        });
        
        Self {
            config,
            control,
            cpu_workers: Vec::new(),
            job_tx: None,
            result_rx: None,
            stats: Arc::new(MiningStats::new()),
            running: false,
        }
    }
    
    /// Start mining
    pub fn start(&mut self) -> anyhow::Result<Receiver<MiningResult>> {
        if self.running {
            anyhow::bail!("Already running");
        }
        
        let (job_tx, job_rx) = bounded::<MiningJob>(10);
        let (result_tx, result_rx) = bounded::<MiningResult>(10);
        
        // Start CPU workers
        let num_threads = self.config.cpu_threads;
        let nonce_range = u64::MAX / num_threads as u64;
        
        for i in 0..num_threads {
            let mut worker = CpuWorker::new(i, self.control.clone());
            worker.start(
                job_rx.clone(),
                result_tx.clone(),
                i as u64 * nonce_range,
                nonce_range,
            );
            self.cpu_workers.push(worker);
        }
        
        log::info!("Started {} CPU workers", num_threads);
        
        // TODO: Start GPU workers if enabled
        if self.config.gpu_enabled {
            log::warn!("GPU mining not yet implemented - using CPU only");
        }
        
        self.job_tx = Some(job_tx);
        self.result_rx = Some(result_rx.clone());
        self.running = true;
        
        // Start stats thread
        let control = self.control.clone();
        let stats = self.stats.clone();
        let interval = self.config.stats_interval;
        
        thread::spawn(move || {
            let mut last_count = 0u64;
            let mut last_time = Instant::now();
            
            while !control.stop.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_secs(interval));
                
                let count = control.hash_count.load(Ordering::Relaxed);
                let elapsed = last_time.elapsed().as_secs_f64();
                
                if elapsed > 0.0 {
                    let hashrate = (count - last_count) as f64 / elapsed;
                    stats.update_hashrate(hashrate);
                    stats.add_hashes(count - last_count);
                    
                    log::info!(
                        "Hashrate: {:.2} H/s | Total: {} hashes",
                        hashrate,
                        count
                    );
                }
                
                last_count = count;
                last_time = Instant::now();
            }
        });
        
        Ok(result_rx)
    }
    
    /// Submit a new job
    pub fn submit_job(&self, job: MiningJob) -> anyhow::Result<()> {
        if let Some(tx) = &self.job_tx {
            // Clear old jobs and send new one
            while tx.try_recv().is_ok() {}
            
            // Send to all workers
            for _ in 0..self.cpu_workers.len() {
                tx.send(job.clone())?;
            }
            
            log::debug!("Submitted job {} to workers", job.job_id);
        }
        Ok(())
    }
    
    /// Stop mining
    pub fn stop(&mut self) {
        if !self.running {
            return;
        }
        
        log::info!("Stopping miners...");
        self.control.stop.store(true, Ordering::Relaxed);
        
        for worker in &mut self.cpu_workers {
            worker.stop();
        }
        
        self.cpu_workers.clear();
        self.running = false;
        log::info!("All miners stopped");
    }
    
    /// Get current hashrate
    pub fn hashrate(&self) -> f64 {
        self.stats.hashrate()
    }
    
    /// Get total hashes
    pub fn total_hashes(&self) -> u64 {
        self.control.hash_count.load(Ordering::Relaxed)
    }
    
    /// Get stats
    pub fn stats(&self) -> &MiningStats {
        &self.stats
    }
    
    /// Is running
    pub fn is_running(&self) -> bool {
        self.running
    }
}

impl Drop for MiningWorker {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mining_job_creation() {
        let job = MiningJob::new([0u8; 32], 1000, 100);
        assert_eq!(job.height, 100);
        assert_eq!(job.epoch, 100 / EPOCH_LENGTH);
    }
}
