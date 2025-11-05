use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::Result;
use rand::RngCore;

const NONCE_LENGTH: usize = 12;

pub fn encrypt_bytes(secret: &[u8], data: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(secret);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let encrypted = cipher.encrypt(nonce, data)?;
    Ok((nonce_bytes.to_vec(), encrypted))
}

pub fn decrypt_bytes(secret: &[u8], nonce: &[u8], encrypted: &[u8]) -> Result<Vec<u8>> {
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(secret);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);

    let decrypted = cipher.decrypt(nonce, encrypted)?;
    Ok(decrypted)
}
