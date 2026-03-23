use std::sync::Arc;

use ephemeral_vault_backend::{build_server, config::Config, AppState};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .init();

    let config = Config::from_env();

    let db = PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run database migrations");

    tracing::info!("Database connected and migrations applied");

    let rpc = Arc::new(RpcClient::new_with_commitment(
        config.rpc_url.clone(),
        CommitmentConfig::confirmed(),
    ));

    tracing::info!("Solana RPC configured: {}", config.rpc_url);
    tracing::info!("Program ID configured: {}", config.program_id);

    let state = AppState {
        config: config.clone(),
        db,
        rpc,
    };

    build_server(state, &config.server_host, config.server_port).await
}
