pub mod auto_deposit;
pub mod delegation;
pub mod session_manager;
pub mod signer;
pub mod vault_monitor;

use actix_web::{web, App, HttpResponse, HttpServer};
use chrono::{DateTime, Utc};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use sqlx::{FromRow, PgPool};
use std::sync::Arc;
use uuid::Uuid;
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
pub const MIN_DEPOSIT_LAMPORTS: u64 = 1_000_000;        // 0.001 SOL
pub const MAX_DEPOSIT_LAMPORTS: u64 = 100_000_000_000;  // 100 SOL
pub const MIN_APPROVED_LAMPORTS: u64 = 1_000_000;       // 0.001 SOL
pub const MAX_APPROVED_LAMPORTS: u64 = 1_000_000_000_000; // 1000 SOL
pub const SESSION_DURATION_HOURS: i64 = 1;
pub const SESSION_RENEWAL_WINDOW_MINUTES: i64 = 5;

// ============================================================================
// SHARED APP STATE  (injected into every handler via web::Data)
// ============================================================================

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub rpc_client: Arc<RpcClient>,
    pub program_id: Pubkey,
}

// ============================================================================
// DATABASE ROW TYPES
// ============================================================================

#[derive(FromRow, Clone)]
pub struct SessionRecord {
    pub session_id: Uuid,
    pub user_wallet: String,
    pub ephemeral_wallet: String,
    pub vault_pda: String,
    pub approved_amount: i64,
    pub total_deposited: i64,
    pub expires_at: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

// ============================================================================
// SHARED REQUEST / RESPONSE TYPES
// ============================================================================

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: Option<String>,
}

// ---------- session_manager shared ----------

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub user_wallet: String,
    #[serde(default = "default_approved_sol")]
    pub approved_amount_sol: f64,
    #[serde(default = "default_session_duration")]
    pub session_duration_hours: i64,
}

fn default_approved_sol() -> f64 {
    10.0
}
fn default_session_duration() -> i64 {
    1
}

#[derive(Serialize)]
pub struct CreateSessionResponse {
    pub session_id: Uuid,
    pub ephemeral_wallet: String,
    pub vault_pda: String,
    pub approved_amount_sol: f64,
    pub approved_amount_lamports: u64,
    pub expires_at: DateTime<Utc>,
    pub transaction_signature: String,
}

#[derive(Deserialize)]
pub struct RevokeRequest {
    pub session_id: Uuid,
}

#[derive(Serialize)]
pub struct RevokeResponse {
    pub session_id: Uuid,
    pub returned_sol: f64,
    pub transaction_signature: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct RenewRequest {
    pub session_id: Uuid,
}

#[derive(Serialize)]
pub struct RenewResponse {
    pub session_id: Uuid,
    pub new_expires_at: DateTime<Utc>,
    pub transaction_signature: String,
}

#[derive(Deserialize)]
pub struct VaultStatsRequest {
    pub session_id: Uuid,
}

#[derive(Serialize)]
pub struct VaultStatsResponse {
    pub session_id: Uuid,
    pub total_deposited_sol: f64,
    pub total_withdrawn_sol: f64,
    pub available_balance_sol: f64,
    pub used_amount_sol: f64,
    pub trade_count: u64,
    pub session_status: String,
    pub is_active: bool,
    pub is_paused: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

// ---------- delegation shared ----------

#[derive(Deserialize)]
pub struct ApproveRequest {
    pub session_id: Uuid,
}

#[derive(Serialize)]
pub struct ApproveResponse {
    pub session_id: Uuid,
    pub delegate_wallet: String,
    pub expires_at: DateTime<Utc>,
    pub transaction_signature: String,
}

// ---------- auto_deposit shared ----------

#[derive(Deserialize)]
pub struct DepositRequest {
    pub session_id: Uuid,
    pub estimated_fee_sol: f64,
}

#[derive(Serialize)]
pub struct DepositResponse {
    pub session_id: Uuid,
    pub deposited_sol: f64,
    pub deposited_lamports: u64,
    pub total_deposited_sol: f64,
    pub available_balance_sol: f64,
    pub transaction_signature: String,
}

#[derive(Deserialize)]
pub struct WithdrawRequest {
    pub session_id: Uuid,
    #[serde(default)]
    pub amount_sol: f64, // 0 = withdraw all
}

#[derive(Serialize)]
pub struct WithdrawResponse {
    pub session_id: Uuid,
    pub withdrawn_sol: f64,
    pub remaining_balance_sol: f64,
    pub transaction_signature: String,
}

// ---------- signer / trade shared ----------

#[derive(Deserialize)]
pub struct ExecuteTradeRequest {
    pub session_id: Uuid,
    pub market: String,
    pub side: String,
    pub size: String,
}

#[derive(Serialize)]
pub struct ExecuteTradeResponse {
    pub session_id: Uuid,
    pub trade_number: u64,
    pub trade_fee_sol: f64,
    pub trade_amount_sol: f64,
    pub remaining_balance_sol: f64,
    pub transaction_signature: String,
}

// ============================================================================
// UTILITY HELPERS (used across multiple modules)
// ============================================================================

pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * LAMPORTS_PER_SOL as f64) as u64
}

pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / LAMPORTS_PER_SOL as f64
}

pub fn validate_approved_amount(amount_sol: f64) -> Result<u64, String> {
    let lamports = sol_to_lamports(amount_sol);
    if lamports < MIN_APPROVED_LAMPORTS {
        return Err(format!(
            "Approved amount too small. Minimum: {} SOL",
            lamports_to_sol(MIN_APPROVED_LAMPORTS)
        ));
    }
    if lamports > MAX_APPROVED_LAMPORTS {
        return Err(format!(
            "Approved amount too large. Maximum: {} SOL",
            lamports_to_sol(MAX_APPROVED_LAMPORTS)
        ));
    }
    Ok(lamports)
}

pub fn validate_deposit(lamports: u64) -> Result<(), String> {
    if lamports < MIN_DEPOSIT_LAMPORTS {
        return Err(format!(
            "Deposit too small. Minimum: {} SOL",
            lamports_to_sol(MIN_DEPOSIT_LAMPORTS)
        ));
    }
    if lamports > MAX_DEPOSIT_LAMPORTS {
        return Err(format!(
            "Deposit too large. Maximum: {} SOL",
            lamports_to_sol(MAX_DEPOSIT_LAMPORTS)
        ));
    }
    Ok(())
}

// ============================================================================
// HEALTH CHECK HANDLER
// ============================================================================

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": "1.0.0",
        "timestamp": Utc::now(),
    }))
}

// ============================================================================
// SERVER FACTORY  (called from main.rs)
// ============================================================================

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // Health
        .route("/health", web::get().to(health_check))

        // Session lifecycle — handled by session_manager
        .route("/session/create",  web::post().to(session_manager::create_session))
        .route("/session/revoke",  web::delete().to(session_manager::revoke_session))
        .route("/session/renew",   web::post().to(session_manager::renew_session))
        .route("/session/stats",   web::get().to(session_manager::get_vault_stats))

        // Delegation — handled by delegation
        .route("/session/approve", web::post().to(delegation::approve_delegate))

        // Deposits & withdrawals — handled by auto_deposit
        .route("/session/deposit",  web::post().to(auto_deposit::deposit))
        .route("/session/withdraw", web::post().to(auto_deposit::withdraw_balance))

        // Trade execution — handled by signer
        .route("/session/execute-trade", web::post().to(signer::execute_trade));
}

pub async fn build_server(
    state: AppState,
    host: &str,
    port: u16,
) -> std::io::Result<()> {
    let state = web::Data::new(state);

    println!("🚀 Ephemeral Vault Backend starting on http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .configure(configure_routes)
    })
    .bind((host, port))?
    .run()
    .await
}