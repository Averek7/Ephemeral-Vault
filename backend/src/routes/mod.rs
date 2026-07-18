use axum::{
    routing::{get, post},
    Router,
};

use crate::{handlers, state::AppState};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/ready", get(handlers::ready))
        .merge(api_routes())
        .nest("/api/v1", api_routes())
        .with_state(state)
}

fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/vault/:user_pubkey", get(handlers::get_vault))
        .route("/vault_stats/:user_pubkey", get(handlers::get_vault_stats))
        .route("/trades/:vault_pubkey", get(handlers::get_trades))
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
        .route("/tx/execute_trade", post(handlers::tx_execute_trade))
        .route("/tx/cleanup", post(handlers::tx_cleanup))
        .route("/tx/simulate", post(handlers::tx_simulate))
        .route("/tx/status/:signature", get(handlers::tx_status))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_sdk::commitment_config::CommitmentConfig;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn test_state() -> AppState {
        AppState {
            config: Config {
                rpc_url: "http://127.0.0.1:8899".into(),
                database_url: "postgres://postgres:postgres@localhost:5432/ephemeral_vault".into(),
                program_id: "3L2LMJHHvgaGnvQ2ic7a5yu6DffLfoAQFLwFSjFJ4QQt".into(),
                server_host: "127.0.0.1".into(),
                server_port: 8080,
            },
            db: PgPoolOptions::new()
                .connect_lazy("postgres://postgres:postgres@localhost:5432/ephemeral_vault")
                .expect("lazy postgres pool"),
            rpc: Arc::new(RpcClient::new_with_commitment(
                "http://127.0.0.1:8899".into(),
                CommitmentConfig::confirmed(),
            )),
        }
    }

    #[tokio::test]
    async fn health_route_returns_ok() {
        let response = router(test_state())
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        assert_eq!(&body[..], br#"{"status":"ok"}"#);
    }

    #[tokio::test]
    async fn versioned_trade_route_is_registered() {
        let response = router(test_state())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/trades/TEST")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_ne!(response.status(), StatusCode::NOT_FOUND);
    }
}
