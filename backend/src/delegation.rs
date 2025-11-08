use anyhow::Result;
use ed25519_dalek::{Signature as DalekSignature, PublicKey as DalekPublicKey, Verifier};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature as SolSignature;
use solana_sdk::signature::Keypair as SolKeypair;
use crate::session_manager::SessionService;
use uuid::Uuid;
use chrono::Utc;

pub struct DelegationManager {}

impl DelegationManager {
    pub fn new() -> Self { Self {} }

    /// Build delegation message to be signed by parent wallet (canonical).
    pub fn build_delegation_message(ephemeral: &Pubkey, vault_pda: &str, session_id: &Uuid, expiry_unix: i64) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(ephemeral.as_ref());
        out.extend_from_slice(vault_pda.as_bytes());
        out.extend_from_slice(session_id.as_bytes());
        out.extend_from_slice(&expiry_unix.to_le_bytes());
        out
    }

    /// Verify ed25519 signature bytes produced by parent over message.
    pub fn verify_signature(parent_pubkey: &Pubkey, message: &[u8], sig_bytes: &[u8]) -> Result<bool> {
        let dalek_pk = DalekPublicKey::from_bytes(parent_pubkey.as_ref())?;
        let sig = DalekSignature::from_bytes(sig_bytes)?;
        Ok(dalek_pk.verify(message, &sig).is_ok())
    }

    /// Activate delegation locally in DB after verification.
    pub async fn activate_delegation(
        session_svc: &SessionService,
        session_id: Uuid,
        delegate_pubkey: &Pubkey,
        approved_amount: i64,
        expiry_unix: i64,
    ) -> Result<()> {
        session_svc.set_delegation(session_id, &delegate_pubkey.to_string(), approved_amount, chrono::DateTime::<Utc>::from_utc(chrono::NaiveDateTime::from_timestamp_opt(expiry_unix, 0).unwrap(), Utc)).await?;
        Ok(())
    }

    pub async fn publish_on_chain(_parent: &SolKeypair, _program_id: &Pubkey) -> Result<SolSignature> {
        // TODO: implement specific program interaction
        Err(anyhow::anyhow!("on-chain publish not implemented in this example"))
    }
}
