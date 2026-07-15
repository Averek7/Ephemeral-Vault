use std::env;

use anyhow::{anyhow, Context, Result};
use solana_sdk::pubkey::Pubkey;

#[derive(Clone)]
pub struct Config {
    pub rpc_url: String,
    pub database_url: String,
    pub program_id: String,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let config = Self {
            rpc_url: env::var("RPC_URL").unwrap_or("https://api.devnet.solana.com".into()),
            program_id: required_env("PROGRAM_ID")?,
            database_url: required_env("DATABASE_URL")?,
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            server_port: parse_server_port(
                env::var("SERVER_PORT")
                    .or_else(|_| env::var("PORT"))
                    .unwrap_or_else(|_| "8080".into()),
            )?,
        };

        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.rpc_url.trim().is_empty() {
            return Err(anyhow!("RPC_URL must not be empty"));
        }

        if !self.rpc_url.starts_with("http://") && !self.rpc_url.starts_with("https://") {
            return Err(anyhow!("RPC_URL must start with http:// or https://"));
        }

        self.program_id
            .parse::<Pubkey>()
            .context("PROGRAM_ID must be a valid Solana pubkey")?;

        if !self.database_url.starts_with("postgres://")
            && !self.database_url.starts_with("postgresql://")
        {
            return Err(anyhow!(
                "DATABASE_URL must start with postgres:// or postgresql://"
            ));
        }

        if self.server_host.trim().is_empty() {
            return Err(anyhow!("SERVER_HOST must not be empty"));
        }

        Ok(())
    }
}

fn required_env(key: &str) -> Result<String> {
    let value = env::var(key).with_context(|| format!("{key} must be set"))?;
    if value.trim().is_empty() {
        return Err(anyhow!("{key} must not be empty"));
    }
    Ok(value)
}

fn parse_server_port(raw: String) -> Result<u16> {
    raw.parse::<u16>()
        .with_context(|| format!("SERVER_PORT/PORT must be a valid u16, got {raw:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> Config {
        Config {
            rpc_url: "https://api.devnet.solana.com".into(),
            database_url: "postgres://postgres:postgres@localhost:5432/ephemeral_vault".into(),
            program_id: "3L2LMJHHvgaGnvQ2ic7a5yu6DffLfoAQFLwFSjFJ4QQt".into(),
            server_host: "127.0.0.1".into(),
            server_port: 8080,
        }
    }

    #[test]
    fn validates_expected_local_config() {
        assert!(valid_config().validate().is_ok());
    }

    #[test]
    fn rejects_invalid_program_id() {
        let mut config = valid_config();
        config.program_id = "not-a-pubkey".into();

        assert!(config.validate().is_err());
    }

    #[test]
    fn rejects_non_postgres_database_url() {
        let mut config = valid_config();
        config.database_url = "mysql://localhost/app".into();

        assert!(config.validate().is_err());
    }

    #[test]
    fn rejects_invalid_server_port() {
        assert!(parse_server_port("not-a-port".into()).is_err());
    }
}
