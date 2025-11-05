use anchor_client::solana_sdk::{
    signature::{Keypair, Signer},
    pubkey::Pubkey,
};
use anyhow::Result;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::db;
use super::encryption;

pub struct SessionManager {
    encryption_key: Vec<u8>,  // 32 bytes AES key from ENV
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionRecord {
    pub user_wallet: Pubkey,
    pub ephemeral_wallet: Pubkey,
    pub encrypted_keypair: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: i64,
    pub expires_at: i64,
}

impl SessionManager {
    pub fn new(encrypted_key: Vec<u8>) -> Self {
        Self { encryption_key: encrypted_key }
    }

    pub fn create_session(&self, user_wallet: Pubkey, ttl_secs: i64) -> Result<SessionRecord> {
        let keypair = Keypair::new();

        let serialized_keypair = keypair.to_bytes();
        let (nonce, encrypted) = encryption::encrypt_bytes(&self.encryption_key, &serialized_keypair)?;

        let session = SessionRecord {
            user_wallet,
            ephemeral_wallet: keypair.pubkey(),
            encrypted_keypair: encrypted,
            nonce,
            created_at: Utc::now().timestamp(),
            expires_at: (Utc::now() + Duration::seconds(ttl_secs)).timestamp(),
        };

        db::store_session(&session)?;
        Ok(session)
    }

    pub fn load_keypair(&self, session: &SessionRecord) -> Result<Keypair> {
        let decrypted =
            encryption::decrypt_bytes(&self.encryption_key, &session.nonce, &session.encrypted_keypair)?;

        Ok(Keypair::from_bytes(&decrypted)?)
    }

    pub fn cleanup_expired(&self) -> Result<()> {
        let expired = db::fetch_expired_sessions(Utc::now().timestamp())?;

        for session in expired {
            println!("♻️ Cleaning expired session {}", session.ephemeral_wallet);
            db::delete_session(&session)?;
        }

        Ok(())
    }
}
