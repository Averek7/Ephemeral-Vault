use anyhow::{Result, anyhow};
use solana_sdk::{
    signature::Signer,
    transaction::Transaction,
};
use solana_client::nonblocking::rpc_client::RpcClient;

#[derive(Clone)]
pub struct TransactionSigner {
    pub rpc: RpcClient,
}

impl TransactionSigner {
    pub async fn sign_and_send(
        &self,
        tx: &mut Transaction,
        wallet: &impl Signer,
    ) -> Result<String> {
        let blockhash = self.rpc.get_latest_blockhash().await?;
        tx.try_sign(&[wallet], blockhash)?;

        let sig = self.rpc.send_transaction(tx).await?;
        Ok(sig.to_string())
    }

    /// Retry with priority fee
    pub async fn retry_with_priority(
        &self,
        tx: &mut Transaction,
        wallet: &impl Signer,
        priority_fee: u64,
    ) -> Result<String> {
        tx.message.recent_blockhash = self.rpc.get_latest_blockhash().await?;
        tx.message.header.num_required_signatures += priority_fee as u8;

        let sig = self.rpc.send_and_confirm_transaction(tx).await?;
        Ok(sig.to_string())
    }
}