pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod routes;
pub mod solana;
pub mod state;

pub use state::AppState;

use anyhow::Context;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub async fn build_server(state: AppState, host: &str, port: u16) -> anyhow::Result<()> {
    let app: Router = routes::router(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_methods(Any),
        )
        .layer(TraceLayer::new_for_http());

    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;

    tracing::info!("HTTP listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

