pub mod session_manager;
pub mod auto_deposit;
pub mod delegation;
pub mod vault_monitor;
pub mod signer;

use axum::{
    extract::{State, Json, Path},
    routing::{post, get, delete},
    Router, response::IntoResponse, http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::session_manager::{SessionService, EphemeralSessionCreateRequest, EphemeralSessionCreateResponse};

pub fn api_router(session_service: SessionService) -> Router {
    let state = Arc::new(session_service);
    Router::new()
        .route("/session/create", post(session_create_handler))
        .route("/session/approve", post(session_approve_handler))
        .route("/session/deposit", post(session_deposit_handler))
        .route("/session/status/:id", get(session_status_handler))
        .route("/session/revoke/:id", delete(session_revoke_handler))
        .with_state(state)
}

async fn session_create_handler(
    State(session_svc): State<SessionService>,
    Json(req): Json<EphemeralSessionCreateRequest>,
) -> impl IntoResponse {
    match session_svc.create_session(&req.user).await {
        Ok(sess) => (StatusCode::CREATED, Json(EphemeralSessionCreateResponse::from(sess))).into_response(),
        Err(e) => {
            tracing::error!("create session failed: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": format!("{}", e)}))).into_response()
        }
    }
}

async fn session_approve_handler() -> impl IntoResponse {
    // For simplicity, activation of delegation is done via /session/approve
    (StatusCode::NOT_IMPLEMENTED, Json(serde_json::json!({"status":"not implemented"})))
}

async fn session_deposit_handler() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, Json(serde_json::json!({"status":"not implemented"})))
}

async fn session_status_handler(
    State(session_svc): State<SessionService>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match uuid::Uuid::parse_str(&id) {
        Ok(uid) => match session_svc.get_session(uid).await {
            Ok(Some(s)) => (StatusCode::OK, Json(serde_json::to_value(s).unwrap())).into_response(),
            Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"not found"}))).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": format!("{}", e)}))).into_response()
        },
        Err(_) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"invalid id"}))).into_response()
    }
}

async fn session_revoke_handler(
    Path(id): Path<String>,
) -> impl IntoResponse {
    // implement as needed - for now not implemented
    (StatusCode::NOT_IMPLEMENTED, Json(serde_json::json!({"status":"not implemented"})))
}