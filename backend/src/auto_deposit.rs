use anyhow::{Result, anyhow};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, commitment_config::CommitmentConfig};

const MIN_REQUIRED_BUFFER_SOL: f64 = 0.015;    // avoid lamport starvation
const MAX_AUTO_DEPOSIT_SOL: f64 = 1.0;         // upper safety limit

#[derive(Debug, Clone)]
pub struct AutoDepositCalculator {
    pub rpc: RpcClient,
}

impl AutoDepositCalculator {
    /// Estimate expected SOL spending for N trading operations
    pub fn estimate_trading_cost(&self, tx_count: u32) -> f64 {
        let avg_fee_per_tx = 0.000005; // ~5k lamports, approx typical Solana tx fee
        tx_count as f64 * avg_fee_per_tx
    }

    /// Compute optimal amount to deposit (vault balance + forecasted trading ops)
    pub fn calculate_optimal_deposit(
        &self,
        tx_count: u32,
        current_vault_sol: f64,
    ) -> f64 {
        let required = self.estimate_trading_cost(tx_count) + MIN_REQUIRED_BUFFER_SOL;

        if current_vault_sol >= required {
            return 0.0; // no deposit needed
        }

        let diff = required - current_vault_sol;
        diff.min(MAX_AUTO_DEPOSIT_SOL)
    }

    /// Read vault SOL balance from chain
    pub async fn get_balance(&self, vault_wallet: &Pubkey) -> Result<f64> {
        let balance = self
            .rpc
            .get_balance_with_commitment(vault_wallet, CommitmentConfig::confirmed())
            .await
            .map_err(|_| anyhow!("Unable to fetch vault balance"))?
            .value;

        Ok(balance as f64 / 1_000_000_000f64)
    }

    /// Decide whether system must trigger a top-up
    pub async fn needs_top_up(
        &self,
        vault_wallet: &Pubkey,
        estimated_tx_count: u32,
    ) -> Result<bool> {
        let current = self.get_balance(vault_wallet).await?;
        let required = self.calculate_optimal_deposit(estimated_tx_count, current);
        Ok(required > 0.0)
    }
}