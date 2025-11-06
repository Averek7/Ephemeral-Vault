use backend::{AppState, build_router};
use std::net::SocketAddr;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // Load environment
    let db_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await?;

    // App shared state
    let state = AppState::new(pool)?;

    // Setup Axum app
    let app = build_router(state);

    // Start server
    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    println!("🚀 Ephemeral Backend running at {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
