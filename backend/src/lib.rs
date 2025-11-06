pub mod session_manager;
pub mod auto_deposit;
pub mod delegation;
pub mod signer;
pub mod vault_monitor;

use sqlx::PgPool;
use axum::{Router, routing::post, routing::get};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

impl AppState {
    pub fn new(db: PgPool) -> anyhow::Result<Self> {
        Ok(Self { db })
    }
}

// REST endpoint router (reused by main.rs)
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/session/create", post(session_manager::create))
        .route("/session/status", get(session_manager::status))
        .route("/session/approve", post(delegation::approve))
        .route("/session/revoke", post(session_manager::revoke))
        .route("/session/deposit", post(auto_deposit::trigger))
        .with_state(state)
}
