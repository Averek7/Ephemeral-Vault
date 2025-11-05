mod db;
mod session_manager;

use axum::{routing::post, routing::get, Router, Json};
use crate::session_manager::SessionManager;
use crate::db::{Database, EphemeralSession};
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use dotenv::dotenv;
use serde::Deserialize;
use uuid::Uuid;

// Temporary: using in-memory DB until a real Postgres or Redis is plugged in
use crate::db::MemoryDB;  // <-- This comes from `session_manager.rs` sample

#[derive(Deserialize)]
struct StartSessionRequest {
    user_wallet: String,
    approved_amount: u64,
}

#[derive(Deserialize)]
struct EndSessionRequest {
    session_id: Uuid,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok(); // load env vars

    println!("🚀 Ephemeral Vault Backend Starting...");

    // Create a DB instance (could be PostgreSQL, Redis, SQLite, etc.)
    let db = Arc::new(Mutex::new(MemoryDB::default()));

    // Inject DB into session manager
    let session_manager = Arc::new(SessionManager::new(db.clone()));

    // API routes
    let app = Router::new()
        .route("/session/start", post(start_session))
        .route("/session/end", post(end_session))
        .route("/session/status", get(session_status))
        .with_state(session_manager);

    let port = 8080;
    let addr = format!("0.0.0.0:{port}");
    println!("✅ Backend listening on http://{}", addr);

    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// POST /session/start
async fn start_session(
    axum::extract::State(sm): axum::extract::State<Arc<SessionManager>>,
    Json(req): Json<StartSessionRequest>,
) -> Result<Json<EphemeralSession>, String> {
    sm.start_session(req.user_wallet, req.approved_amount)
        .await
        .map(Json)
        .map_err(|e| e.to_string())
}

/// POST /session/end
async fn end_session(
    axum::extract::State(sm): axum::extract::State<Arc<SessionManager>>,
    Json(req): Json<EndSessionRequest>,
) -> Result<Json<String>, String> {
    sm.end_session(req.session_id)
        .await
        .map(|_| Json("Session ended".to_string()))
        .map_err(|e| e.to_string())
}

/// GET /session/status?wallet=<pubkey>
async fn session_status(
    axum::extract::State(sm): axum::extract::State<Arc<SessionManager>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Option<EphemeralSession>>, String> {
    let Some(wallet) = params.get("wallet") else {
        return Err("wallet query param required".into());
    };

    sm.get_session(wallet)
        .await
        .map(Json)
        .map_err(|e| e.to_string())
}
