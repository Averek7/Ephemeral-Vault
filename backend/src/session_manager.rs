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
    // symmetric key for encrypting ephemeral private keys (from env)
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
}


impl SessionService {
    pub async fn new(db: Arc<PgPool>, sealing_key: aead::LessSafeKey) -> Self {
        Self { db, sealing_key }
    }


    pub async fn create_session(&self, user: &str) -> Result<EphemeralSession> {
        // Generate ephemeral Solana keypair
        let kp = Keypair::generate(&mut OsRng);
        let pubkey = kp.pubkey().to_string();


        // Encrypt private key bytes
        let sk_bytes = kp.to_bytes();
        let nonce = aead::Nonce::assume_unique_for_key([0u8; 12]); // in prod use random nonce
        let mut in_out = sk_bytes.to_vec();
        in_out.extend_from_slice(&[0u8; aead::MAX_TAG_LEN]);
        let aad = aead::Aad::empty();
        self.sealing_key.seal_in_place_append_tag(nonce, aad, &mut in_out)
        .map_err(|e| anyhow::anyhow!("seal failed: {:?}", e))?;


        // Prepare DB insert
        let id = Uuid::new_v4();
        let vault_pda = format!("vault_{}", id);
        let expires_at = Utc::now() + chrono::Duration::minutes(15);


        sqlx::query!(
            r#"INSERT INTO sessions (id, user_id, ephemeral_pubkey, encrypted_key, vault_pda, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)"#,
            id,
            user,
            pubkey,
            in_out,
            vault_pda,
            expires_at
        )
        .execute(self.db.as_ref())
        .await?;


        Ok(EphemeralSession { id, user: user.to_string(), ephemeral_wallet: pubkey, encrypted_key: in_out, vault_pda, expires_at })
    }


    pub async fn get_session(&self, id: Uuid) -> Result<Option<EphemeralSession>> {
        let rec = sqlx::query_as!(EphemeralSession, r#"SELECT id, user_id as "user: _, ephemeral_pubkey as "ephemeral_wallet: _, encrypted_key, vault_pda, expires_at FROM sessions WHERE id = $1"#, id)
        .fetch_optional(self.db.as_ref())
        .await?;
        Ok(rec)
    }
}