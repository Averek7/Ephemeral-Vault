use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_sdk::{pubkey::Pubkey, signature::Signature};

use crate::{
    db::{models::NewTrade, queries},
    error::{AppError, Result},
    solana,
    state::AppState,
};

const MIN_APPROVED_AMOUNT_LAMPORTS: u64 = 1_000_000;
const MAX_APPROVED_AMOUNT_LAMPORTS: u64 = 1_000_000_000_000;
const MIN_DEPOSIT_LAMPORTS: u64 = 1_000_000;
const MAX_DEPOSIT_LAMPORTS: u64 = 100_000_000_000;
const MAX_SESSION_DURATION_SECONDS: i64 = 3_600;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRequest {
    user_pubkey: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVaultRequest {
    user_pubkey: String,
    approved_amount_lamports: u64,
    delegate_pubkey: Option<String>,
    custom_duration_seconds: Option<i64>,
    initial_deposit_lamports: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmountRequest {
    user_pubkey: String,
    amount_lamports: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApproveDelegateRequest {
    user_pubkey: String,
    delegate_pubkey: String,
    custom_duration_seconds: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApprovedAmountRequest {
    user_pubkey: String,
    new_approved_amount_lamports: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteTradeRequest {
    vault_pubkey: String,
    delegate_pubkey: String,
    trade_fee_lamports: u64,
    trade_amount_lamports: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupRequest {
    vault_pubkey: String,
    cleaner_pubkey: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulateTransactionRequest {
    transaction_base64: String,
}

fn parse_pubkey(raw: &str, field: &str) -> Result<Pubkey> {
    raw.parse::<Pubkey>()
        .map_err(|e| AppError::InvalidSignature(format!("invalid {field}: {e}")))
}

fn validate_lamports_range(value: u64, field: &str, min: u64, max: u64) -> Result<()> {
    if value < min || value > max {
        return Err(AppError::Validation(format!(
            "{field} must be between {min} and {max} lamports"
        )));
    }

    Ok(())
}

fn validate_positive_lamports(value: u64, field: &str) -> Result<()> {
    if value == 0 {
        return Err(AppError::Validation(format!(
            "{field} must be greater than 0"
        )));
    }

    Ok(())
}

fn validate_custom_duration(duration: Option<i64>) -> Result<()> {
    if let Some(duration) = duration {
        if duration <= 0 || duration > MAX_SESSION_DURATION_SECONDS {
            return Err(AppError::Validation(format!(
                "customDurationSeconds must be between 1 and {MAX_SESSION_DURATION_SECONDS}"
            )));
        }
    }

    Ok(())
}

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessResponse {
    status: &'static str,
    postgres: DependencyStatus,
    solana_rpc: DependencyStatus,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyStatus {
    ok: bool,
    message: String,
}

impl DependencyStatus {
    fn ok(message: impl Into<String>) -> Self {
        Self {
            ok: true,
            message: message.into(),
        }
    }

    fn failed(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            message: message.into(),
        }
    }
}

pub async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    let postgres = match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => DependencyStatus::ok("connected"),
        Err(err) => DependencyStatus::failed(format!("query failed: {err}")),
    };

    let solana_rpc = match state.rpc.get_latest_blockhash().await {
        Ok(blockhash) => DependencyStatus::ok(format!("latest blockhash {blockhash}")),
        Err(err) => DependencyStatus::failed(format!("rpc failed: {err}")),
    };

    let is_ready = postgres.ok && solana_rpc.ok;
    let status = if is_ready { "ready" } else { "not_ready" };
    let status_code = if is_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(ReadinessResponse {
            status,
            postgres,
            solana_rpc,
        }),
    )
}

pub async fn get_vault(
    State(state): State<AppState>,
    Path(user_pubkey): Path<String>,
) -> Result<Json<solana::VaultDto>> {
    let user_pubkey = user_pubkey
        .parse::<Pubkey>()
        .map_err(|e| AppError::InvalidSignature(format!("invalid user pubkey: {e}")))?;
    let vault = solana::fetch_vault_by_user(&state.rpc, &state.config, user_pubkey).await?;
    Ok(Json(vault))
}

pub async fn get_vault_stats(
    State(state): State<AppState>,
    Path(user_pubkey): Path<String>,
) -> Result<Json<solana::VaultStatsDto>> {
    let user_pubkey = user_pubkey
        .parse::<Pubkey>()
        .map_err(|e| AppError::InvalidSignature(format!("invalid user pubkey: {e}")))?;
    let stats = solana::fetch_vault_stats_by_user(&state.rpc, &state.config, user_pubkey).await?;
    Ok(Json(stats))
}

pub async fn get_trades(
    State(state): State<AppState>,
    Path(vault_pubkey): Path<String>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<Vec<crate::db::models::TradeRecord>>> {
    let limit = query.limit.clamp(1, 100);
    let offset = query.offset.max(0);
    let trades = queries::get_trades_for_vault(&state.db, &vault_pubkey, limit, offset).await?;
    Ok(Json(trades))
}

pub async fn create_trade(
    State(state): State<AppState>,
    Json(body): Json<NewTrade>,
) -> Result<Json<crate::db::models::TradeRecord>> {
    body.validate()?;
    let trade = queries::insert_trade(&state.db, &body).await?;
    Ok(Json(trade))
}

pub async fn tx_create_vault(
    State(state): State<AppState>,
    Json(body): Json<CreateVaultRequest>,
) -> Result<Json<solana::TxEnvelope>> {
    let user = parse_pubkey(&body.user_pubkey, "userPubkey")?;
    let delegate = body
        .delegate_pubkey
        .as_deref()
        .map(|raw| parse_pubkey(raw, "delegatePubkey"))
        .transpose()?;
    validate_lamports_range(
        body.approved_amount_lamports,
        "approvedAmountLamports",
        MIN_APPROVED_AMOUNT_LAMPORTS,
        MAX_APPROVED_AMOUNT_LAMPORTS,
    )?;
    validate_custom_duration(body.custom_duration_seconds)?;
    if let Some(amount) = body.initial_deposit_lamports.filter(|amount| *amount > 0) {
        validate_lamports_range(
            amount,
            "initialDepositLamports",
            MIN_DEPOSIT_LAMPORTS,
            MAX_DEPOSIT_LAMPORTS,
        )?;
    }

    let tx = solana::build_create_vault_tx(
        &state.rpc,
        &state.config,
        user,
        body.approved_amount_lamports,
        delegate,
        body.custom_duration_seconds,
        body.initial_deposit_lamports,
    )
    .await?;

    Ok(Json(tx))
}

pub async fn tx_deposit(
    State(state): State<AppState>,
    Json(body): Json<AmountRequest>,
) -> Result<Json<solana::TxEnvelope>> {
    let user = parse_pubkey(&body.user_pubkey, "userPubkey")?;
    validate_lamports_range(
        body.amount_lamports,
        "amountLamports",
        MIN_DEPOSIT_LAMPORTS,
        MAX_DEPOSIT_LAMPORTS,
    )?;
    let tx =
        solana::build_deposit_tx(&state.rpc, &state.config, user, body.amount_lamports).await?;
    Ok(Json(tx))
}

pub async fn tx_withdraw(
    State(state): State<AppState>,
    Json(body): Json<AmountRequest>,
) -> Result<Json<solana::TxEnvelope>> {
    let user = parse_pubkey(&body.user_pubkey, "userPubkey")?;
    if body.amount_lamports > 0 {
        validate_positive_lamports(body.amount_lamports, "amountLamports")?;
    }
    let tx =
        solana::build_withdraw_tx(&state.rpc, &state.config, user, body.amount_lamports).await?;
    Ok(Json(tx))
}

pub async fn tx_pause(
    State(state): State<AppState>,
    Json(body): Json<UserRequest>,
) -> Result<Json<solana::TxEnvelope>> {
    let user = parse_pubkey(&body.user_pubkey, "userPubkey")?;
    let tx = solana::build_pause_tx(&state.rpc, &state.config, user).await?;
    Ok(Json(tx))
}

pub async fn tx_unpause(
    State(state): State<AppState>,
    Json(body): Json<UserRequest>,
) -> Result<Json<solana::TxEnvelope>> {
    let user = parse_pubkey(&body.user_pubkey, "userPubkey")?;
    let tx = solana::build_unpause_tx(&state.rpc, &state.config, user).await?;
    Ok(Json(tx))
}

pub async fn tx_revoke(
    State(state): State<AppState>,
    Json(body): Json<UserRequest>,
) -> Result<Json<solana::TxEnvelope>> {
    let user = parse_pubkey(&body.user_pubkey, "userPubkey")?;
    let tx = solana::build_revoke_tx(&state.rpc, &state.config, user).await?;
    Ok(Json(tx))
}

pub async fn tx_renew_session(
    State(state): State<AppState>,
    Json(body): Json<UserRequest>,
) -> Result<Json<solana::TxEnvelope>> {
    let user = parse_pubkey(&body.user_pubkey, "userPubkey")?;
    let tx = solana::build_renew_session_tx(&state.rpc, &state.config, user).await?;
    Ok(Json(tx))
}

pub async fn tx_approve_delegate(
    State(state): State<AppState>,
    Json(body): Json<ApproveDelegateRequest>,
) -> Result<Json<solana::TxEnvelope>> {
    let user = parse_pubkey(&body.user_pubkey, "userPubkey")?;
    let delegate = parse_pubkey(&body.delegate_pubkey, "delegatePubkey")?;
    validate_custom_duration(body.custom_duration_seconds)?;
    let tx = solana::build_approve_delegate_tx(
        &state.rpc,
        &state.config,
        user,
        delegate,
        body.custom_duration_seconds,
    )
    .await?;
    Ok(Json(tx))
}

pub async fn tx_reactivate(
    State(state): State<AppState>,
    Json(body): Json<UserRequest>,
) -> Result<Json<solana::TxEnvelope>> {
    let user = parse_pubkey(&body.user_pubkey, "userPubkey")?;
    let tx = solana::build_reactivate_tx(&state.rpc, &state.config, user).await?;
    Ok(Json(tx))
}

pub async fn tx_update_approved_amount(
    State(state): State<AppState>,
    Json(body): Json<UpdateApprovedAmountRequest>,
) -> Result<Json<solana::TxEnvelope>> {
    let user = parse_pubkey(&body.user_pubkey, "userPubkey")?;
    validate_lamports_range(
        body.new_approved_amount_lamports,
        "newApprovedAmountLamports",
        MIN_APPROVED_AMOUNT_LAMPORTS,
        MAX_APPROVED_AMOUNT_LAMPORTS,
    )?;
    let tx = solana::build_update_approved_amount_tx(
        &state.rpc,
        &state.config,
        user,
        body.new_approved_amount_lamports,
    )
    .await?;
    Ok(Json(tx))
}

pub async fn tx_execute_trade(
    State(state): State<AppState>,
    Json(body): Json<ExecuteTradeRequest>,
) -> Result<Json<solana::TxEnvelope>> {
    let vault = parse_pubkey(&body.vault_pubkey, "vaultPubkey")?;
    let delegate = parse_pubkey(&body.delegate_pubkey, "delegatePubkey")?;
    validate_positive_lamports(body.trade_fee_lamports, "tradeFeeLamports")?;
    validate_lamports_range(
        body.trade_amount_lamports,
        "tradeAmountLamports",
        MIN_APPROVED_AMOUNT_LAMPORTS,
        MAX_APPROVED_AMOUNT_LAMPORTS,
    )?;
    let tx = solana::build_execute_trade_tx(
        &state.rpc,
        &state.config,
        vault,
        delegate,
        body.trade_fee_lamports,
        body.trade_amount_lamports,
    )
    .await?;
    Ok(Json(tx))
}

pub async fn tx_cleanup(
    State(state): State<AppState>,
    Json(body): Json<CleanupRequest>,
) -> Result<Json<solana::TxEnvelope>> {
    let vault = parse_pubkey(&body.vault_pubkey, "vaultPubkey")?;
    let cleaner = parse_pubkey(&body.cleaner_pubkey, "cleanerPubkey")?;
    let tx = solana::build_cleanup_tx(&state.rpc, &state.config, vault, cleaner).await?;
    Ok(Json(tx))
}

pub async fn tx_simulate(
    State(state): State<AppState>,
    Json(body): Json<SimulateTransactionRequest>,
) -> Result<Json<solana::TxSimulationDto>> {
    if body.transaction_base64.trim().is_empty() {
        return Err(AppError::Validation(
            "transactionBase64 must not be empty".into(),
        ));
    }

    let simulation =
        solana::simulate_transaction_base64(&state.rpc, &body.transaction_base64).await?;
    Ok(Json(simulation))
}

pub async fn tx_status(
    State(state): State<AppState>,
    Path(signature): Path<String>,
) -> Result<Json<solana::TxStatusDto>> {
    let signature = signature
        .parse::<Signature>()
        .map_err(|e| AppError::Validation(format!("invalid signature: {e}")))?;
    let status = solana::fetch_transaction_status(&state.rpc, signature).await?;
    Ok(Json(status))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_lamport_ranges() {
        assert!(validate_lamports_range(1_000_000, "amount", 1_000_000, 2_000_000).is_ok());
        assert!(validate_lamports_range(999_999, "amount", 1_000_000, 2_000_000).is_err());
        assert!(validate_lamports_range(2_000_001, "amount", 1_000_000, 2_000_000).is_err());
    }

    #[test]
    fn validates_positive_lamports() {
        assert!(validate_positive_lamports(1, "amount").is_ok());
        assert!(validate_positive_lamports(0, "amount").is_err());
    }

    #[test]
    fn validates_custom_duration_bounds() {
        assert!(validate_custom_duration(None).is_ok());
        assert!(validate_custom_duration(Some(1)).is_ok());
        assert!(validate_custom_duration(Some(MAX_SESSION_DURATION_SECONDS)).is_ok());
        assert!(validate_custom_duration(Some(0)).is_err());
        assert!(validate_custom_duration(Some(MAX_SESSION_DURATION_SECONDS + 1)).is_err());
    }
}
