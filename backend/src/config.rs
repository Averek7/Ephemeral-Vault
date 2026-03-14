use std::env;

#[derive(Clone)]
pub struct Config {
    pub rpc_url: String,
    pub database_url: String,
    pub program_id: String,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            rpc_url: env::var("RPC_URL").unwrap_or("https://api.devnet.solana.com".into()),
            program_id: env::var("PROGRAM_ID").expect("PROGRAM_ID must be set"),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            server_port: env::var("SERVER_PORT")
                .or_else(|_| env::var("PORT"))
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .unwrap_or(8080),
        }
    }
}
