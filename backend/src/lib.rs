pub mod session_manager;
pub mod auto_deposit;
pub mod delegation;
pub mod vault_monitor;
pub mod signer;

use axum::{
    Router,
    routing::{post, get, delete},
    extract::{State, Path},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use anyhow::Result;
use sqlx::{Pool, Postgres};
use ring::aead::{LessSafeKey, UnboundKey, AES_256_GCM};
use solana_client::nonblocking::rpc_client::RpcClient;
use uuid::Uuid;

// Import modules
use session_manager::SessionManager;
use auto_deposit::AutoDepositCalculator;
use delegation::DelegationManager;
use vault_monitor::VaultMonitor;
use signer::TransactionSigner;


// === App State (shared across routes) ===
#[derive(Clone)]
pub struct AppState {
    pub session_manager: SessionManager,
    pub auto_deposit: AutoDepositCalculator,
    pub delegation_manager: DelegationManager,
    pub vault_monitor: VaultMonitor,
    pub tx_signer: TransactionSigner,
}


// === Request DTOs ===
#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub user_id: String,
    pub lifetime_minutes: i64,
}

#[derive(Deserialize)]
pub struct ApproveDelegateRequest {
    pub session_id: Uuid,
    pub solana_delegate_address: String,
}

#[derive(Deserialize)]
pub struct AutoDepositRequest {
    pub session_id: Uuid,
    pub estimated_fee_sol: u64,
}


// === Routes ===

// POST /session/create
pub async fn create_session(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateSessionRequest>
) -> Json<Uuid> {
    let id = state
        .session_manager
        .create_session(body.user_id, body.lifetime_minutes)
        .await
        .expect("session failed to create");

    Json(id)
}

// POST /session/approve
pub async fn approve_delegate(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ApproveDelegateRequest>
) -> Json<String> {
    let session_keypair = state
        .session_manager
        .get_session_keypair(body.session_id)
        .await
        .expect("session not found");

    state
        .delegation_manager
        .revoke(session_keypair.pubkey())
        .await
        .unwrap();

    Json("delegate approved".into())
}

// POST /session/deposit
pub async fn auto_deposit(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AutoDepositRequest>
) -> Json<String> {
    state
        .auto_deposit
        .ensure_sol_for_fees(body.session_id, body.estimated_fee_sol)
        .await
        .unwrap();

    Json("deposit complete".into())
}

// GET /session/status/{session_id}
pub async fn session_status(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
) -> Json<String> {
    let balance = state
        .vault_monitor
        .monitor_balance(&session_id)
        .await
        .unwrap_or(0.0);

    Json(format!("Balance: {balance} SOL"))
}

// DELETE /session/revoke/{session_id}
pub async fn revoke(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
) -> Json<String> {
    state
        .session_manager
        .cleanup_session(session_id)
        .await
        .unwrap();

    Json("Session closed".into())
}


// === Router exposed to main.rs ===

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/session/create", post(create_session))
        .route("/session/approve", post(approve_delegate))
        .route("/session/deposit", post(auto_deposit))
        .route("/session/status/:session_id", get(session_status))
        .route("/session/revoke/:session_id", delete(revoke))
        .with_state(state)
}


// === Initialization Helper ===
pub async fn init_state(
    db: Pool<Postgres>,
    rpc_url: &str,
    sealing_key_bytes: &[u8; 32],
) -> Result<AppState> {
    let sealing_key =
        LessSafeKey::new(UnboundKey::new(&AES_256_GCM, sealing_key_bytes).unwrap());

    let rpc = RpcClient::new(rpc_url.to_string());

    Ok(AppState {
        session_manager: SessionManager { db: db.clone(), sealing_key: Arc::new(sealing_key) },
        auto_deposit: AutoDepositCalculator { db: db.clone(), rpc: rpc.clone() },
        delegation_manager: DelegationManager { rpc: rpc.clone() },
        vault_monitor: VaultMonitor { db: db.clone(), rpc: rpc.clone() },
        tx_signer: TransactionSigner { rpc },
    })
}
