use crate::{
    lamports_to_sol, sol_to_lamports, validate_deposit,
    session_manager::{fetch_session, parse_pubkey},
    AppState,
    DepositRequest, DepositResponse,
    WithdrawRequest, WithdrawResponse,
};

use actix_web::{web, Error, HttpResponse};
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use chrono::Utc;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signature,
    system_program,
};

// ============================================================================
// POST /session/deposit
// Transfers SOL from the user's wallet into the vault PDA to fund trade fees.
// Validates deposit size and running total against the approved ceiling.
// ============================================================================

pub async fn deposit(
    req: web::Json<DepositRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    // Convert SOL → lamports and validate bounds
    let fee_lamports = sol_to_lamports(req.estimated_fee_sol);
    validate_deposit(fee_lamports).map_err(ErrorBadRequest)?;

    let session = fetch_session(&state, req.session_id).await?;

    // Guard: vault must be active and not paused
    if !session.is_active {
        return Err(ErrorBadRequest("Session is not active"));
    }

    // Guard: deposit must not push total above approved ceiling
    let new_total = (session.total_deposited as u64)
        .checked_add(fee_lamports)
        .ok_or_else(|| ErrorBadRequest("Arithmetic overflow calculating new total"))?;

    if new_total > session.approved_amount as u64 {
        return Err(ErrorBadRequest(format!(
            "Deposit would exceed approved amount. Remaining allowance: {} SOL",
            lamports_to_sol(session.approved_amount as u64 - session.total_deposited as u64)
        )));
    }

    let user_pubkey = parse_pubkey(&session.user_wallet, "user wallet")?;
    let vault_pda   = parse_pubkey(&session.vault_pda,   "vault PDA")?;

    // Build auto_deposit_for_trade instruction
    let _ix = build_auto_deposit_ix(&state.program_id, &vault_pda, &user_pubkey, fee_lamports);

    // NOTE: User must sign — serialize and return tx in production.
    let signature = Signature::default(); // Placeholder

    // Update running deposit total in DB
    sqlx::query!(
        r#"
        UPDATE ephemeral_sessions
        SET total_deposited = total_deposited + $1,
            last_activity   = $2
        WHERE session_id = $3
        "#,
        fee_lamports as i64,
        Utc::now(),
        req.session_id,
    )
    .execute(&state.db)
    .await
    .map_err(ErrorInternalServerError)?;

    log::info!(
        "Deposit {} SOL into session {} (new total: {} SOL)",
        req.estimated_fee_sol,
        req.session_id,
        lamports_to_sol(new_total)
    );

    Ok(HttpResponse::Ok().json(DepositResponse {
        session_id: req.session_id,
        deposited_sol: req.estimated_fee_sol,
        deposited_lamports: fee_lamports,
        total_deposited_sol: lamports_to_sol(new_total),
        available_balance_sol: lamports_to_sol(new_total), // Simplified; query on-chain for precision
        transaction_signature: signature.to_string(),
    }))
}

// ============================================================================
// POST /session/withdraw
// Transfers available SOL back from the vault PDA to the user's wallet.
// Passing amount_sol = 0 signals "withdraw everything available".
// ============================================================================

pub async fn withdraw_balance(
    req: web::Json<WithdrawRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let session = fetch_session(&state, req.session_id).await?;

    if !session.is_active {
        return Err(ErrorBadRequest("Session is not active"));
    }

    // 0 means withdraw all available balance
    let amount_lamports = if req.amount_sol == 0.0 {
        0u64 // on-chain contract handles "0 = all"
    } else {
        let lamports = sol_to_lamports(req.amount_sol);
        // Basic sanity check — on-chain will enforce rent-exempt minimum
        if lamports == 0 {
            return Err(ErrorBadRequest("Withdrawal amount too small"));
        }
        lamports
    };

    let user_pubkey = parse_pubkey(&session.user_wallet, "user wallet")?;
    let vault_pda   = parse_pubkey(&session.vault_pda,   "vault PDA")?;

    // Build withdraw_balance instruction
    let _ix = build_withdraw_balance_ix(
        &state.program_id,
        &vault_pda,
        &user_pubkey,
        amount_lamports,
    );

    // NOTE: User must sign — serialize and return tx in production.
    let signature = Signature::default(); // Placeholder

    // Stamp activity; remaining_balance_sol would be filled post-confirmation
    sqlx::query!(
        "UPDATE ephemeral_sessions SET last_activity = $1 WHERE session_id = $2",
        Utc::now(),
        req.session_id,
    )
    .execute(&state.db)
    .await
    .map_err(ErrorInternalServerError)?;

    log::info!(
        "Withdraw {} SOL from session {}",
        req.amount_sol,
        req.session_id
    );

    Ok(HttpResponse::Ok().json(WithdrawResponse {
        session_id: req.session_id,
        withdrawn_sol: req.amount_sol,
        remaining_balance_sol: 0.0, // Populated from on-chain state post-confirmation
        transaction_signature: signature.to_string(),
    }))
}

// ============================================================================
// INSTRUCTION BUILDERS
// ============================================================================

/// Builds the `auto_deposit_for_trade` Anchor instruction.
pub fn build_auto_deposit_ix(
    program_id: &Pubkey,
    vault_pda: &Pubkey,
    user: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*vault_pda, false),
        AccountMeta::new(*user, true),
        AccountMeta::new_readonly(system_program::ID, false),
    ];
    let mut data = vec![0xa2, 0x1f, 0x83, 0xd4, 0x56, 0xc7, 0x98, 0x3b]; // auto_deposit discriminator
    data.extend_from_slice(&amount.to_le_bytes());
    Instruction { program_id: *program_id, accounts, data }
}

/// Builds the `withdraw_balance` Anchor instruction.
pub fn build_withdraw_balance_ix(
    program_id: &Pubkey,
    vault_pda: &Pubkey,
    user: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*vault_pda, false),
        AccountMeta::new(*user, true),
    ];
    let mut data = vec![0xc4, 0x5e, 0xa2, 0xf3, 0x78, 0xb9, 0x1e, 0x5d]; // withdraw_balance discriminator
    data.extend_from_slice(&amount.to_le_bytes());
    Instruction { program_id: *program_id, accounts, data }
}