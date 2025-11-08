use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use ring::aead;
use std::time::Duration;
use tokio::time::sleep;

/// TransactionSigner: decrypts ephemeral keypair and signs/sends transaction with retries.
pub struct TransactionSigner {
    pub rpc: RpcClient,
    pub sealing_key: aead::LessSafeKey,
}

impl TransactionSigner {
    pub fn new(rpc_url: &str, sealing_key: aead::LessSafeKey) -> Self {
        Self { rpc: RpcClient::new(rpc_url.to_string()), sealing_key }
    }

    /// Decrypt blob (nonce || ciphertext+tag) and build Solana Keypair
    pub fn decrypt_keypair(&self, encrypted_blob: &[u8]) -> Result<Keypair> {
        if encrypted_blob.len() < 12 + aead::MAX_TAG_LEN {
            return Err(anyhow::anyhow!("invalid encrypted blob"));
        }
        let (nonce_bytes, ciphertext) = encrypted_blob.split_at(12);
        let nonce = aead::Nonce::try_assume_unique_for_key(nonce_bytes)?;
        let mut in_out = ciphertext.to_vec();
        let aad = aead::Aad::empty();
        let decrypted = self.sealing_key.open_in_place(nonce, aad, &mut in_out)
            .map_err(|e| anyhow::anyhow!("open failed: {:?}", e))?;
        let kp = Keypair::from_bytes(decrypted)
            .map_err(|e| anyhow::anyhow!("keypair decode failed: {:?}", e))?;
        Ok(kp)
    }

    /// Sign & send transaction with retries
    pub async fn sign_and_send(&self, mut tx: Transaction, encrypted_key: &[u8]) -> Result<solana_sdk::signature::Signature> {
        let kp = self.decrypt_keypair(encrypted_key)?;
        tx.sign(&[&kp], tx.message.recent_blockhash);
        let mut attempt = 0u32;
        let mut delay = Duration::from_millis(200);
        loop {
            attempt += 1;
            match self.rpc.send_and_confirm_transaction(&tx) {
                Ok(sig) => return Ok(sig),
                Err(e) => {
                    tracing::warn!("send_and_confirm attempt {} failed: {:?}", attempt, e);
                    if attempt >= 5 {
                        return Err(anyhow::anyhow!("send failed after retries: {:?}", e));
                    }
                    sleep(delay).await;
                    delay *= 2;
                }
            }
        }
    }
}
