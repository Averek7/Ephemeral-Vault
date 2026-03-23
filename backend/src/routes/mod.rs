use axum::{routing::{get, post}, Router};

use crate::{handlers, state::AppState};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/vault/{user_pubkey}", get(handlers::get_vault))
        .route("/vault_stats/{user_pubkey}", get(handlers::get_vault_stats))
        .route("/trades/{vault_pubkey}", get(handlers::get_trades))
        .route("/trades", post(handlers::create_trade))
        .route("/tx/create_vault", post(handlers::tx_create_vault))
        .route("/tx/deposit", post(handlers::tx_deposit))
        .route("/tx/withdraw", post(handlers::tx_withdraw))
        .route("/tx/pause", post(handlers::tx_pause))
        .route("/tx/unpause", post(handlers::tx_unpause))
        .route("/tx/revoke", post(handlers::tx_revoke))
        .route("/tx/renew_session", post(handlers::tx_renew_session))
        .route("/tx/approve_delegate", post(handlers::tx_approve_delegate))
        .route("/tx/reactivate", post(handlers::tx_reactivate))
        .route(
            "/tx/update_approved_amount",
            post(handlers::tx_update_approved_amount),
        )
        .with_state(state)
}
