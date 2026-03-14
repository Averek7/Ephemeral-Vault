use sqlx::PgPool;
use crate::db::models::{NewTrade, TradeRecord};
use crate::error::Result;

pub async fn insert_trade(pool: &PgPool, trade: &NewTrade) -> Result<TradeRecord> {
    let rec = sqlx::query_as::<_, TradeRecord>(
        r#"
        INSERT INTO trades (id, vault_address, tx_hash, trade_type, amount_sol, fee_sol, status, slot)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, vault_address, tx_hash, trade_type, amount_sol, fee_sol, status, slot, created_at
        "#,
    )
    .bind(uuid::Uuid::new_v4())
    .bind(&trade.vault_address)
    .bind(&trade.tx_hash)
    .bind(&trade.trade_type)
    .bind(trade.amount_sol)
    .bind(trade.fee_sol)
    .bind(&trade.status)
    .bind(trade.slot)
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
    let records = sqlx::query_as::<_, TradeRecord>(
        r#"
        SELECT id, vault_address, tx_hash, trade_type, amount_sol, fee_sol, status, slot, created_at
        FROM trades
        WHERE vault_address = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(vault_address)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(records)
}

pub async fn count_trades_for_vault(pool: &PgPool, vault_address: &str) -> Result<i64> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint FROM trades WHERE vault_address = $1",
    )
    .bind(vault_address)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn get_recent_trades(pool: &PgPool, limit: i64) -> Result<Vec<TradeRecord>> {
    let records = sqlx::query_as::<_, TradeRecord>(
        "SELECT id, vault_address, tx_hash, trade_type, amount_sol, fee_sol, status, slot, created_at FROM trades ORDER BY created_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(records)
}
