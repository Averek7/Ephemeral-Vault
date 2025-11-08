use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use solana_sdk::signature::{Keypair, Signer};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;
use ring::aead;
use std::sync::Arc;

#[derive(Clone)]
pub struct SessionService {
    pub db: Arc<PgPool>,
    pub sealing_key: aead::LessSafeKey,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EphemeralSession {
    pub id: Uuid,
    pub user: String,
    pub ephemeral_wallet: String,
    pub encrypted_key: Vec<u8>,
    pub vault_pda: String,
    pub expires_at: DateTime<Utc>,
    pub approved_amount: i64,
    pub total_deposited: i64,
    pub delegate_pubkey: Option<String>,
    pub delegate_approved: bool,
}

#[derive(Deserialize)]
pub struct EphemeralSessionCreateRequest {
    pub user: String,
}

#[derive(Serialize)]
pub struct EphemeralSessionCreateResponse {
    pub id: Uuid,
    pub user: String,
    pub ephemeral_wallet: String,
    pub vault_pda: String,
    pub expires_at: DateTime<Utc>,
}

impl From<EphemeralSession> for EphemeralSessionCreateResponse {
    fn from(s: EphemeralSession) -> Self {
        EphemeralSessionCreateResponse {
            id: s.id,
            user: s.user,
            ephemeral_wallet: s.ephemeral_wallet,
            vault_pda: s.vault_pda,
            expires_at: s.expires_at,
        }
    }
}

impl SessionService {
    pub fn new(db: Arc<PgPool>, sealing_key: aead::LessSafeKey) -> Self {
        Self { db, sealing_key }
    }

    pub async fn create_session(&self, user: &str) -> Result<EphemeralSession> {
        // Generate ephemeral Solana keypair
        let kp = Keypair::generate(&mut OsRng);
        let pubkey_str = kp.pubkey().to_string();

        // Encrypt private key bytes
        let sk_bytes = kp.to_bytes(); // 64 bytes
        // create random nonce (12 bytes)
        let mut nonce_bytes = [0u8; 12];
        getrandom::getrandom(&mut nonce_bytes)?;
        let nonce = aead::Nonce::try_assume_unique_for_key(nonce_bytes)?;
        let aad = aead::Aad::empty();

        let mut in_out = sk_bytes.to_vec();
        in_out.extend_from_slice(&[0u8; aead::MAX_TAG_LEN]); // space for tag
        self.sealing_key.seal_in_place_append_tag(nonce, aad, &mut in_out)
            .map_err(|e| anyhow::anyhow!("seal failed: {:?}", e))?;

        // store nonce + ciphertext in DB
        let mut encrypted_blob = nonce_bytes.to_vec();
        encrypted_blob.extend_from_slice(&in_out);

        let id = Uuid::new_v4();
        let vault_pda = format!("vault_{}", id);
        let expires_at = Utc::now() + chrono::Duration::minutes(15);

        sqlx::query!(
            r#"
            INSERT INTO sessions (id, user_id, ephemeral_pubkey, encrypted_key, vault_pda, expires_at, approved_amount, total_deposited, delegate_approved)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            id,
            user,
            pubkey_str,
            &encrypted_blob,
            vault_pda,
            expires_at,
            0i64,
            0i64,
            false
        )
        .execute(self.db.as_ref())
        .await?;

        Ok(EphemeralSession {
            id,
            user: user.to_string(),
            ephemeral_wallet: pubkey_str,
            encrypted_key: encrypted_blob,
            vault_pda,
            expires_at,
            approved_amount: 0,
            total_deposited: 0,
            delegate_pubkey: None,
            delegate_approved: false,
        })
    }

    pub async fn get_session(&self, id: Uuid) -> Result<Option<EphemeralSession>> {
        let rec = sqlx::query_as!(
            EphemeralSession,
            r#"
            SELECT id, user_id as "user!", ephemeral_pubkey as "ephemeral_wallet!", encrypted_key, vault_pda, expires_at, approved_amount, total_deposited, delegate_pubkey, delegate_approved
            FROM sessions
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.db.as_ref())
        .await?;
        Ok(rec)
    }

    /// Save approved delegation info (called after verifying parent signature)
    pub async fn set_delegation(&self, id: Uuid, delegate_pubkey: &str, approved_amount: i64, expires_at: chrono::DateTime<Utc>) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE sessions
            SET delegate_pubkey = $2, delegate_approved = true, approved_amount = $3, expires_at = $4
            WHERE id = $1
            "#,
            id,
            delegate_pubkey,
            approved_amount,
            expires_at
        )
        .execute(self.db.as_ref())
        .await?;
        Ok(())
    }
}
