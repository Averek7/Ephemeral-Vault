use axum::{routing::{get, post}, Router};

use crate::{handlers, state::AppState};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/vault/{user_pubkey}", get(handlers::get_vault))
        .route("/vault_stats/{user_pubkey}", get(handlers::get_vault_stats))
        .route("/trades/{vault_pubkey}", get(handlers::get_trades))
        .route("/trades", post(handlers::create_trade))
        .with_state(state)
}
