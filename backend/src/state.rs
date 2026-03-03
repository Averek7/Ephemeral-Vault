use std::sync::Arc;
use sqlx::PgPool;
use redis::aio::ConnectionManager;
use solana_client::nonblocking::rpc_client::RpcClient;
use crate::config::Config;
use crate::websocket::hub::BroadcastHub;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub rpc: Arc<RpcClient>,
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub hub: Arc<BroadcastHub>,
}