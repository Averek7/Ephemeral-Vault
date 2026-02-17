use crate::{
    session_manager::{fetch_session, parse_pubkey},
    AppState, ApproveRequest, ApproveResponse,
};

use actix_web::{web, Error, HttpResponse};
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use chrono::Utc;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signature,
};

// ============================================================================
// POST /session/approve
// Approves an ephemeral wallet as the trading delegate for the vault.
// Optionally accepts a custom session duration (capped to 3600s on-chain).
// ============================================================================

pub async fn approve_delegate(
    req: web::Json<ApproveRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let session = fetch_session(&state, req.session_id).await?;

    // Guard: session must be active before delegation can occur
    if !session.is_active {
        return Err(ErrorBadRequest("Session is not active"));
    }

    // Parse all required public keys
    let user_pubkey     = parse_pubkey(&session.user_wallet,     "user wallet")?;
    let vault_pda       = parse_pubkey(&session.vault_pda,       "vault PDA")?;
    let delegate_pubkey = parse_pubkey(&session.ephemeral_wallet, "ephemeral wallet")?;

    // Build and submit approve_delegate instruction
    let _ix = build_approve_delegate_ix(
        &state.program_id,
        &vault_pda,
        &user_pubkey,
        &delegate_pubkey,
        None, // Use default on-chain session duration (1 hour)
    );

    // NOTE: In production, serialize + return tx for user-side signing,
    // or sign with a backend-managed fee-payer keypair.
    let signature = Signature::default(); // Placeholder

    // Stamp last_activity in DB
    sqlx::query!(
        "UPDATE ephemeral_sessions SET last_activity = $1 WHERE session_id = $2",
        Utc::now(),
        req.session_id,
    )
    .execute(&state.db)
    .await
    .map_err(ErrorInternalServerError)?;

    log::info!(
        "Delegate approved for session {} → delegate {}",
        req.session_id,
        session.ephemeral_wallet
    );

    Ok(HttpResponse::Ok().json(ApproveResponse {
        session_id: req.session_id,
        delegate_wallet: session.ephemeral_wallet,
        expires_at: session.expires_at,
        transaction_signature: signature.to_string(),
    }))
}

// ============================================================================
// INSTRUCTION BUILDERS
// ============================================================================

/// Builds the `approve_delegate` Anchor instruction.
///
/// # Arguments
/// * `program_id`       - Deployed program pubkey
/// * `vault_pda`        - Vault PDA account
/// * `user`             - Vault owner (signer)
/// * `delegate`         - Ephemeral wallet being approved
/// * `custom_duration`  - Optional custom TTL in seconds (None → 3600s default)
pub fn build_approve_delegate_ix(
    program_id: &Pubkey,
    vault_pda: &Pubkey,
    user: &Pubkey,
    delegate: &Pubkey,
    custom_duration: Option<i64>,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*vault_pda, false),
        AccountMeta::new(*user, true),
    ];

    // Anchor discriminator for approve_delegate
    let mut data = vec![0x8d, 0x3e, 0x72, 0xa1, 0x45, 0xb9, 0xc0, 0x2f];
    data.extend_from_slice(delegate.as_ref());

    // Encode Option<i64> custom duration
    match custom_duration {
        Some(duration) => {
            data.push(1); // Option::Some discriminant
            data.extend_from_slice(&duration.to_le_bytes());
        }
        None => {
            data.push(0); // Option::None discriminant
        }
    }

    Instruction {
        program_id: *program_id,
        accounts,
        data,
    }
}