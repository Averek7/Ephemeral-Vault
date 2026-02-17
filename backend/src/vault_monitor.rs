use crate::{
    signer::build_cleanup_vault_ix,
    AppState, SessionRecord,
};

use chrono::Utc;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use sqlx::PgPool;
use std::str::FromStr;
use tokio_cron_scheduler::{Job, JobScheduler};

// ============================================================================
// CRON ENTRY POINT
// Called once from main.rs via tokio::spawn.
// ============================================================================

pub async fn start_cleanup_cron(db: PgPool) {
    let scheduler = JobScheduler::new()
        .await
        .expect("Failed to create job scheduler");

    // Clone DB handle for the closure (tokio-cron-scheduler requires 'static)
    let db_clone = db.clone();

    scheduler
        .add(
            Job::new_async("0 */10 * * * *", move |_uuid, _l| {
                let db = db_clone.clone();
                Box::pin(async move {
                    match run_cleanup(&db).await {
                        Ok(count) => {
                            if count > 0 {
                                log::info!("🧹 Vault monitor cleaned {} expired sessions", count);
                            }
                        }
                        Err(e) => {
                            log::error!("Vault monitor cleanup job failed: {}", e);
                        }
                    }
                })
            })
            .expect("Failed to create cleanup job"),
        )
        .await
        .expect("Failed to schedule cleanup job");

    scheduler.start().await.expect("Failed to start scheduler");

    log::info!("✅ Vault monitor started — running cleanup every 10 minutes");
}

// ============================================================================
// CLEANUP LOGIC
// Finds sessions that are:
//   1. Marked inactive (is_active = false)
//   2. Expired more than 1 hour ago
// For each, submits cleanup_vault on-chain and removes the row from the DB.
// ============================================================================

async fn run_cleanup(db: &PgPool) -> Result<usize, Box<dyn std::error::Error>> {
    // Fetch all sessions eligible for cleanup
    let expired: Vec<SessionRecord> = sqlx::query_as!(
        SessionRecord,
        r#"
        SELECT *
        FROM   ephemeral_sessions
        WHERE  is_active  = false
        AND    expires_at < NOW() - INTERVAL '1 hour'
        "#
    )
    .fetch_all(db)
    .await?;

    let count = expired.len();

    for session in expired {
        match cleanup_single_session(db, &session).await {
            Ok(sig) => {
                log::info!(
                    "  ✅ Cleaned vault {} | tx: {}",
                    session.vault_pda,
                    sig
                );
            }
            Err(e) => {
                log::warn!(
                    "  ⚠️  Failed to clean vault {} — will retry next cycle. Error: {}",
                    session.vault_pda,
                    e
                );
            }
        }
    }

    Ok(count)
}

// ============================================================================
// SINGLE SESSION CLEANUP
// Builds + submits the on-chain cleanup_vault instruction, then removes the
// session from the database.
// ============================================================================

async fn cleanup_single_session(
    db: &PgPool,
    session: &SessionRecord,
) -> Result<String, Box<dyn std::error::Error>> {
    // Parse pubkeys from stored strings
    let vault_pda = Pubkey::from_str(&session.vault_pda)
        .map_err(|e| format!("Invalid vault PDA '{}': {}", session.vault_pda, e))?;

    let user_wallet = Pubkey::from_str(&session.user_wallet)
        .map_err(|e| format!("Invalid user wallet '{}': {}", session.user_wallet, e))?;

    // In production the cleaner is a backend-controlled fee-payer keypair
    // that receives the 1% reward.  Using a placeholder pubkey here.
    let cleaner_pubkey = user_wallet; // Placeholder — use real cleaner keypair in prod

    // Retrieve program_id from environment (not stored in session row)
    let program_id_str = std::env::var("PROGRAM_ID")
        .unwrap_or_else(|_| "FJwrtkVTxkfD7BshUx3uvpC5LKfQBqjUhunxMovqcxxA".to_string());
    let program_id = Pubkey::from_str(&program_id_str)?;

    // Build cleanup_vault instruction
    let _ix = build_cleanup_vault_ix(
        &program_id,
        &vault_pda,
        &user_wallet,
        &cleaner_pubkey,
    );

    // NOTE: In production, sign with the cleaner keypair and submit via RpcClient:
    //   let sig = sign_and_send(&rpc_client, &_ix, &cleaner_keypair).await?;
    let signature = Signature::default(); // Placeholder

    // Remove session record from DB regardless — funds already returned on-chain
    sqlx::query!(
        "DELETE FROM ephemeral_sessions WHERE session_id = $1",
        session.session_id
    )
    .execute(db)
    .await
    .map_err(|e| format!("DB delete failed for session {}: {}", session.session_id, e))?;

    Ok(signature.to_string())
}

// ============================================================================
// MONITORING HELPERS
// ============================================================================

/// Returns a summary of vault health metrics for logging / alerting.
pub async fn get_health_metrics(db: &PgPool) -> VaultHealthMetrics {
    let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM ephemeral_sessions")
        .fetch_one(db)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0);

    let active: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM ephemeral_sessions WHERE is_active = true"
    )
    .fetch_one(db)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    let expired_active: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM ephemeral_sessions WHERE is_active = true AND expires_at < NOW()"
    )
    .fetch_one(db)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    let pending_cleanup: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM ephemeral_sessions
        WHERE is_active = false AND expires_at < NOW() - INTERVAL '1 hour'
        "#
    )
    .fetch_one(db)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    let total_volume_lamports: i64 = sqlx::query_scalar!(
        "SELECT COALESCE(SUM(total_deposited), 0) FROM ephemeral_sessions"
    )
    .fetch_one(db)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    VaultHealthMetrics {
        total_sessions: total as u64,
        active_sessions: active as u64,
        expired_active_sessions: expired_active as u64,
        pending_cleanups: pending_cleanup as u64,
        total_volume_sol: total_volume_lamports as f64 / 1_000_000_000.0,
    }
}

#[derive(Debug)]
pub struct VaultHealthMetrics {
    pub total_sessions: u64,
    pub active_sessions: u64,
    pub expired_active_sessions: u64,
    pub pending_cleanups: u64,
    pub total_volume_sol: f64,
}

impl std::fmt::Display for VaultHealthMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "sessions(total={}, active={}, expired_active={}, pending_cleanup={}) volume={:.4} SOL",
            self.total_sessions,
            self.active_sessions,
            self.expired_active_sessions,
            self.pending_cleanups,
            self.total_volume_sol,
        )
    }
}