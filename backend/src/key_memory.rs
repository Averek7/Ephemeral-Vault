use dashmap::DashMap;
use solana_sdk::signature::Keypair;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct EphemeralKeyMemory {
    /// Stores ephemeral keypairs only in memory. Not persisted!
    pub inner: Arc<DashMap<Uuid, Keypair>>,
}

impl EphemeralKeyMemory {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new())
        }
    }

    pub fn insert(&self, session_id: Uuid, keypair: Keypair) {
        self.inner.insert(session_id, keypair);
    }

    pub fn get(&self, session_id: &Uuid) -> Option<Keypair> {
        self.inner.get(session_id).map(|v| v.clone())
    }

    pub fn remove(&self, session_id: &Uuid) {
        self.inner.remove(session_id);
    }
}
