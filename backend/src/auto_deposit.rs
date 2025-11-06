use anyhow::Result;

pub struct AutoDepositCalculator;

impl AutoDepositCalculator {
    pub async fn estimate_fee_lamports() -> Result<u64> {
        Ok(5_000) // 5000 lamports as example
    }

    pub async fn optimal_deposit(current_balance: u64, buffer: u64) -> Result<u64> {
        let fee = Self::estimate_fee_lamports().await?;
        if current_balance >= fee + buffer {
            Ok(0)
        } else {
            Ok((fee + buffer).saturating_sub(current_balance))
        }
    }
}