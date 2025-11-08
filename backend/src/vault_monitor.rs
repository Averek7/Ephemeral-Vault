use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use chrono::Utc;
use uuid::Uuid;
use crate::session_manager::EphemeralSession;

/// VaultMonitor scans DB for expired sessions and triggers a cleanup callback.
pub struct VaultMonitor {
    pub db: Arc<PgPool>,
    pub check_interval_seconds: u64,
}

impl VaultMonitor {
    pub fn new(db: Arc<PgPool>, check_interval_seconds: u64) -> Self {
        Self { db, check_interval_seconds }
    }

    /// Start background monitor. `cleanup_callback` is called with the session id for each expired session.
    pub async fn start<F, Fut>(self: Arc<Self>, mut cleanup_callback: F) -> Result<()>
    where
        F: FnMut(Uuid) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let mut ticker = interval(Duration::from_secs(self.check_interval_seconds));
        loop {
            ticker.tick().await;
            tracing::info!("VaultMonitor tick: scanning for expired sessions");
            let expired = sqlx::query_as!(
                EphemeralSession,
                r#"
                SELECT id, user_id as "user!", ephemeral_pubkey as "ephemeral_wallet!", encrypted_key, vault_pda, expires_at, approved_amount, total_deposited, delegate_pubkey, delegate_approved
                FROM sessions
                WHERE expires_at < $1
                "#,
                Utc::now()
            )
            .fetch_all(self.db.as_ref())
            .await?;

            for s in expired {
                tracing::info!("Found expired session id={}", s.id);
                if let Err(e) = cleanup_callback(s.id).await {
                    tracing::error!("cleanup callback failed for {}: {:?}", s.id, e);
                }
            }
        }
    }
}
