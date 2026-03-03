use std::env;

#[derive(Clone)]
pub struct Config {
    pub rpc_url: String,
    pub ws_url: String,
    pub program_id: String,
    pub database_url: String,
    pub redis_url: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            rpc_url: env::var("RPC_URL").unwrap_or("https://api.devnet.solana.com".into()),
            ws_url: env::var("WS_URL").unwrap_or("wss://api.devnet.solana.com".into()),
            program_id: env::var("PROGRAM_ID").expect("PROGRAM_ID must be set"),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            redis_url: env::var("REDIS_URL").unwrap_or("redis://127.0.0.1:6379".into()),
            server_port: env::var("PORT")
                .unwrap_or("8080".into())
                .parse()
                .unwrap_or(8080),
        }
    }
}
