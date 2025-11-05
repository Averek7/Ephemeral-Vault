use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature},
};
use anyhow::Result;
use chrono::Utc;
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DelegationPayload {
    pub parent: Pubkey,
    pub session: Pubkey,
    pub vault: Pubkey,
    pub approved_amount: u64,
    pub nonce: u64,
    pub expiry_unix: i64,
    pub scopes: Vec<String>,
}

pub struct DelegationManager {
    // DB handle, or session manager ref
}

impl DelegationManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build_payload(
        &self,
        parent: Pubkey,
        session: Pubkey,
        vault: Pubkey,
        approved_amount: u64,
        ttl_seconds: i64,
        scopes: Vec<String>,
    ) -> DelegationPayload {
        let nonce: u64 = rand::random();
        DelegationPayload {
            parent,
            session,
            vault,
            approved_amount,
            nonce,
            expiry_unix: (Utc::now().timestamp() + ttl_seconds),
            scopes,
        }
    }

    /// Convert payload to canonical bytes for signing (deterministic)
    pub fn payload_bytes(payload: &DelegationPayload) -> Vec<u8> {
        // Use serde_bincode or canonical JSON with sorted keys
        bincode::serialize(payload).unwrap()
    }

    /// Verify a signature (signed by parent)
    pub fn verify_signature(msg: &[u8], sig_bytes: &[u8], signer: &Pubkey) -> Result<()> {
        use solana_sdk::pubkey::Pubkey as SolKey;
        use solana_sdk::signature::Signature;
        let sig = Signature::new(sig_bytes);
        // This verifies using ed25519 with Solana's verification utilities:
        if sig.verify(signer.as_ref(), msg) {
            Ok(())
        } else {
            anyhow::bail!("invalid delegation signature")
        }
    }

    /// Save signed delegation into DB (session + signature + payload)
    pub fn store_delegation(&self, payload: &DelegationPayload, signature: Vec<u8>) -> Result<()> {
        // db::store_delegation_record(...)
        Ok(())
    }
}
