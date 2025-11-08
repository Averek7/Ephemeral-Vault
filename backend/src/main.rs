use std::net::SocketAddr;
use std::sync::Arc;

use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{EnvFilter, fmt};

mod lib;
mod session_manager;
mod auto_deposit;
mod delegation;
mod vault_monitor;
mod signer;

use lib::api_router;
use session_manager::SessionService;
use ring::aead;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    // tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");
    let rpc_url = std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let sealing_key_b64 = std::env::var("SEALING_KEY_BASE64")
        .expect("SEALING_KEY_BASE64 must be set (base64 32 bytes)");

    let key_bytes = base64::decode(sealing_key_b64)?;
    if key_bytes.len() != 32 {
        panic!("SEALING_KEY_BASE64 must decode to 32 bytes for AES-256-GCM");
    }
    let unbound = aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
        .expect("invalid sealing key");
    let sealing_key = aead::LessSafeKey::new(unbound);

    // DB pool
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let session_service = SessionService::new(Arc::new(db), sealing_key);

    // background monitor
    let monitor_db = session_service.db.clone();
    let monitor = vault_monitor::VaultMonitor::new(monitor_db, 60);

    // spawn monitor
    let monitor_task = {
        let monitor = std::sync::Arc::new(monitor);
        let cleanup_cb = move |session_id: uuid::Uuid| {
            // For simplicity, cleanup callback logs only.
            let db = session_service.db.clone();
            async move {
                tracing::info!("Cleanup requested for session {}", session_id);
                // Implement on-chain or DB-level cleanup here as needed.
                Ok::<(), anyhow::Error>(())
            }
        };
        let m = monitor.clone();
        tokio::spawn(async move {
            if let Err(e) = m.start(cleanup_cb).await {
                tracing::error!("Vault monitor failed: {:?}", e);
            }
        })
    };

    // run server
    let app = api_router(session_service);
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;

    // keep monitor alive
    monitor_task.await.ok();

    Ok(())
}
