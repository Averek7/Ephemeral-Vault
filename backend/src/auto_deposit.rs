use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use std::time::Duration;
use tokio::time::sleep;

/// Simple auto-deposit calculator and top-up helper
pub struct AutoDepositCalculator {
    pub rpc: RpcClient,
}

impl AutoDepositCalculator {
    pub fn new(rpc_url: &str) -> Self {
        Self { rpc: RpcClient::new(rpc_url.to_string()) }
    }

    pub fn estimate_fee_lamports(&self) -> u64 {
        5_000u64 // 0.000005 SOL
    }

    pub fn optimal_deposit(&self, current_balance: u64, buffer: u64) -> u64 {
        let fee = self.estimate_fee_lamports();
        let target = fee.saturating_add(buffer);
        if current_balance >= target { 0 } else { target - current_balance }
    }

    pub async fn top_up_vault(
        &self,
        parent: &Keypair,
        vault: &Pubkey,
        amount: u64,
    ) -> Result<solana_sdk::signature::Signature> {
        let mut attempt = 0u32;
        let mut delay = Duration::from_millis(200);
        loop {
            attempt += 1;
            let blockhash = self.rpc.get_latest_blockhash()?;
            let ix = system_instruction::transfer(&parent.pubkey(), vault, amount);
            let tx = Transaction::new_signed_with_payer(&[ix], Some(&parent.pubkey()), &[parent], blockhash);
            match self.rpc.send_and_confirm_transaction(&tx) {
                Ok(sig) => return Ok(sig),
                Err(e) => {
                    tracing::warn!("top_up failed attempt {}: {:?}", attempt, e);
                    if attempt >= 5 { return Err(anyhow::anyhow!("top_up failed: {:?}", e)); }
                    sleep(delay).await;
                    delay *= 2;
                }
            }
        }
    }
}
