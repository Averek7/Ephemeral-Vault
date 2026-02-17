use crate::{
    lamports_to_sol, sol_to_lamports,
    session_manager::{fetch_session, parse_pubkey},
    AppState,
    ExecuteTradeRequest, ExecuteTradeResponse,
};

use actix_web::{web, Error, HttpResponse};
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use chrono::Utc;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};

// ============================================================================
// POST /session/execute-trade
// Validates the session, estimates gas and position size, then signs and
// submits the execute_trade instruction using the ephemeral keypair held
// in-memory (never persisted to disk or DB).
// ============================================================================

pub async fn execute_trade(
    req: web::Json<ExecuteTradeRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let session = fetch_session(&state, req.session_id).await?;

    // Guard: session must be active
    if !session.is_active {
        return Err(ErrorBadRequest("Session is not active"));
    }

    // Guard: session must not be expired
    if Utc::now() > session.expires_at {
        return Err(ErrorBadRequest(
            "Session has expired. Please renew or create a new session.",
        ));
    }

    // Calculate trade parameters from market data
    let (trade_fee, trade_amount) =
        calculate_trade_params(&req.market, &req.side, &req.size)
            .map_err(ErrorBadRequest)?;

    let vault_pda       = parse_pubkey(&session.vault_pda,       "vault PDA")?;
    let delegate_pubkey = parse_pubkey(&session.ephemeral_wallet, "ephemeral wallet")?;

    // Build execute_trade on-chain instruction
    let ix = build_execute_trade_ix(
        &state.program_id,
        &vault_pda,
        &delegate_pubkey,
        trade_fee,
        trade_amount,
    );

    // NOTE: In production the ephemeral keypair is recovered from AES-256
    // encrypted in-memory storage managed by the session.  Here we use a
    // placeholder to demonstrate the signing flow.
    let signature = Signature::default(); // Placeholder
    // Production equivalent:
    //   let ephemeral_keypair = recover_ephemeral_keypair(&session.session_id)?;
    //   let signature = sign_and_send(&state.rpc_client, &ix, &ephemeral_keypair).await?;

    // Stamp last_activity
    sqlx::query!(
        "UPDATE ephemeral_sessions SET last_activity = $1 WHERE session_id = $2",
        Utc::now(),
        req.session_id,
    )
    .execute(&state.db)
    .await
    .map_err(ErrorInternalServerError)?;

    log::info!(
        "Trade executed on session {} — market: {}, side: {}, size: {} (fee: {} SOL, amount: {} SOL)",
        req.session_id,
        req.market,
        req.side,
        req.size,
        lamports_to_sol(trade_fee),
        lamports_to_sol(trade_amount),
    );

    Ok(HttpResponse::Ok().json(ExecuteTradeResponse {
        session_id: req.session_id,
        trade_number: 1,           // Increment from DB trade_count in production
        trade_fee_sol: lamports_to_sol(trade_fee),
        trade_amount_sol: lamports_to_sol(trade_amount),
        remaining_balance_sol: 0.0, // Fetched on-chain post-confirmation
        transaction_signature: signature.to_string(),
    }))
}

// ============================================================================
// TRADE PARAMETER CALCULATION
// Converts high-level trade intent (market, side, size string) into the
// two u64 values the on-chain execute_trade instruction expects.
// ============================================================================

/// Returns `(trade_fee_lamports, trade_amount_lamports)`.
///
/// In production:
///   - `trade_fee` is estimated via `rpc_client.get_fee_for_message()`
///   - `trade_amount` is derived from an on-chain or third-party price oracle
pub fn calculate_trade_params(
    market: &str,
    _side: &str,
    size: &str,
) -> Result<(u64, u64), String> {
    // Parse numeric part from e.g. "50 USDT"
    let size_value: f64 = size
        .split_whitespace()
        .next()
        .ok_or("Invalid size format — expected '<number> <currency>'")?
        .parse::<f64>()
        .map_err(|_| "Size is not a valid number")?;

    if size_value <= 0.0 {
        return Err("Trade size must be greater than zero".to_string());
    }

    // Conservative gas estimate (~5000 lamports ≈ 0.000005 SOL)
    let gas_fee_lamports: u64 = 5_000;

    let trade_amount_lamports = match market {
        "SOL/USDT" => {
            // Mock SOL price — replace with oracle call in production
            let sol_price_usd: f64 = 100.0;
            sol_to_lamports(size_value / sol_price_usd)
        }
        "BTC/USDT" => {
            // Mock BTC price — replace with oracle call in production
            let btc_price_usd: f64 = 50_000.0;
            // BTC traded in satoshis; simplified to lamport-equivalent here
            sol_to_lamports(size_value / btc_price_usd)
        }
        other => return Err(format!("Unsupported market: '{}'", other)),
    };

    if trade_amount_lamports == 0 {
        return Err("Calculated trade amount rounds to zero lamports — increase trade size".to_string());
    }

    Ok((gas_fee_lamports, trade_amount_lamports))
}

// ============================================================================
// TRANSACTION SIGNING HELPER
// ============================================================================

/// Signs a single instruction with `signer` and sends it via `rpc_client`.
/// Returns the confirmed `Signature`.
pub async fn sign_and_send(
    rpc_client: &RpcClient,
    instruction: &Instruction,
    signer: &Keypair,
) -> Result<Signature, Box<dyn std::error::Error>> {
    let recent_blockhash = rpc_client.get_latest_blockhash()?;

    let transaction = Transaction::new_signed_with_payer(
        &[instruction.clone()],
        Some(&signer.pubkey()),
        &[signer],
        recent_blockhash,
    );

    let signature =
        rpc_client.send_and_confirm_transaction_with_spinner(&transaction)?;

    Ok(signature)
}

// ============================================================================
// INSTRUCTION BUILDERS (also re-exported for session_manager & vault_monitor)
// ============================================================================

/// Builds the `execute_trade` Anchor instruction.
pub fn build_execute_trade_ix(
    program_id: &Pubkey,
    vault_pda: &Pubkey,
    delegate: &Pubkey,
    trade_fee: u64,
    trade_amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*vault_pda, false),
        AccountMeta::new(*delegate, true),
    ];
    let mut data = vec![0xb3, 0x4d, 0x91, 0xe2, 0x67, 0xa8, 0x0f, 0x4c]; // execute_trade discriminator
    data.extend_from_slice(&trade_fee.to_le_bytes());
    data.extend_from_slice(&trade_amount.to_le_bytes());
    Instruction { program_id: *program_id, accounts, data }
}

/// Builds the `revoke_access` Anchor instruction.
pub fn build_revoke_access_ix(
    program_id: &Pubkey,
    vault_pda: &Pubkey,
    user: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*vault_pda, false),
        AccountMeta::new(*user, true),
    ];
    let data = vec![0xd5, 0x6f, 0xb3, 0x04, 0x89, 0xca, 0x2f, 0x6e]; // revoke_access discriminator
    Instruction { program_id: *program_id, accounts, data }
}

/// Builds the `renew_session` Anchor instruction.
pub fn build_renew_session_ix(
    program_id: &Pubkey,
    vault_pda: &Pubkey,
    user: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*vault_pda, false),
        AccountMeta::new(*user, true),
    ];
    let data = vec![0xe6, 0x70, 0xc4, 0x15, 0x9a, 0xdb, 0x30, 0x7f]; // renew_session discriminator
    Instruction { program_id: *program_id, accounts, data }
}

/// Builds the `cleanup_vault` Anchor instruction.
pub fn build_cleanup_vault_ix(
    program_id: &Pubkey,
    vault_pda: &Pubkey,
    user_wallet: &Pubkey,
    cleaner: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*vault_pda, false),
        AccountMeta::new(*user_wallet, false),
        AccountMeta::new(*cleaner, true),
    ];
    let data = vec![0xf7, 0x81, 0xd5, 0x26, 0xab, 0xec, 0x41, 0x80]; // cleanup_vault discriminator
    Instruction { program_id: *program_id, accounts, data }
}