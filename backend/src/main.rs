use ephemeral_vault_backend::{build_server, AppState};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env file
    dotenv::dotenv().ok();

    // Initialise logger
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // ── Environment variables ──────────────────────────────────────────────
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

    let program_id_str = std::env::var("PROGRAM_ID")
        .expect("PROGRAM_ID must be set");

    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
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

    log::info!("✅ Database connected and migrations applied");

    // ── Solana RPC ─────────────────────────────────────────────────────────
    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        rpc_url.clone(),
        CommitmentConfig::confirmed(),
    ));

    let program_id = Pubkey::from_str(&program_id_str)
        .expect("PROGRAM_ID is not a valid Solana pubkey");

    log::info!("✅ Solana RPC connected: {}", rpc_url);
    log::info!("✅ Program ID: {}", program_id);

    // ── Shared application state ───────────────────────────────────────────
    let state = AppState {
        db: db.clone(),
        rpc_client,
        program_id,
    };

    // ── Background vault-monitor cron job ──────────────────────────────────
    tokio::spawn(ephemeral_vault_backend::vault_monitor::start_cleanup_cron(
        db.clone(),
    ));

    // ── HTTP server ────────────────────────────────────────────────────────
    build_server(state, &host, port).await
}