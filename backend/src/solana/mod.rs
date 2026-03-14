use crate::error::{AppError, Result};
use base64::Engine;
use borsh::{BorshDeserialize, BorshSerialize};
use sha2::{Digest, Sha256};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::hash::Hash;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::message::Message;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_program;
use solana_sdk::transaction::Transaction;

pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

fn anchor_discriminator(prefix: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{prefix}:{name}");
    let hash = Sha256::digest(preimage.as_bytes());
    hash[..8].try_into().expect("slice is 8 bytes")
}

fn ix_data(method: &str, args: Vec<u8>) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + args.len());
    out.extend_from_slice(&anchor_discriminator("global", method));
    out.extend_from_slice(&args);
    out
}

fn account_discriminator(name: &str) -> [u8; 8] {
    anchor_discriminator("account", name)
}

pub fn derive_vault_pda(program_id: &Pubkey, user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault", user.as_ref()], program_id)
}

pub async fn latest_blockhash(rpc: &RpcClient) -> Result<Hash> {
    rpc.get_latest_blockhash()
        .await
        .map_err(|e| AppError::SolanaRpc(e.to_string()))
}

pub fn build_anchor_instruction(
    program_id: Pubkey,
    method: &str,
    accounts: Vec<AccountMeta>,
    args_borsh: Vec<u8>,
) -> Instruction {
    Instruction {
        program_id,
        accounts,
        data: ix_data(method, args_borsh),
    }
}

pub fn build_unsigned_tx_base64(
    fee_payer: Pubkey,
    instructions: Vec<Instruction>,
    recent_blockhash: Hash,
) -> Result<String> {
    let message = Message::new(&instructions, Some(&fee_payer));
    let mut tx = Transaction::new_unsigned(message);
    tx.message.recent_blockhash = recent_blockhash;

    let bytes =
        bincode::serialize(&tx).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct EphemeralVault {
    pub user_wallet: [u8; 32],
    pub vault_pda: [u8; 32],
    pub created_at: i64,
    pub last_activity: i64,
    pub approved_amount: u64,
    pub used_amount: u64,
    pub available_amount: u64,
    pub delegate_wallet: Option<[u8; 32]>,
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

pub fn decode_ephemeral_vault_account(data: &[u8]) -> Result<EphemeralVault> {
    if data.len() < 8 {
        return Err(AppError::Internal("account data too short".into()));
    }
    let expected = account_discriminator("EphemeralVault");
    if data[..8] != expected {
        return Err(AppError::Internal("unexpected account discriminator".into()));
    }
    let mut slice = &data[8..];
    EphemeralVault::deserialize(&mut slice)
        .map_err(|e| AppError::Internal(format!("borsh decode failed: {e}")))
}

pub fn sol_from_lamports(lamports: u64) -> f64 {
    (lamports as f64) / (LAMPORTS_PER_SOL as f64)
}

// --------------------------
// Anchor instruction args
// --------------------------

#[derive(BorshSerialize)]
pub struct CreateEphemeralVaultArgs {
    pub approved_amount: u64,
}

#[derive(BorshSerialize)]
pub struct ApproveDelegateArgs {
    pub delegate: Pubkey,
    pub custom_duration: Option<i64>,
}

#[derive(BorshSerialize)]
pub struct AutoDepositArgs {
    pub trade_fee_estimate: u64,
}

#[derive(BorshSerialize)]
pub struct WithdrawBalanceArgs {
    pub amount: u64,
}

// Convenience metas
pub fn meta_user_signer_writable(user: Pubkey) -> AccountMeta {
    AccountMeta::new(user, true)
}

pub fn meta_user_signer_readonly(user: Pubkey) -> AccountMeta {
    AccountMeta::new_readonly(user, true)
}

pub fn meta_vault_writable(vault: Pubkey) -> AccountMeta {
    AccountMeta::new(vault, false)
}

pub fn meta_system_program() -> AccountMeta {
    AccountMeta::new_readonly(system_program::ID, false)
}

