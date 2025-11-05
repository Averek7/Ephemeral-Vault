use crate::session::manager::SessionManager;
use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use std::time::Duration;
use tokio::time::sleep;

pub struct TransactionSigner {
    rpc: RpcClient,
    session_manager: SessionManager,
    max_retries: usize,
}

impl TransactionSigner {
    pub fn new(rpc_url: &str, sm: SessionManager) -> Self {
        Self {
            rpc: RpcClient::new(rpc_url.to_string()),
            session_manager: sm,
            max_retries: 5,
        }
    }

    pub fn send_signed_tx(
        &self,
        keypair: &Keypair,
        mut tx: Transaction,
    ) -> Result<solana_client::rpc_response::RpcSignatureResult> {
        // sign and send
        tx.sign(&[keypair], self.rpc.get_latest_blockhash()?);
        let sig = self.rpc.send_and_confirm_transaction(&tx)?;
        Ok(sig)
    }

    /// High-level: sign a simple transfer from ephemeral -> dest
    pub async fn transfer_from_ephemeral(
        &self,
        session: &crate::session::manager::SessionRecord,
        dest: &Pubkey,
        amount: u64,
    ) -> Result<()> {
        let kp = self.session_manager.load_keypair(session)?;
        let ix = system_instruction::transfer(&kp.pubkey(), dest, amount);
        let (recent_blockhash, _fee_calc) = self.rpc.get_latest_blockhash_with_commitment(
            solana_sdk::commitment_config::CommitmentConfig::confirmed(),
        )?;
        let tx =
            Transaction::new_signed_with_payer(&[ix], Some(&kp.pubkey()), &[&kp], recent_blockhash);

        let mut attempt = 0usize;
        loop {
            attempt += 1;
            match self.rpc.send_and_confirm_transaction_with_spinner(&tx) {
                Ok(sig) => {
                    println!("tx success {}", sig);
                    return Ok(());
                }
                Err(e) => {
                    if attempt >= self.max_retries {
                        return Err(anyhow::anyhow!("tx failed after retries: {:?}", e));
                    }
                    let backoff = Duration::from_millis(500 * (attempt as u64));
                    println!("tx failed, retrying after {:?}: {:?}", backoff, e);
                    sleep(backoff).await;
                }
            }
        }
    }
}
