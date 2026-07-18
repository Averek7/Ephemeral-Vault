use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TradeRecord {
    pub id: Uuid,
    pub vault_address: String,
    pub tx_hash: String,
    pub trade_type: String,
    pub amount_sol: f64,
    pub fee_sol: f64,
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

impl NewTrade {
    pub fn validate(&self) -> Result<()> {
        validate_required_text(&self.vault_address, "vault_address")?;
        validate_required_text(&self.tx_hash, "tx_hash")?;
        validate_required_text(&self.trade_type, "trade_type")?;
        validate_required_text(&self.status, "status")?;

        if !self.amount_sol.is_finite() || self.amount_sol < 0.0 {
            return Err(AppError::Validation(
                "amount_sol must be a finite non-negative number".into(),
            ));
        }

        if !self.fee_sol.is_finite() || self.fee_sol < 0.0 {
            return Err(AppError::Validation(
                "fee_sol must be a finite non-negative number".into(),
            ));
        }

        if matches!(self.slot, Some(slot) if slot < 0) {
            return Err(AppError::Validation("slot must be non-negative".into()));
        }

        Ok(())
    }
}

fn validate_required_text(value: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(AppError::Validation(format!("{field} must not be empty")));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_trade() -> NewTrade {
        NewTrade {
            vault_address: "vault".into(),
            tx_hash: "tx".into(),
            trade_type: "buy".into(),
            amount_sol: 1.0,
            fee_sol: 0.000005,
            status: "confirmed".into(),
            slot: Some(42),
        }
    }

    #[test]
    fn validates_trade_payload() {
        assert!(valid_trade().validate().is_ok());
    }

    #[test]
    fn rejects_empty_trade_text_fields() {
        let mut trade = valid_trade();
        trade.tx_hash = " ".into();

        assert!(trade.validate().is_err());
    }

    #[test]
    fn rejects_negative_amounts() {
        let mut trade = valid_trade();
        trade.amount_sol = -1.0;

        assert!(trade.validate().is_err());
    }
}
