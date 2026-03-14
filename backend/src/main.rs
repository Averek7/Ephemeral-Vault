use backend::{build_server, AppState};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env
    dotenv::dotenv().ok();

    // Initialise tracing logger
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // ── Environment variables ──────────────────────────────────────────────
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

    let program_id_str =
        std::env::var("PROGRAM_ID").expect("PROGRAM_ID must be set");

    let host =
        std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let port: u16 = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("SERVER_PORT must be a valid u16");

    // ── Database ───────────────────────────────────────────────────────────
    let db = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run database migrations");

    tracing::info!("Database connected and migrations applied");

    // ── Solana RPC ─────────────────────────────────────────────────────────
    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        rpc_url.clone(),
        CommitmentConfig::confirmed(),
    ));

    let program_id = Pubkey::from_str(&program_id_str)
        .expect("PROGRAM_ID is not a valid Solana pubkey");

    tracing::info!("Solana RPC connected: {}", rpc_url);
    tracing::info!("Program ID: {}", program_id);

    // ── Shared application state ───────────────────────────────────────────
    let state = AppState {
        db: db.clone(),
        rpc_client,
        program_id,
    };

    // ── Background vault monitor job ───────────────────────────────────────
    tokio::spawn(backend::vault_monitor::start_cleanup_cron(
        db.clone(),
    ));

    // ── HTTP server ────────────────────────────────────────────────────────
    build_server(state, &host, port).await?;

    Ok(())
}