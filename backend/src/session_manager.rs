use std::sync::Arc;
use chrono::{Utc, Duration};
use sqlx::{Pool, Postgres};
use ring::aead::{LessSafeKey, UnboundKey, AES_256_GCM, Nonce};
use solana_sdk::signature::{Keypair, Signer};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use rand::rngs::OsRng;
use rand::RngCore;

#[derive(Debug, Clone)]
pub struct SessionManager {
    pub db: Pool<Postgres>,
    pub sealing_key: Arc<LessSafeKey>,
}

impl SessionManager {
    /// Create new ephemeral trading session
    pub async fn create_session(
        &self,
        user_id: String,
        session_lifetime_minutes: i64,
    ) -> Result<Uuid> {
        // Generate ephemeral wallet
        let keypair = Keypair::generate(&mut OsRng);
        let pubkey = keypair.pubkey().to_string();
        let private_key_bytes = keypair.to_bytes();

        // Encrypt private key before storing in DB
        let encrypted_key = self.encrypt(&private_key_bytes)?;

        let session_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::minutes(session_lifetime_minutes);

        sqlx::query!(
            r#"INSERT INTO sessions (id, user_id, ephemeral_pubkey, encrypted_key, expires_at)
               VALUES ($1, $2, $3, $4, $5)"#,
            session_id,
            user_id,
            pubkey,
            encrypted_key,
            expires_at
        )
        .execute(&self.db)
        .await?;

        Ok(session_id)
    }

    /// Load decrypted ephemeral wallet from database
    pub async fn get_session_keypair(&self, session_id: Uuid) -> Result<Keypair> {
        let row = sqlx::query!(
            r#"SELECT encrypted_key
               FROM sessions
               WHERE id = $1 AND expires_at > NOW()"#,
            session_id
        )
        .fetch_one(&self.db)
        .await?;

        let decrypted = self.decrypt(&row.encrypted_key)?;

        let keypair = Keypair::from_bytes(&decrypted)
            .map_err(|_| anyhow!("Failed to deserialize keypair"))?;

        Ok(keypair)
    }

    /// Encrypt private key
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        let mut buffer = data.to_vec();
        buffer.extend_from_slice(&[0u8; 16]); // space for tag

        self.sealing_key
            .seal_in_place_append_tag(nonce, ring::aead::Aad::empty(), &mut buffer)
            .map_err(|_| anyhow!("Encryption failed"))?;

        // prepend nonce so decrypt knows how to reverse it
        let mut out = nonce_bytes.to_vec();
        out.extend_from_slice(&buffer);

        Ok(out)
    }

    /// Decrypt private key
    fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        if encrypted.len() < 12 {
            return Err(anyhow!("Invalid encrypted blob"));
        }

        let (nonce_bytes, ciphertext) = encrypted.split_at(12);
        let nonce = Nonce::assume_unique_for_key(nonce_bytes.try_into().unwrap());

        let mut buffer = ciphertext.to_vec();

        let decrypted_data = self
            .sealing_key
            .open_in_place(nonce, ring::aead::Aad::empty(), &mut buffer)
            .map_err(|_| anyhow!("Decryption failed"))?;

        Ok(decrypted_data.to_vec())
    }
}
