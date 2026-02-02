//! RPC API server

use axum::{
    routing::{get, post},
    Router, Json,
    extract::State,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use aequitas_core::{Blockchain, Block, Transaction, Address};
use crate::mempool::Mempool;

/// RPC server state
pub struct RpcState {
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub mempool: Arc<RwLock<Mempool>>,
}

/// Create RPC router
pub fn create_router(state: Arc<RpcState>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/info", get(get_info))
        .route("/block/:hash", get(get_block))
        .route("/block/height/:height", get(get_block_by_height))
        .route("/tx/:hash", get(get_transaction))
        .route("/balance/:address", get(get_balance))
        .route("/mempool", get(get_mempool))
        .route("/tx/send", post(send_transaction))
        .route("/getblocktemplate", post(get_block_template))
        .route("/submitblock", post(submit_block))
        .with_state(state)
}

/// Index endpoint
async fn index() -> &'static str {
    "Aequitas Node RPC v0.1.0"
}

/// Node info response
#[derive(Serialize)]
struct InfoResponse {
    version: String,
    network: String,
    height: u64,
    difficulty: u64,
    mempool_size: usize,
    peers: usize,
}

/// Get node info
async fn get_info(State(state): State<Arc<RpcState>>) -> Json<InfoResponse> {
    let chain = state.blockchain.read().await;
    let mempool = state.mempool.read().await;
    
    Json(InfoResponse {
        version: "0.1.0".to_string(),
        network: "testnet".to_string(),
        height: chain.height(),
        difficulty: chain.difficulty(),
        mempool_size: mempool.size(),
        peers: 0, // TODO: Get from network
    })
}

/// Block response
#[derive(Serialize)]
struct BlockResponse {
    hash: String,
    height: u64,
    prev_hash: String,
    timestamp: i64,
    difficulty: u64,
    nonce: u64,
    tx_count: usize,
}

impl From<&Block> for BlockResponse {
    fn from(block: &Block) -> Self {
        Self {
            hash: hex::encode(block.hash()),
            height: block.header.height,
            prev_hash: hex::encode(block.header.prev_hash),
            timestamp: block.header.timestamp.timestamp(),
            difficulty: block.header.difficulty,
            nonce: block.header.nonce,
            tx_count: block.transactions.len(),
        }
    }
}

/// Get block by hash
async fn get_block(
    State(state): State<Arc<RpcState>>,
    axum::extract::Path(hash): axum::extract::Path<String>,
) -> Result<Json<BlockResponse>, StatusCode> {
    let hash_bytes = hex::decode(&hash).map_err(|_| StatusCode::BAD_REQUEST)?;
    if hash_bytes.len() != 32 {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let mut hash_arr = [0u8; 32];
    hash_arr.copy_from_slice(&hash_bytes);
    
    let chain = state.blockchain.read().await;
    
    chain.get_block(&hash_arr)
        .map(|b| Json(BlockResponse::from(b)))
        .ok_or(StatusCode::NOT_FOUND)
}

/// Get block by height
async fn get_block_by_height(
    State(state): State<Arc<RpcState>>,
    axum::extract::Path(height): axum::extract::Path<u64>,
) -> Result<Json<BlockResponse>, StatusCode> {
    let chain = state.blockchain.read().await;
    
    chain.get_block_at_height(height)
        .map(|b| Json(BlockResponse::from(b)))
        .ok_or(StatusCode::NOT_FOUND)
}

/// Transaction response
#[derive(Serialize)]
struct TxResponse {
    hash: String,
    inputs: usize,
    outputs: usize,
    timestamp: i64,
}

/// Get transaction
async fn get_transaction(
    State(_state): State<Arc<RpcState>>,
    axum::extract::Path(hash): axum::extract::Path<String>,
) -> Result<Json<TxResponse>, StatusCode> {
    // TODO: Implement transaction lookup
    Err(StatusCode::NOT_FOUND)
}

/// Balance response
#[derive(Serialize)]
struct BalanceResponse {
    address: String,
    balance: u64,
    balance_formatted: String,
}

/// Get balance
async fn get_balance(
    State(state): State<Arc<RpcState>>,
    axum::extract::Path(address): axum::extract::Path<String>,
) -> Result<Json<BalanceResponse>, StatusCode> {
    let addr = Address::from_string(&address).map_err(|_| StatusCode::BAD_REQUEST)?;
    let chain = state.blockchain.read().await;
    let balance = chain.get_balance(&addr);
    
    Ok(Json(BalanceResponse {
        address,
        balance,
        balance_formatted: format!("{:.9} AEQ", balance as f64 / 1_000_000_000.0),
    }))
}

/// Mempool response
#[derive(Serialize)]
struct MempoolResponse {
    size: usize,
    total_fees: u64,
    hashes: Vec<String>,
}

/// Get mempool info
async fn get_mempool(State(state): State<Arc<RpcState>>) -> Json<MempoolResponse> {
    let mempool = state.mempool.read().await;
    
    Json(MempoolResponse {
        size: mempool.size(),
        total_fees: mempool.total_fees(),
        hashes: mempool.hashes().iter().map(hex::encode).collect(),
    })
}

/// Send transaction request
#[derive(Deserialize)]
struct SendTxRequest {
    tx_hex: String,
}

/// Send transaction response
#[derive(Serialize)]
struct SendTxResponse {
    success: bool,
    hash: Option<String>,
    error: Option<String>,
}

/// Send transaction
async fn send_transaction(
    State(state): State<Arc<RpcState>>,
    Json(request): Json<SendTxRequest>,
) -> Json<SendTxResponse> {
    let tx_bytes = match hex::decode(&request.tx_hex) {
        Ok(b) => b,
        Err(e) => return Json(SendTxResponse {
            success: false,
            hash: None,
            error: Some(format!("Invalid hex: {}", e)),
        }),
    };
    
    let tx: Transaction = match bincode::deserialize(&tx_bytes) {
        Ok(t) => t,
        Err(e) => return Json(SendTxResponse {
            success: false,
            hash: None,
            error: Some(format!("Invalid transaction: {}", e)),
        }),
    };
    
    let hash = tx.hash();
    let mut mempool = state.mempool.write().await;
    
    match mempool.add(tx, 0) {
        Ok(_) => Json(SendTxResponse {
            success: true,
            hash: Some(hex::encode(hash)),
            error: None,
        }),
        Err(e) => Json(SendTxResponse {
            success: false,
            hash: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Block template request
#[derive(Deserialize)]
struct BlockTemplateRequest {
    address: Option<String>,
}

/// Block template response
#[derive(Serialize)]
struct BlockTemplateResponse {
    height: u64,
    difficulty: u64,
    prev_hash: String,
    header_hash: String,
    timestamp: i64,
    reward: u64,
}

/// Get block template for mining
async fn get_block_template(
    State(state): State<Arc<RpcState>>,
    Json(_request): Json<BlockTemplateRequest>,
) -> Json<BlockTemplateResponse> {
    let chain = state.blockchain.read().await;
    let tip = chain.tip_block();
    
    let height = chain.height() + 1;
    let difficulty = chain.next_difficulty();
    let (reward, _treasury) = Blockchain::rewards_for_height(height);
    
    // Create a template header hash
    let mut header_data = Vec::new();
    header_data.extend_from_slice(&tip.hash());
    header_data.extend_from_slice(&height.to_le_bytes());
    header_data.extend_from_slice(&difficulty.to_le_bytes());
    
    let header_hash = blake3::hash(&header_data);
    
    Json(BlockTemplateResponse {
        height,
        difficulty,
        prev_hash: hex::encode(tip.hash()),
        header_hash: hex::encode(header_hash.as_bytes()),
        timestamp: chrono::Utc::now().timestamp(),
        reward,
    })
}

/// Submit block request
#[derive(Deserialize)]
struct SubmitBlockRequest {
    job_id: String,
    nonce: u64,
    hash: String,
}

/// Submit block response
#[derive(Serialize)]
struct SubmitBlockResponse {
    success: bool,
    message: String,
}

/// Submit mined block
async fn submit_block(
    State(_state): State<Arc<RpcState>>,
    Json(request): Json<SubmitBlockRequest>,
) -> Json<SubmitBlockResponse> {
    // TODO: Validate and add block
    log::info!("Block submission received: job={}, nonce={}", request.job_id, request.nonce);
    
    Json(SubmitBlockResponse {
        success: true,
        message: "Block received".to_string(),
    })
}
