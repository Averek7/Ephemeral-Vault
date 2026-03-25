use anchor_lang::{prelude::Pubkey as AnchorPubkey, InstructionData};
use borsh::BorshDeserialize;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::Serialize;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    system_program,
    transaction::Transaction,
};

use crate::config::Config;
use crate::error::{AppError, Result};

const LAMPORTS_PER_SOL: f64 = 1_000_000_000.0;
const RENEWAL_WINDOW_SECONDS: i64 = 300;

#[derive(Clone, Debug, BorshDeserialize)]
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

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TxEnvelope {
    pub transaction_base64: String,
    pub vault_pda: String,
}

fn to_sol(lamports: u64) -> f64 {
    lamports as f64 / LAMPORTS_PER_SOL
}

fn to_anchor_pubkey(pubkey: Pubkey) -> AnchorPubkey {
    AnchorPubkey::new_from_array(pubkey.to_bytes())
}

fn program_id(config: &Config) -> Result<Pubkey> {
    config
        .program_id
        .parse::<Pubkey>()
        .map_err(|e| AppError::Internal(format!("invalid PROGRAM_ID: {e}")))
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
        Some(expires_at) if expires_at - now_ts <= RENEWAL_WINDOW_SECONDS => {
            SessionStatusDto::ExpiringSoon
        }
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

async fn latest_blockhash(rpc: &RpcClient) -> Result<solana_sdk::hash::Hash> {
    rpc.get_latest_blockhash()
        .await
        .map_err(|e| AppError::SolanaRpc(format!("failed to fetch recent blockhash: {e}")))
}

fn encode_transaction(
    payer: Pubkey,
    instructions: Vec<Instruction>,
    blockhash: solana_sdk::hash::Hash,
    vault_pda: Pubkey,
) -> Result<TxEnvelope> {
    let tx = Transaction::new_unsigned(Message::new_with_blockhash(
        &instructions,
        Some(&payer),
        &blockhash,
    ));
    let bytes = bincode::serialize(&tx)
        .map_err(|e| AppError::Internal(format!("failed to serialize transaction: {e}")))?;

    Ok(TxEnvelope {
        transaction_base64: BASE64.encode(bytes),
        vault_pda: vault_pda.to_string(),
    })
}

fn create_vault_instruction(program_id: Pubkey, user: Pubkey, vault_pda: Pubkey, approved_amount: u64) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: ephemeralvault::instruction::CreateEphemeralVault { approved_amount }.data(),
    }
}

fn approve_delegate_instruction(
    program_id: Pubkey,
    user: Pubkey,
    vault_pda: Pubkey,
    delegate: Pubkey,
    custom_duration: Option<i64>,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![AccountMeta::new(vault_pda, false), AccountMeta::new(user, true)],
        data: ephemeralvault::instruction::ApproveDelegate {
            delegate: to_anchor_pubkey(delegate),
            custom_duration,
        }
        .data(),
    }
}

fn deposit_instruction(program_id: Pubkey, user: Pubkey, vault_pda: Pubkey, amount: u64) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(user, true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: ephemeralvault::instruction::AutoDepositForTrade {
            trade_fee_estimate: amount,
        }
        .data(),
    }
}

fn withdraw_instruction(program_id: Pubkey, user: Pubkey, vault_pda: Pubkey, amount: u64) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![AccountMeta::new(vault_pda, false), AccountMeta::new(user, true)],
        data: ephemeralvault::instruction::WithdrawBalance { amount }.data(),
    }
}

fn revoke_instruction(program_id: Pubkey, user: Pubkey, vault_pda: Pubkey) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![AccountMeta::new(vault_pda, false), AccountMeta::new(user, true)],
        data: ephemeralvault::instruction::RevokeAccess {}.data(),
    }
}

fn renew_instruction(program_id: Pubkey, user: Pubkey, vault_pda: Pubkey) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![AccountMeta::new(vault_pda, false), AccountMeta::new_readonly(user, true)],
        data: ephemeralvault::instruction::RenewSession {}.data(),
    }
}

fn reactivate_instruction(program_id: Pubkey, user: Pubkey, vault_pda: Pubkey) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![AccountMeta::new(vault_pda, false), AccountMeta::new_readonly(user, true)],
        data: ephemeralvault::instruction::ReactivateVault {}.data(),
    }
}

fn update_approved_amount_instruction(
    program_id: Pubkey,
    user: Pubkey,
    vault_pda: Pubkey,
    new_approved_amount: u64,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![AccountMeta::new(vault_pda, false), AccountMeta::new_readonly(user, true)],
        data: ephemeralvault::instruction::UpdateApprovedAmount {
            new_approved_amount,
        }
        .data(),
    }
}

fn pause_instruction(program_id: Pubkey, user: Pubkey, vault_pda: Pubkey) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![AccountMeta::new(vault_pda, false), AccountMeta::new_readonly(user, true)],
        data: ephemeralvault::instruction::EmergencyPause {}.data(),
    }
}

fn unpause_instruction(program_id: Pubkey, user: Pubkey, vault_pda: Pubkey) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![AccountMeta::new(vault_pda, false), AccountMeta::new_readonly(user, true)],
        data: ephemeralvault::instruction::UnpauseVault {}.data(),
    }
}

fn execute_trade_instruction(
    program_id: Pubkey,
    delegate: Pubkey,
    vault_pda: Pubkey,
    trade_fee: u64,
    trade_amount: u64,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(vault_pda, false),
            AccountMeta::new_readonly(delegate, true),
        ],
        data: ephemeralvault::instruction::ExecuteTrade {
            trade_fee,
            trade_amount,
        }
        .data(),
    }
}

fn cleanup_instruction(
    program_id: Pubkey,
    vault_pda: Pubkey,
    user_wallet: Pubkey,
    cleaner: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(user_wallet, false),
            AccountMeta::new(cleaner, true),
        ],
        data: ephemeralvault::instruction::CleanupVault {}.data(),
    }
}

pub async fn fetch_vault_by_user(
    rpc: &RpcClient,
    config: &Config,
    user_pubkey: Pubkey,
) -> Result<VaultDto> {
    let program_id = program_id(config)?;
    let (vault_pda, _) = derive_vault_pda(&program_id, &user_pubkey);

    let account = rpc
        .get_account(&vault_pda)
        .await
        .map_err(|e| AppError::VaultNotFound(format!("{vault_pda}: {e}")))?;

    let vault = parse_vault_account(&account.data)?;
    Ok(to_vault_dto(vault_pda, vault, chrono::Utc::now().timestamp()))
}

pub async fn fetch_vault_stats_by_user(
    rpc: &RpcClient,
    config: &Config,
    user_pubkey: Pubkey,
) -> Result<VaultStatsDto> {
    let program_id = program_id(config)?;
    let (vault_pda, _) = derive_vault_pda(&program_id, &user_pubkey);

    let account = rpc
        .get_account(&vault_pda)
        .await
        .map_err(|e| AppError::VaultNotFound(format!("{vault_pda}: {e}")))?;
    let vault = parse_vault_account(&account.data)?;
    Ok(to_vault_stats_dto(&vault, chrono::Utc::now().timestamp()))
}

pub async fn build_create_vault_tx(
    rpc: &RpcClient,
    config: &Config,
    user: Pubkey,
    approved_amount_lamports: u64,
    delegate: Option<Pubkey>,
    custom_duration_seconds: Option<i64>,
    initial_deposit_lamports: Option<u64>,
) -> Result<TxEnvelope> {
    let program_id = program_id(config)?;
    let (vault_pda, _) = derive_vault_pda(&program_id, &user);

    let mut instructions = vec![create_vault_instruction(
        program_id,
        user,
        vault_pda,
        approved_amount_lamports,
    )];

    if let Some(delegate) = delegate {
        instructions.push(approve_delegate_instruction(
            program_id,
            user,
            vault_pda,
            delegate,
            custom_duration_seconds,
        ));
    }

    if let Some(amount) = initial_deposit_lamports.filter(|amount| *amount > 0) {
        instructions.push(deposit_instruction(program_id, user, vault_pda, amount));
    }

    encode_transaction(user, instructions, latest_blockhash(rpc).await?, vault_pda)
}

pub async fn build_deposit_tx(
    rpc: &RpcClient,
    config: &Config,
    user: Pubkey,
    amount_lamports: u64,
) -> Result<TxEnvelope> {
    let program_id = program_id(config)?;
    let (vault_pda, _) = derive_vault_pda(&program_id, &user);
    encode_transaction(
        user,
        vec![deposit_instruction(program_id, user, vault_pda, amount_lamports)],
        latest_blockhash(rpc).await?,
        vault_pda,
    )
}

pub async fn build_withdraw_tx(
    rpc: &RpcClient,
    config: &Config,
    user: Pubkey,
    amount_lamports: u64,
) -> Result<TxEnvelope> {
    let program_id = program_id(config)?;
    let (vault_pda, _) = derive_vault_pda(&program_id, &user);
    encode_transaction(
        user,
        vec![withdraw_instruction(program_id, user, vault_pda, amount_lamports)],
        latest_blockhash(rpc).await?,
        vault_pda,
    )
}

pub async fn build_pause_tx(rpc: &RpcClient, config: &Config, user: Pubkey) -> Result<TxEnvelope> {
    let program_id = program_id(config)?;
    let (vault_pda, _) = derive_vault_pda(&program_id, &user);
    encode_transaction(
        user,
        vec![pause_instruction(program_id, user, vault_pda)],
        latest_blockhash(rpc).await?,
        vault_pda,
    )
}

pub async fn build_unpause_tx(
    rpc: &RpcClient,
    config: &Config,
    user: Pubkey,
) -> Result<TxEnvelope> {
    let program_id = program_id(config)?;
    let (vault_pda, _) = derive_vault_pda(&program_id, &user);
    encode_transaction(
        user,
        vec![unpause_instruction(program_id, user, vault_pda)],
        latest_blockhash(rpc).await?,
        vault_pda,
    )
}

pub async fn build_revoke_tx(rpc: &RpcClient, config: &Config, user: Pubkey) -> Result<TxEnvelope> {
    let program_id = program_id(config)?;
    let (vault_pda, _) = derive_vault_pda(&program_id, &user);
    encode_transaction(
        user,
        vec![revoke_instruction(program_id, user, vault_pda)],
        latest_blockhash(rpc).await?,
        vault_pda,
    )
}

pub async fn build_renew_session_tx(
    rpc: &RpcClient,
    config: &Config,
    user: Pubkey,
) -> Result<TxEnvelope> {
    let program_id = program_id(config)?;
    let (vault_pda, _) = derive_vault_pda(&program_id, &user);
    encode_transaction(
        user,
        vec![renew_instruction(program_id, user, vault_pda)],
        latest_blockhash(rpc).await?,
        vault_pda,
    )
}

pub async fn build_approve_delegate_tx(
    rpc: &RpcClient,
    config: &Config,
    user: Pubkey,
    delegate: Pubkey,
    custom_duration_seconds: Option<i64>,
) -> Result<TxEnvelope> {
    let program_id = program_id(config)?;
    let (vault_pda, _) = derive_vault_pda(&program_id, &user);
    encode_transaction(
        user,
        vec![approve_delegate_instruction(
            program_id,
            user,
            vault_pda,
            delegate,
            custom_duration_seconds,
        )],
        latest_blockhash(rpc).await?,
        vault_pda,
    )
}

pub async fn build_reactivate_tx(
    rpc: &RpcClient,
    config: &Config,
    user: Pubkey,
) -> Result<TxEnvelope> {
    let program_id = program_id(config)?;
    let (vault_pda, _) = derive_vault_pda(&program_id, &user);
    encode_transaction(
        user,
        vec![reactivate_instruction(program_id, user, vault_pda)],
        latest_blockhash(rpc).await?,
        vault_pda,
    )
}

pub async fn build_update_approved_amount_tx(
    rpc: &RpcClient,
    config: &Config,
    user: Pubkey,
    new_approved_amount_lamports: u64,
) -> Result<TxEnvelope> {
    let program_id = program_id(config)?;
    let (vault_pda, _) = derive_vault_pda(&program_id, &user);
    encode_transaction(
        user,
        vec![update_approved_amount_instruction(
            program_id,
            user,
            vault_pda,
            new_approved_amount_lamports,
        )],
        latest_blockhash(rpc).await?,
        vault_pda,
    )
}

pub async fn build_execute_trade_tx(
    rpc: &RpcClient,
    config: &Config,
    vault_pda: Pubkey,
    delegate: Pubkey,
    trade_fee_lamports: u64,
    trade_amount_lamports: u64,
) -> Result<TxEnvelope> {
    let program_id = program_id(config)?;
    encode_transaction(
        delegate,
        vec![execute_trade_instruction(
            program_id,
            delegate,
            vault_pda,
            trade_fee_lamports,
            trade_amount_lamports,
        )],
        latest_blockhash(rpc).await?,
        vault_pda,
    )
}

pub async fn build_cleanup_tx(
    rpc: &RpcClient,
    config: &Config,
    vault_pda: Pubkey,
    cleaner: Pubkey,
) -> Result<TxEnvelope> {
    let program_id = program_id(config)?;
    let account = rpc
        .get_account(&vault_pda)
        .await
        .map_err(|e| AppError::VaultNotFound(format!("{vault_pda}: {e}")))?;
    let vault = parse_vault_account(&account.data)?;

    encode_transaction(
        cleaner,
        vec![cleanup_instruction(
            program_id,
            vault_pda,
            vault.user_wallet,
            cleaner,
        )],
        latest_blockhash(rpc).await?,
        vault_pda,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_vault() -> EphemeralVaultAccount {
        EphemeralVaultAccount {
            user_wallet: Pubkey::new_unique(),
            vault_pda: Pubkey::new_unique(),
            created_at: 1_700_000_000,
            last_activity: 1_700_000_100,
            approved_amount: 2_000_000,
            used_amount: 250_000,
            available_amount: 500_000,
            delegate_wallet: Some(Pubkey::new_unique()),
            delegated_at: Some(1_700_000_050),
            session_expires_at: Some(1_700_000_600),
            total_deposited: 750_000,
            total_withdrawn: 100_000,
            trade_count: 3,
            is_active: true,
            is_paused: false,
            version: 1,
            bump: 254,
        }
    }

    #[test]
    fn derive_vault_pda_uses_expected_seed_scheme() {
        let program_id = Pubkey::new_unique();
        let user = Pubkey::new_unique();

        let (expected_pda, expected_bump) =
            Pubkey::find_program_address(&[b"vault", user.as_ref()], &program_id);
        let (actual_pda, actual_bump) = derive_vault_pda(&program_id, &user);

        assert_eq!(actual_pda, expected_pda);
        assert_eq!(actual_bump, expected_bump);
    }

    #[test]
    fn vault_stats_marks_session_states_correctly() {
        let active_vault = sample_vault();
        let active = to_vault_stats_dto(&active_vault, 1_700_000_200);
        assert!(matches!(active.session_status, SessionStatusDto::Active));
        assert!(matches!(active.status, VaultStatusDto::Active));

        let expiring = to_vault_stats_dto(&active_vault, 1_700_000_310);
        assert!(matches!(expiring.session_status, SessionStatusDto::ExpiringSoon));

        let expired = to_vault_stats_dto(&active_vault, 1_700_000_650);
        assert!(matches!(expired.session_status, SessionStatusDto::Expired));
        assert!(matches!(expired.status, VaultStatusDto::Expired));
    }

    #[test]
    fn vault_stats_prioritizes_pause_and_inactive_flags() {
        let mut paused_vault = sample_vault();
        paused_vault.is_paused = true;
        let paused = to_vault_stats_dto(&paused_vault, 1_700_000_650);
        assert!(matches!(paused.status, VaultStatusDto::Paused));

        let mut inactive_vault = sample_vault();
        inactive_vault.is_active = false;
        let inactive = to_vault_stats_dto(&inactive_vault, 1_700_000_200);
        assert!(matches!(inactive.status, VaultStatusDto::Inactive));
    }

    #[test]
    fn create_vault_instruction_matches_contract_accounts() {
        let program_id = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let vault = Pubkey::new_unique();
        let ix = create_vault_instruction(program_id, user, vault, 42);

        assert_eq!(ix.program_id, program_id);
        assert_eq!(ix.accounts.len(), 3);
        assert_eq!(ix.accounts[0], AccountMeta::new(user, true));
        assert_eq!(ix.accounts[1], AccountMeta::new(vault, false));
        assert_eq!(
            ix.accounts[2],
            AccountMeta::new_readonly(system_program::ID, false)
        );
        assert_eq!(
            ix.data,
            ephemeralvault::instruction::CreateEphemeralVault {
                approved_amount: 42,
            }
            .data()
        );
    }

    #[test]
    fn approve_delegate_instruction_encodes_delegate_and_duration() {
        let program_id = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let vault = Pubkey::new_unique();
        let delegate = Pubkey::new_unique();
        let ix = approve_delegate_instruction(program_id, user, vault, delegate, Some(900));

        assert_eq!(ix.accounts, vec![AccountMeta::new(vault, false), AccountMeta::new(user, true)]);
        assert_eq!(
            ix.data,
            ephemeralvault::instruction::ApproveDelegate {
                delegate: to_anchor_pubkey(delegate),
                custom_duration: Some(900),
            }
            .data()
        );
    }

    #[test]
    fn cleanup_instruction_orders_accounts_for_close() {
        let program_id = Pubkey::new_unique();
        let vault = Pubkey::new_unique();
        let user_wallet = Pubkey::new_unique();
        let cleaner = Pubkey::new_unique();
        let ix = cleanup_instruction(program_id, vault, user_wallet, cleaner);

        assert_eq!(ix.program_id, program_id);
        assert_eq!(
            ix.accounts,
            vec![
                AccountMeta::new(vault, false),
                AccountMeta::new(user_wallet, false),
                AccountMeta::new(cleaner, true),
            ]
        );
        assert_eq!(
            ix.data,
            ephemeralvault::instruction::CleanupVault {}.data()
        );
    }
}
