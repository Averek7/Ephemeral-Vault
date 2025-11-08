use anyhow::{Result, anyhow};
use chrono::Utc;
use sqlx::{Pool, Postgres};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

#[derive(Clone)]
pub struct VaultMonitor {
    pub db: Pool<Postgres>,
    pub rpc: RpcClient,
}

impl VaultMonitor {
    /// Get all active (non-expired) vaults
    pub async fn list_active_vaults(&self) -> Result<Vec<(uuid::Uuid, Pubkey)>> {
        let rows = sqlx::query!(
            "SELECT id, ephemeral_pubkey FROM sessions WHERE expires_at > NOW()"
        )
        .fetch_all(&self.db)
        .await?;

        let result = rows
            .into_iter()
            .map(|r| (r.id, r.ephemeral_pubkey.parse().unwrap()))
            .collect();

        Ok(result)
    }

    /// Detect expired vault sessions
    pub async fn detect_expired_sessions(&self) -> Result<Vec<uuid::Uuid>> {
        let expired = sqlx::query!(
            "SELECT id FROM sessions WHERE expires_at < NOW()"
        )
        .fetch_all(&self.db)
        .await?;

        Ok(expired.into_iter().map(|r| r.id).collect())
    }

    /// Monitor balance for a single vault
    pub async fn monitor_balance(&self, vault: &Pubkey) -> Result<f64> {
        let lamports = self.rpc.get_balance(vault).await?;
        Ok(lamports as f64 / 1e9)
    }

    /// Cleanup abandoned vaults
    pub async fn cleanup(&self, session_id: uuid::Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM sessions WHERE id = $1", session_id)
            .execute(&self.db)
            .await?;

        println!("🧹 Cleaned expired vault session: {:?}", session_id);
        Ok(())
    }
}