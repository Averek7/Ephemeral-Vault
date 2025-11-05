use anyhow::Result;
use anchor_client::solana_sdk::pubkey::Pubkey;
use solana_client::rpc_client::RpcClient;
use std::cmp;

pub struct AutoDepositCalculator {
    rpc: RpcClient,
}

impl AutoDepositCalculator {
    pub fn new(rpc_url: String) -> Self {
        Self { rpc: RpcClient::new(rpc_url) }
    }

    /// Estimate SOL needed for upcoming trading fees
    pub fn estimate_fee(&self, expected_tx_count: u64) -> u64 {
        const AVG_FEE: u64 = 5000; // ~0.000005 SOL
        expected_tx_count * AVG_FEE
    }

    /// Trigger auto-deposit only when vault balance is low
    pub fn needs_top_up(&self, vault_key: &Pubkey, threshold: u64) -> Result<bool> {
        let balance = self.rpc.get_balance(vault_key)?;
        Ok(balance < threshold)
    }

    /// Decide optimal deposit
    pub fn calculate_optimal_deposit(&self, desired_budget: u64, approved_limit: u64) -> u64 {
        cmp::min(desired_budget, approved_limit)
    }
}
