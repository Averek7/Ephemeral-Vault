use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use solana_sdk::pubkey::Pubkey;

use crate::{
    db::{models::NewTrade, queries},
    error::{AppError, Result},
    solana,
    state::AppState,
};

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

fn parse_pubkey(raw: &str, field: &str) -> Result<Pubkey> {
    raw.parse::<Pubkey>()
        .map_err(|e| AppError::InvalidSignature(format!("invalid {field}: {e}")))
}

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
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
    let tx = solana::build_deposit_tx(&state.rpc, &state.config, user, body.amount_lamports).await?;
    Ok(Json(tx))
}

pub async fn tx_withdraw(
    State(state): State<AppState>,
    Json(body): Json<AmountRequest>,
) -> Result<Json<solana::TxEnvelope>> {
    let user = parse_pubkey(&body.user_pubkey, "userPubkey")?;
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
    let tx = solana::build_update_approved_amount_tx(
        &state.rpc,
        &state.config,
        user,
        body.new_approved_amount_lamports,
    )
    .await?;
    Ok(Json(tx))
}
