use anchor_lang::AnchorDeserialize;
use serde::Serialize;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::config::Config;
use crate::error::{AppError, Result};

const LAMPORTS_PER_SOL: f64 = 1_000_000_000.0;

#[derive(Clone, Debug, AnchorDeserialize)]
pub struct EphemeralVaultAccount {
    pub user_wallet: Pubkey,
    pub vault_pda: Pubkey,
    pub created_at: i64,
    pub last_activity: i64,
    pub approved_amount: u64,
    pub used_amount: u64,
    pub available_amount: u64,
    pub delegate_wallet: Option<Pubkey>,
    pub delegated_at: Option<i64>,
    pub session_expires_at: Option<i64>,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub trade_count: u64,
    pub is_active: bool,
    pub is_paused: bool,
    pub version: u8,
    pub bump: u8,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultDto {
    pub address: String,
    pub owner: String,
    pub delegate: Option<String>,
    pub approved_amount_lamports: u64,
    pub available_amount_lamports: u64,
    pub used_amount_lamports: u64,
    pub total_deposited_lamports: u64,
    pub total_withdrawn_lamports: u64,
    pub approved_amount_sol: f64,
    pub available_amount_sol: f64,
    pub used_amount_sol: f64,
    pub total_deposited_sol: f64,
    pub total_withdrawn_sol: f64,
    pub trade_count: u64,
    pub session_expiry: Option<i64>,
    pub delegated_at: Option<i64>,
    pub created_at: i64,
    pub last_activity: i64,
    pub is_active: bool,
    pub is_paused: bool,
    pub session_status: SessionStatusDto,
    pub status: VaultStatusDto,
    pub version: u8,
    pub bump: u8,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatusDto {
    NoSession,
    Active,
    ExpiringSoon,
    Expired,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VaultStatusDto {
    Active,
    Paused,
    Inactive,
    Expired,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultStatsDto {
    pub approved_amount_lamports: u64,
    pub available_amount_lamports: u64,
    pub used_amount_lamports: u64,
    pub total_deposited_lamports: u64,
    pub total_withdrawn_lamports: u64,
    pub trade_count: u64,
    pub is_active: bool,
    pub is_paused: bool,
    pub session_expiry: Option<i64>,
    pub session_status: SessionStatusDto,
    pub status: VaultStatusDto,
}

fn to_sol(lamports: u64) -> f64 {
    lamports as f64 / LAMPORTS_PER_SOL
}

pub fn derive_vault_pda(program_id: &Pubkey, user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault", user.as_ref()], program_id)
}

fn parse_vault_account(data: &[u8]) -> Result<EphemeralVaultAccount> {
    if data.len() < 8 {
        return Err(AppError::Internal("vault account is too small".into()));
    }

    let mut bytes = &data[8..];
    EphemeralVaultAccount::deserialize(&mut bytes)
        .map_err(|e| AppError::SerializationMessage(format!("failed to decode vault account: {e}")))
}

fn session_status(vault: &EphemeralVaultAccount, now_ts: i64) -> SessionStatusDto {
    match vault.session_expires_at {
        None => SessionStatusDto::NoSession,
        Some(expires_at) if now_ts >= expires_at => SessionStatusDto::Expired,
        Some(expires_at) if expires_at - now_ts <= 300 => SessionStatusDto::ExpiringSoon,
        Some(_) => SessionStatusDto::Active,
    }
}

fn vault_status(vault: &EphemeralVaultAccount, session_status: SessionStatusDto) -> VaultStatusDto {
    if !vault.is_active {
        VaultStatusDto::Inactive
    } else if vault.is_paused {
        VaultStatusDto::Paused
    } else if matches!(session_status, SessionStatusDto::Expired) {
        VaultStatusDto::Expired
    } else {
        VaultStatusDto::Active
    }
}

pub fn to_vault_dto(vault_pubkey: Pubkey, vault: EphemeralVaultAccount, now_ts: i64) -> VaultDto {
    let session_status = session_status(&vault, now_ts);
    let status = vault_status(&vault, session_status);

    VaultDto {
        address: vault_pubkey.to_string(),
        owner: vault.user_wallet.to_string(),
        delegate: vault.delegate_wallet.map(|pk| pk.to_string()),
        approved_amount_lamports: vault.approved_amount,
        available_amount_lamports: vault.available_amount,
        used_amount_lamports: vault.used_amount,
        total_deposited_lamports: vault.total_deposited,
        total_withdrawn_lamports: vault.total_withdrawn,
        approved_amount_sol: to_sol(vault.approved_amount),
        available_amount_sol: to_sol(vault.available_amount),
        used_amount_sol: to_sol(vault.used_amount),
        total_deposited_sol: to_sol(vault.total_deposited),
        total_withdrawn_sol: to_sol(vault.total_withdrawn),
        trade_count: vault.trade_count,
        session_expiry: vault.session_expires_at,
        delegated_at: vault.delegated_at,
        created_at: vault.created_at,
        last_activity: vault.last_activity,
        is_active: vault.is_active,
        is_paused: vault.is_paused,
        session_status,
        status,
        version: vault.version,
        bump: vault.bump,
    }
}

pub fn to_vault_stats_dto(vault: &EphemeralVaultAccount, now_ts: i64) -> VaultStatsDto {
    let session_status = session_status(vault, now_ts);
    let status = vault_status(vault, session_status);

    VaultStatsDto {
        approved_amount_lamports: vault.approved_amount,
        available_amount_lamports: vault.available_amount,
        used_amount_lamports: vault.used_amount,
        total_deposited_lamports: vault.total_deposited,
        total_withdrawn_lamports: vault.total_withdrawn,
        trade_count: vault.trade_count,
        is_active: vault.is_active,
        is_paused: vault.is_paused,
        session_expiry: vault.session_expires_at,
        session_status,
        status,
    }
}

pub async fn fetch_vault_by_user(
    rpc: &RpcClient,
    config: &Config,
    user_pubkey: Pubkey,
) -> Result<VaultDto> {
    let program_id = config
        .program_id
        .parse::<Pubkey>()
        .map_err(|e| AppError::Internal(format!("invalid PROGRAM_ID: {e}")))?;
    let (vault_pda, _) = derive_vault_pda(&program_id, &user_pubkey);

    let account = rpc
        .get_account(&vault_pda)
        .await
        .map_err(|e| AppError::VaultNotFound(format!("{vault_pda}: {e}")))?;

    let vault = parse_vault_account(&account.data)?;
    let now_ts = rpc
        .get_latest_blockhash()
        .await
        .map(|_| chrono::Utc::now().timestamp())
        .unwrap_or_else(|_| chrono::Utc::now().timestamp());

    Ok(to_vault_dto(vault_pda, vault, now_ts))
}

pub async fn fetch_vault_stats_by_user(
    rpc: &RpcClient,
    config: &Config,
    user_pubkey: Pubkey,
) -> Result<VaultStatsDto> {
    let program_id = config
        .program_id
        .parse::<Pubkey>()
        .map_err(|e| AppError::Internal(format!("invalid PROGRAM_ID: {e}")))?;
    let (vault_pda, _) = derive_vault_pda(&program_id, &user_pubkey);

    let account = rpc
        .get_account(&vault_pda)
        .await
        .map_err(|e| AppError::VaultNotFound(format!("{vault_pda}: {e}")))?;
    let vault = parse_vault_account(&account.data)?;
    Ok(to_vault_stats_dto(&vault, chrono::Utc::now().timestamp()))
}
