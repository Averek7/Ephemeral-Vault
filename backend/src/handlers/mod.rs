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
