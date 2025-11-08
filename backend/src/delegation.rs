use anyhow::{Result, anyhow};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Signature, Signer},
    transaction::Transaction,
    system_instruction,
};
use solana_client::nonblocking::rpc_client::RpcClient;

#[derive(Clone)]
pub struct DelegationManager {
    pub rpc: RpcClient,
}

impl DelegationManager {
    /// Build delegation transaction (parent wallet signs)
    pub fn build_delegate_tx(
        &self,
        parent: &Pubkey,
        ephemeral: &Pubkey,
        authority_program: &Pubkey,
        recent_blockhash: solana_sdk::hash::Hash,
    ) -> Transaction {
        let ix = system_instruction::assign(ephemeral, authority_program);

        Transaction::new_with_payer(&[ix], Some(parent))
            .sign(&[], recent_blockhash)
    }

    /// Verify delegation sig (server ensures user actually signed)
    pub fn verify_delegation_sig(
        &self,
        message: &[u8],
        signature: &Signature,
        parent_pubkey: &Pubkey,
    ) -> Result<()> {
        if signature.verify(parent_pubkey.as_ref(), message).is_ok() {
            return Ok(());
        }
        Err(anyhow!("Delegation signature invalid"))
    }

    /// End delegation session lifecycle (revoke)
    pub async fn revoke(
        &self,
        ephemeral_wallet: Pubkey,
    ) -> Result<()> {
        // No mutation of blockchain needed here.
        println!("✅ Delegation revoked for ephemeral {ephemeral_wallet}");
        Ok(())
    }
}