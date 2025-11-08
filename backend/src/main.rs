// ==================== main.rs ====================
use backend::{build_router, init_state};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use axum::Server;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let rpc_url = std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".into());

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .unwrap();

    // AES key stored in env: SEALING_KEY=32bytehex...
    let sealing_key = hex::decode(std::env::var("SEALING_KEY").unwrap())
        .unwrap()
        .try_into()
        .unwrap();

    let state = Arc::new(init_state(db, &rpc_url, &sealing_key).await.unwrap());
    let router = build_router(state);

    println!("🚀 Ephemeral Wallet Backend Running on http://127.0.0.1:8000");

    Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}
