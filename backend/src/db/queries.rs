use sqlx::PgPool;
use uuid::Uuid;
use crate::db::models::{NewTrade, TradeRecord};
use crate::error::Result;

pub async fn insert_trade(pool: &PgPool, trade: &NewTrade) -> Result<TradeRecord> {
    let rec = sqlx::query_as!(
        TradeRecord,
        r#"
        INSERT INTO trades (id, vault_address, tx_hash, trade_type, amount_sol, fee_sol, status, slot)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
        Uuid::new_v4(),
        trade.vault_address,
        trade.tx_hash,
        trade.trade_type,
        trade.amount_sol,
        trade.fee_sol,
        trade.status,
        trade.slot,
    )
    .fetch_one(pool)
    .await?;
    Ok(rec)
}

pub async fn get_trades_for_vault(
    pool: &PgPool,
    vault_address: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<TradeRecord>> {
    let records = sqlx::query_as!(
        TradeRecord,
        r#"
        SELECT * FROM trades
        WHERE vault_address = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        vault_address,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;
    Ok(records)
}

pub async fn count_trades_for_vault(pool: &PgPool, vault_address: &str) -> Result<i64> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM trades WHERE vault_address = $1",
        vault_address
    )
    .fetch_one(pool)
    .await?;
    Ok(count.unwrap_or(0))
}

pub async fn get_recent_trades(pool: &PgPool, limit: i64) -> Result<Vec<TradeRecord>> {
    let records = sqlx::query_as!(
        TradeRecord,
        "SELECT * FROM trades ORDER BY created_at DESC LIMIT $1",
        limit,
    )
    .fetch_all(pool)
    .await?;
    Ok(records)
}
