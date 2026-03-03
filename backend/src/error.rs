use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Vault not found: {0}")]
    VaultNotFound(String),

    #[error("Session expired")]
    SessionExpired,

    #[error("Exceeds approved limit")]
    ExceedsApprovedLimit,

    #[error("Unauthorized delegate")]
    UnauthorizedDelegate,

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Solana RPC error: {0}")]
    SolanaRpc(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::VaultNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::SessionExpired => (StatusCode::GONE, self.to_string()),
            AppError::ExceedsApprovedLimit => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::UnauthorizedDelegate => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::InvalidSignature(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::SolanaRpc(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;