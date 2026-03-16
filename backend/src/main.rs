use backend::config::Config;
use backend::{build_server, AppState};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use sqlx::PgPool;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env
    dotenv::dotenv().ok();

    // Initialise tracing logger
    tracing_subscriber::fmt().with_env_filter("info").init();

    let config = Config::from_env();

    // ── Database ───────────────────────────────────────────────────────────
    let db = PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run database migrations");

    tracing::info!("Database connected and migrations applied");

    // ── Solana RPC ─────────────────────────────────────────────────────────
    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        config.rpc_url.clone(),
        CommitmentConfig::confirmed(),
    ));

    let program_id = config
        .program_id
        .parse::<Pubkey>()
        .expect("PROGRAM_ID is not a valid Solana pubkey");

    tracing::info!("Solana RPC connected: {}", config.rpc_url);
    tracing::info!("Program ID: {}", program_id);

    // ── Shared application state ───────────────────────────────────────────
    let state = AppState {
        db: db.clone(),
        rpc: rpc_client,
        program_id,
    };

    // ── HTTP server ────────────────────────────────────────────────────────
    build_server(state, &config.server_host, config.server_port).await?;

    Ok(())
}
