use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents one ephemeral trading session (mapped 1:1 with a vault PDA)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EphemeralSession {
    pub session_id: Uuid,
    pub user_wallet: String,      // Parent wallet pubkey (base58)
    pub ephemeral_wallet: String, // Generated ephemeral keypair pubkey
    pub vault_pda: String,        // Anchor PDA created on-chain
    pub approved_amount: u64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub active: bool,
}

/// Represents updates from vault-monitor (fiat/equity tracking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultState {
    pub vault_pda: String,
    pub balance_sol: u64,
    pub balance_usdt: u64,
    pub last_activity: DateTime<Utc>,
}

/// Activity logs (auto-deposits, trade executions, revokes, cleanup)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: Uuid,
    pub session_id: Uuid,
    pub event: String,
    pub timestamp: DateTime<Utc>,
}

/// Database Interface (pluggable)
#[async_trait]
pub trait Database: Send + Sync {
    async fn save_session(&self, session: EphemeralSession) -> Result<()>;
    async fn get_active_session(&self, user_wallet: &str) -> Result<Option<EphemeralSession>>;
    async fn end_session(&self, session_id: Uuid) -> Result<()>;

    async fn update_vault_state(&self, vault: VaultState) -> Result<()>;
    async fn get_vault_state(&self, vault_pda: &str) -> Result<Option<VaultState>>;

    async fn log_event(&self, log: ActivityLog) -> Result<()>;
}

/// Helper constructors
impl EphemeralSession {
    pub fn new(
        user_wallet: String,
        ephemeral_wallet: String,
        vault_pda: String,
        approved_amount: u64,
        session_duration_secs: u64,
    ) -> Self {
        let now = Utc::now();

        Self {
            session_id: Uuid::new_v4(),
            user_wallet,
            ephemeral_wallet,
            vault_pda,
            approved_amount,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(session_duration_secs as i64),
            active: true,
        }
    }
}
