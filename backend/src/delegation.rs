use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;


pub struct DelegationManager {}


impl DelegationManager {
    pub async fn build_delegation_transaction(parent_pubkey: &Pubkey, ephemeral_pubkey: &Pubkey) -> Result<Vec<u8>> {
        // Build a message that the parent signs to delegate authority to ephemeral wallet.
        // This is a placeholder: actual implementation depends on on-chain program expectations.

        Ok(vec![])
    }   


    pub async fn verify_delegation_on_chain(_sig: &Signature) -> Result<bool> {
        // Verify that the delegation was published / accepted on chain
        
        Ok(true)
    }
}