//! Peer management
//!
//! Handles peer connections, scoring, and banning.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

/// Maximum number of peers
pub const MAX_PEERS: usize = 50;

/// Maximum number of outbound connections
pub const MAX_OUTBOUND: usize = 8;

/// Peer ban duration
pub const BAN_DURATION: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours

/// Peer state
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PeerState {
    /// Connection pending
    Connecting,
    
    /// Connected and handshake complete
    Connected,
    
    /// Syncing blockchain
    Syncing,
    
    /// Fully synced and active
    Active,
    
    /// Disconnected
    Disconnected,
}

/// Peer information
#[derive(Clone, Debug)]
pub struct PeerInfo {
    /// Peer address
    pub addr: SocketAddr,
    
    /// Connection state
    pub state: PeerState,
    
    /// Is this an inbound or outbound connection
    pub inbound: bool,
    
    /// Peer's best known block height
    pub height: u64,
    
    /// Peer's best known block hash
    pub best_hash: [u8; 32],
    
    /// User agent string
    pub user_agent: String,
    
    /// Services offered
    pub services: u64,
    
    /// Connection time
    pub connected_at: Instant,
    
    /// Last message time
    pub last_message: Instant,
    
    /// Ping latency in milliseconds
    pub latency_ms: Option<u32>,
    
    /// Bytes received
    pub bytes_recv: u64,
    
    /// Bytes sent
    pub bytes_sent: u64,
    
    /// Peer score (for reputation)
    pub score: i32,
}

impl PeerInfo {
    /// Create new peer info
    pub fn new(addr: SocketAddr, inbound: bool) -> Self {
        Self {
            addr,
            state: PeerState::Connecting,
            inbound,
            height: 0,
            best_hash: [0u8; 32],
            user_agent: String::new(),
            services: 0,
            connected_at: Instant::now(),
            last_message: Instant::now(),
            latency_ms: None,
            bytes_recv: 0,
            bytes_sent: 0,
            score: 100,
        }
    }
    
    /// Update peer height
    pub fn update_height(&mut self, height: u64, hash: [u8; 32]) {
        if height > self.height {
            self.height = height;
            self.best_hash = hash;
        }
    }
    
    /// Increase score (good behavior)
    pub fn increase_score(&mut self, amount: i32) {
        self.score = (self.score + amount).min(200);
    }
    
    /// Decrease score (bad behavior)
    pub fn decrease_score(&mut self, amount: i32) {
        self.score = (self.score - amount).max(-100);
    }
    
    /// Check if peer should be banned (score too low)
    pub fn should_ban(&self) -> bool {
        self.score <= -50
    }
    
    /// Get connection duration
    pub fn connection_duration(&self) -> Duration {
        self.connected_at.elapsed()
    }
    
    /// Check if peer is stale (no message in 5 minutes)
    pub fn is_stale(&self) -> bool {
        self.last_message.elapsed() > Duration::from_secs(300)
    }
}

/// Ban information
#[derive(Clone, Debug)]
pub struct BanInfo {
    /// Banned address
    pub addr: SocketAddr,
    
    /// Reason for ban
    pub reason: String,
    
    /// Ban start time
    pub banned_at: Instant,
    
    /// Ban duration
    pub duration: Duration,
}

impl BanInfo {
    /// Check if ban has expired
    pub fn is_expired(&self) -> bool {
        self.banned_at.elapsed() > self.duration
    }
}

/// Peer manager
pub struct PeerManager {
    /// Connected peers
    peers: HashMap<SocketAddr, PeerInfo>,
    
    /// Banned peers
    banned: HashMap<SocketAddr, BanInfo>,
    
    /// Known peer addresses (for discovery)
    known_addrs: Vec<SocketAddr>,
}

impl PeerManager {
    /// Create a new peer manager
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            banned: HashMap::new(),
            known_addrs: Vec::new(),
        }
    }
    
    /// Add a new peer connection
    pub fn add_peer(&mut self, addr: SocketAddr, inbound: bool) -> Result<(), PeerError> {
        // Check if banned
        if let Some(ban) = self.banned.get(&addr) {
            if !ban.is_expired() {
                return Err(PeerError::Banned);
            }
            self.banned.remove(&addr);
        }
        
        // Check limits
        if self.peers.len() >= MAX_PEERS {
            return Err(PeerError::TooManyPeers);
        }
        
        if !inbound && self.outbound_count() >= MAX_OUTBOUND {
            return Err(PeerError::TooManyOutbound);
        }
        
        // Add peer
        let info = PeerInfo::new(addr, inbound);
        self.peers.insert(addr, info);
        
        Ok(())
    }
    
    /// Remove a peer
    pub fn remove_peer(&mut self, addr: &SocketAddr) {
        self.peers.remove(addr);
    }
    
    /// Get peer info
    pub fn get_peer(&self, addr: &SocketAddr) -> Option<&PeerInfo> {
        self.peers.get(addr)
    }
    
    /// Get mutable peer info
    pub fn get_peer_mut(&mut self, addr: &SocketAddr) -> Option<&mut PeerInfo> {
        self.peers.get_mut(addr)
    }
    
    /// Ban a peer
    pub fn ban_peer(&mut self, addr: SocketAddr, reason: String) {
        self.peers.remove(&addr);
        
        let ban = BanInfo {
            addr,
            reason,
            banned_at: Instant::now(),
            duration: BAN_DURATION,
        };
        
        self.banned.insert(addr, ban);
    }
    
    /// Get number of connected peers
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }
    
    /// Get number of outbound connections
    pub fn outbound_count(&self) -> usize {
        self.peers.values().filter(|p| !p.inbound).count()
    }
    
    /// Get number of inbound connections
    pub fn inbound_count(&self) -> usize {
        self.peers.values().filter(|p| p.inbound).count()
    }
    
    /// Get all connected peers
    pub fn connected_peers(&self) -> Vec<&PeerInfo> {
        self.peers.values().collect()
    }
    
    /// Get best peer (highest height)
    pub fn best_peer(&self) -> Option<&PeerInfo> {
        self.peers.values()
            .filter(|p| p.state == PeerState::Active)
            .max_by_key(|p| p.height)
    }
    
    /// Add known address
    pub fn add_known_addr(&mut self, addr: SocketAddr) {
        if !self.known_addrs.contains(&addr) && !self.banned.contains_key(&addr) {
            self.known_addrs.push(addr);
        }
    }
    
    /// Get addresses for discovery
    pub fn get_addrs_for_sharing(&self) -> Vec<SocketAddr> {
        self.peers.keys().cloned().collect()
    }
    
    /// Get addresses to connect to
    pub fn get_addrs_to_connect(&self) -> Vec<SocketAddr> {
        self.known_addrs.iter()
            .filter(|a| !self.peers.contains_key(a))
            .filter(|a| !self.banned.contains_key(a))
            .take(10)
            .cloned()
            .collect()
    }
    
    /// Clean up expired bans and stale peers
    pub fn cleanup(&mut self) {
        // Remove expired bans
        self.banned.retain(|_, ban| !ban.is_expired());
        
        // Mark stale peers for disconnection
        for peer in self.peers.values_mut() {
            if peer.is_stale() && peer.state == PeerState::Active {
                peer.decrease_score(10);
            }
        }
    }
}

impl Default for PeerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Peer-related errors
#[derive(Debug, thiserror::Error)]
pub enum PeerError {
    #[error("Peer is banned")]
    Banned,
    
    #[error("Too many peers connected")]
    TooManyPeers,
    
    #[error("Too many outbound connections")]
    TooManyOutbound,
    
    #[error("Peer not found")]
    NotFound,
    
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    
    fn test_addr(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
    }
    
    #[test]
    fn test_add_peer() {
        let mut manager = PeerManager::new();
        let addr = test_addr(8888);
        
        assert!(manager.add_peer(addr, false).is_ok());
        assert_eq!(manager.peer_count(), 1);
    }
    
    #[test]
    fn test_ban_peer() {
        let mut manager = PeerManager::new();
        let addr = test_addr(8888);
        
        manager.add_peer(addr, false).unwrap();
        manager.ban_peer(addr, "Test ban".to_string());
        
        assert_eq!(manager.peer_count(), 0);
        assert!(manager.add_peer(addr, false).is_err());
    }
    
    #[test]
    fn test_peer_scoring() {
        let mut peer = PeerInfo::new(test_addr(8888), false);
        assert_eq!(peer.score, 100);
        
        peer.increase_score(50);
        assert_eq!(peer.score, 150);
        
        peer.decrease_score(200);
        assert_eq!(peer.score, -50);
        assert!(peer.should_ban());
    }
}
