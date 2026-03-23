use solana_client::nonblocking::rpc_client::RpcClient;
use sqlx::PgPool;
use std::sync::Arc;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
    pub rpc: Arc<RpcClient>,
}
