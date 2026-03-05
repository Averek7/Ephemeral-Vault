use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TradeRecord {
    pub id: Uuid,
    pub vault_address: String,
    pub tx_hash: String,
    pub trade_type: String,
    pub amount_sol: f64,
    pub fee_sol
    : f64,
    pub status: String,
    pub slot: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VaultSnapshot {
    pub id: Uuid,
    pub vault_address: String,
    pub owner: String,
    pub balance_sol: Option<f64>,
    pub approved_amount_sol: Option<f64>,
    pub trades_executed: Option<i32>,
    pub status: Option<String>,
    pub snapshot_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTrade {
    pub vault_address: String,
    pub tx_hash: String,
    pub trade_type: String,
    pub amount_sol: f64,
    pub fee_sol: f64,
    pub status: String,
    pub slot: Option<i64>,
}
