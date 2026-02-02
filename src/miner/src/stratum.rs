//! Stratum protocol support for pool mining

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use crate::worker::MiningJob;

/// Stratum protocol version
pub const STRATUM_VERSION: &str = "2.0.0";

/// Stratum method types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum StratumMethod {
    #[serde(rename = "mining.subscribe")]
    Subscribe(Vec<String>),
    
    #[serde(rename = "mining.authorize")]
    Authorize(String, String), // worker, password
    
    #[serde(rename = "mining.submit")]
    Submit(String, String, String, String, String), // worker, job_id, nonce, header, mixhash
    
    #[serde(rename = "mining.notify")]
    Notify(StratumJob),
    
    #[serde(rename = "mining.set_difficulty")]
    SetDifficulty(f64),
    
    #[serde(rename = "mining.set_extranonce")]
    SetExtranonce(String, u32),
}

/// Stratum job notification
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StratumJob {
    /// Job ID
    pub job_id: String,
    
    /// Block header hash
    pub header_hash: String,
    
    /// Seed hash (for DAG)
    pub seed_hash: String,
    
    /// Difficulty target
    pub target: String,
    
    /// Clean jobs flag
    pub clean_jobs: bool,
    
    /// Block height
    pub height: Option<u64>,
}

/// Stratum request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StratumRequest {
    pub id: u64,
    pub method: String,
    pub params: Vec<serde_json::Value>,
}

/// Stratum response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StratumResponse {
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<StratumError>,
}

/// Stratum error
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StratumError {
    pub code: i32,
    pub message: String,
}

/// Stratum client for pool mining
pub struct StratumClient {
    /// Pool URL
    url: String,
    
    /// Worker name
    worker: String,
    
    /// Password
    password: String,
    
    /// Connection
    stream: Option<TcpStream>,
    
    /// Request ID counter
    request_id: u64,
    
    /// Subscribed flag
    subscribed: bool,
    
    /// Authorized flag
    authorized: bool,
    
    /// Current difficulty
    difficulty: f64,
    
    /// Current extranonce
    extranonce: String,
    
    /// Extranonce size
    extranonce_size: u32,
    
    /// Running flag
    running: Arc<AtomicBool>,
}

impl StratumClient {
    /// Create new stratum client
    pub fn new(url: &str, worker: &str, password: &str) -> Self {
        Self {
            url: url.to_string(),
            worker: worker.to_string(),
            password: password.to_string(),
            stream: None,
            request_id: 0,
            subscribed: false,
            authorized: false,
            difficulty: 1.0,
            extranonce: String::new(),
            extranonce_size: 0,
            running: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// Connect to pool
    pub fn connect(&mut self) -> anyhow::Result<()> {
        log::info!("Connecting to pool: {}", self.url);
        
        // Parse URL
        let url = self.url.replace("stratum+tcp://", "");
        let stream = TcpStream::connect(&url)?;
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(Duration::from_secs(10)))?;
        
        self.stream = Some(stream);
        self.running.store(true, Ordering::Relaxed);
        
        // Subscribe
        self.subscribe()?;
        
        // Authorize
        self.authorize()?;
        
        log::info!("Connected and authorized to pool");
        
        Ok(())
    }
    
    /// Subscribe to mining notifications
    fn subscribe(&mut self) -> anyhow::Result<()> {
        let request = StratumRequest {
            id: self.next_id(),
            method: "mining.subscribe".to_string(),
            params: vec![
                serde_json::Value::String(format!("AequitasMiner/{}", STRATUM_VERSION)),
            ],
        };
        
        self.send_request(&request)?;
        let response = self.receive_response()?;
        
        if let Some(error) = response.error {
            anyhow::bail!("Subscribe failed: {}", error.message);
        }
        
        // Parse subscription result
        if let Some(result) = response.result {
            if let Some(arr) = result.as_array() {
                // Extract extranonce
                if arr.len() >= 2 {
                    if let Some(en) = arr[1].as_str() {
                        self.extranonce = en.to_string();
                    }
                    if let Some(size) = arr[2].as_u64() {
                        self.extranonce_size = size as u32;
                    }
                }
            }
        }
        
        self.subscribed = true;
        Ok(())
    }
    
    /// Authorize worker
    fn authorize(&mut self) -> anyhow::Result<()> {
        let request = StratumRequest {
            id: self.next_id(),
            method: "mining.authorize".to_string(),
            params: vec![
                serde_json::Value::String(self.worker.clone()),
                serde_json::Value::String(self.password.clone()),
            ],
        };
        
        self.send_request(&request)?;
        let response = self.receive_response()?;
        
        if let Some(error) = response.error {
            anyhow::bail!("Authorization failed: {}", error.message);
        }
        
        if response.result != Some(serde_json::Value::Bool(true)) {
            anyhow::bail!("Authorization rejected");
        }
        
        self.authorized = true;
        Ok(())
    }
    
    /// Submit a share
    pub fn submit_share(
        &mut self,
        job_id: &str,
        nonce: &str,
        header: &str,
        mixhash: &str,
    ) -> anyhow::Result<bool> {
        let request = StratumRequest {
            id: self.next_id(),
            method: "mining.submit".to_string(),
            params: vec![
                serde_json::Value::String(self.worker.clone()),
                serde_json::Value::String(job_id.to_string()),
                serde_json::Value::String(nonce.to_string()),
                serde_json::Value::String(header.to_string()),
                serde_json::Value::String(mixhash.to_string()),
            ],
        };
        
        self.send_request(&request)?;
        let response = self.receive_response()?;
        
        if let Some(error) = response.error {
            log::warn!("Share rejected: {}", error.message);
            return Ok(false);
        }
        
        Ok(response.result == Some(serde_json::Value::Bool(true)))
    }
    
    /// Receive a job notification
    pub fn receive_job(&mut self) -> anyhow::Result<Option<MiningJob>> {
        if let Some(ref mut stream) = self.stream {
            let mut reader = BufReader::new(stream.try_clone()?);
            let mut line = String::new();
            
            match reader.read_line(&mut line) {
                Ok(0) => {
                    anyhow::bail!("Connection closed");
                }
                Ok(_) => {
                    let notification: serde_json::Value = serde_json::from_str(&line)?;
                    
                    if let Some(method) = notification.get("method").and_then(|m| m.as_str()) {
                        match method {
                            "mining.notify" => {
                                if let Some(params) = notification.get("params").and_then(|p| p.as_array()) {
                                    return Ok(Some(self.parse_job(params)?));
                                }
                            }
                            "mining.set_difficulty" => {
                                if let Some(params) = notification.get("params").and_then(|p| p.as_array()) {
                                    if let Some(diff) = params.first().and_then(|d| d.as_f64()) {
                                        self.difficulty = diff;
                                        log::info!("Difficulty set to: {}", diff);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return Ok(None);
                }
                Err(e) => {
                    anyhow::bail!("Read error: {}", e);
                }
            }
        }
        
        Ok(None)
    }
    
    /// Parse job from notification params
    fn parse_job(&self, params: &[serde_json::Value]) -> anyhow::Result<MiningJob> {
        let job_id = params.get(0)
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing job_id"))?;
        
        let header_hash = params.get(1)
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing header_hash"))?;
        
        let mut hash_bytes = [0u8; 32];
        hex::decode_to_slice(header_hash, &mut hash_bytes)?;
        
        let target = params.get(3)
            .and_then(|v| v.as_str())
            .unwrap_or("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        
        // Convert target to difficulty
        let difficulty = self.target_to_difficulty(target);
        
        let height = params.get(5)
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        Ok(MiningJob {
            job_id: job_id.to_string(),
            header_hash: hash_bytes,
            difficulty,
            height,
            epoch: height / aequitas_consensus::aequihash::EPOCH_LENGTH,
        })
    }
    
    /// Convert target hex to difficulty
    fn target_to_difficulty(&self, target: &str) -> u64 {
        // Simplified conversion
        let target_bytes = hex::decode(target).unwrap_or_else(|_| vec![0xff; 32]);
        if target_bytes.len() >= 4 {
            let leading = u32::from_be_bytes([target_bytes[0], target_bytes[1], target_bytes[2], target_bytes[3]]);
            if leading == 0 { 1000000 } else { (u32::MAX / leading) as u64 }
        } else {
            1000
        }
    }
    
    /// Send a request
    fn send_request(&mut self, request: &StratumRequest) -> anyhow::Result<()> {
        if let Some(ref mut stream) = self.stream {
            let json = serde_json::to_string(request)? + "\n";
            stream.write_all(json.as_bytes())?;
            stream.flush()?;
        }
        Ok(())
    }
    
    /// Receive a response
    fn receive_response(&mut self) -> anyhow::Result<StratumResponse> {
        if let Some(ref mut stream) = self.stream {
            let mut reader = BufReader::new(stream.try_clone()?);
            let mut line = String::new();
            reader.read_line(&mut line)?;
            
            let response: StratumResponse = serde_json::from_str(&line)?;
            return Ok(response);
        }
        
        anyhow::bail!("Not connected")
    }
    
    /// Get next request ID
    fn next_id(&mut self) -> u64 {
        self.request_id += 1;
        self.request_id
    }
    
    /// Is connected
    pub fn is_connected(&self) -> bool {
        self.stream.is_some() && self.authorized
    }
    
    /// Get current difficulty
    pub fn difficulty(&self) -> f64 {
        self.difficulty
    }
    
    /// Disconnect
    pub fn disconnect(&mut self) {
        self.stream = None;
        self.subscribed = false;
        self.authorized = false;
        self.running.store(false, Ordering::Relaxed);
    }
}

impl Drop for StratumClient {
    fn drop(&mut self) {
        self.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stratum_client_creation() {
        let client = StratumClient::new(
            "stratum+tcp://pool.example.com:3333",
            "aeq1TestWorker",
            "x",
        );
        
        assert!(!client.is_connected());
        assert_eq!(client.difficulty(), 1.0);
    }
}
