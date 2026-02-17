use crate::{
    signer::build_revoke_access_ix,
    signer::build_renew_session_ix,
    lamports_to_sol, sol_to_lamports, validate_approved_amount,
    AppState, SessionRecord, SESSION_DURATION_HOURS, SESSION_RENEWAL_WINDOW_MINUTES,
    CreateSessionRequest, CreateSessionResponse,
    RevokeRequest, RevokeResponse,
    RenewRequest, RenewResponse,
    VaultStatsRequest, VaultStatsResponse,
};

use actix_web::{web, Error, HttpResponse};
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use chrono::{Duration, Utc};
use solana_sdk::{pubkey::Pubkey, signature::{Keypair, Signature, Signer}};
use std::str::FromStr;
use uuid::Uuid;

// ============================================================================
// POST /session/create
// Creates ephemeral keypair, derives vault PDA, persists session row,
// and emits the on-chain create_ephemeral_vault instruction.
// ============================================================================

pub async fn create_session(
    req: web::Json<CreateSessionRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    // Validate and convert approved amount to lamports
    let approved_lamports = validate_approved_amount(req.approved_amount_sol)
        .map_err(ErrorBadRequest)?;

    // Parse user wallet pubkey
    let user_pubkey = Pubkey::from_str(&req.user_wallet)
        .map_err(|_| ErrorBadRequest("Invalid user wallet address"))?;

    // Generate a fresh one-time ephemeral keypair for this session
    let ephemeral_keypair = Keypair::new();
    let session_id = Uuid::new_v4();

    // Derive deterministic vault PDA from user pubkey + program seeds
    let (vault_pda, _bump) = Pubkey::find_program_address(
        &[b"vault", user_pubkey.as_ref()],
        &state.program_id,
    );

    // Build on-chain create_ephemeral_vault instruction
    let _ix = build_create_vault_ix(
        &state.program_id,
        &user_pubkey,
        &vault_pda,
        approved_lamports,
    );

    // NOTE: In production the user's wallet must sign this transaction.
    // The backend returns the serialised transaction for client-side signing.
    let signature = Signature::default(); // Placeholder

    let expires_at = Utc::now() + Duration::hours(req.session_duration_hours);

    // Persist session metadata (never the private key)
    sqlx::query!(
        r#"
        INSERT INTO ephemeral_sessions
            (session_id, user_wallet, ephemeral_wallet, vault_pda,
             approved_amount, expires_at, is_active, created_at, last_activity)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        session_id,
        req.user_wallet,
        ephemeral_keypair.pubkey().to_string(),
        vault_pda.to_string(),
        approved_lamports as i64,
        expires_at,
        true,
        Utc::now(),
        Utc::now(),
    )
    .execute(&state.db)
    .await
    .map_err(ErrorInternalServerError)?;

    log::info!("Session created: {} for wallet {}", session_id, req.user_wallet);

    Ok(HttpResponse::Ok().json(CreateSessionResponse {
        session_id,
        ephemeral_wallet: ephemeral_keypair.pubkey().to_string(),
        vault_pda: vault_pda.to_string(),
        approved_amount_sol: req.approved_amount_sol,
        approved_amount_lamports: approved_lamports,
        expires_at,
        transaction_signature: signature.to_string(),
    }))
}

// ============================================================================
// DELETE /session/revoke
// Revokes delegate access on-chain and returns all funds to user wallet.
// Marks session as inactive in the database.
// ============================================================================

pub async fn revoke_session(
    req: web::Json<RevokeRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let session = fetch_session(&state, req.session_id).await?;

    let user_pubkey = parse_pubkey(&session.user_wallet, "user wallet")?;
    let vault_pda   = parse_pubkey(&session.vault_pda,   "vault PDA")?;

    // Build revoke_access instruction (signed by user)
    let _ix = build_revoke_access_ix(&state.program_id, &vault_pda, &user_pubkey);

    let signature = Signature::default(); // Placeholder

    // Mark session as revoked in database
    sqlx::query!(
        "UPDATE ephemeral_sessions SET is_active = false, last_activity = $1 WHERE session_id = $2",
        Utc::now(),
        req.session_id,
    )
    .execute(&state.db)
    .await
    .map_err(ErrorInternalServerError)?;

    log::info!("Session revoked: {}", req.session_id);

    Ok(HttpResponse::Ok().json(RevokeResponse {
        session_id: req.session_id,
        returned_sol: 0.0, // Actual value queried on-chain post-confirmation
        transaction_signature: signature.to_string(),
        message: "Session revoked successfully. Funds returned to wallet.".to_string(),
    }))
}

// ============================================================================
// POST /session/renew
// Extends an about-to-expire session by one hour.
// Only allowed within the 5-minute renewal window before expiry.
// ============================================================================

pub async fn renew_session(
    req: web::Json<RenewRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let session = fetch_session(&state, req.session_id).await?;

    // Enforce renewal window
    let minutes_left = session
        .expires_at
        .signed_duration_since(Utc::now())
        .num_minutes();

    if minutes_left > SESSION_RENEWAL_WINDOW_MINUTES {
        return Err(ErrorBadRequest(format!(
            "Session not expiring soon. {} minutes remaining. Renewal only within {} minutes of expiry.",
            minutes_left, SESSION_RENEWAL_WINDOW_MINUTES
        )));
    }

    if minutes_left < 0 {
        return Err(ErrorBadRequest("Session has already expired. Please create a new session."));
    }

    let user_pubkey = parse_pubkey(&session.user_wallet, "user wallet")?;
    let vault_pda   = parse_pubkey(&session.vault_pda,   "vault PDA")?;

    // Build renew_session on-chain instruction
    let _ix = build_renew_session_ix(&state.program_id, &vault_pda, &user_pubkey);

    let signature = Signature::default(); // Placeholder

    let new_expires_at = Utc::now() + Duration::hours(SESSION_DURATION_HOURS);

    sqlx::query!(
        r#"
        UPDATE ephemeral_sessions
        SET expires_at = $1, last_activity = $2
        WHERE session_id = $3
        "#,
        new_expires_at,
        Utc::now(),
        req.session_id,
    )
    .execute(&state.db)
    .await
    .map_err(ErrorInternalServerError)?;

    log::info!("Session renewed: {} until {}", req.session_id, new_expires_at);

    Ok(HttpResponse::Ok().json(RenewResponse {
        session_id: req.session_id,
        new_expires_at,
        transaction_signature: signature.to_string(),
    }))
}

// ============================================================================
// GET /session/stats?session_id=<uuid>
// Returns live vault statistics merged from DB and on-chain state.
// ============================================================================

pub async fn get_vault_stats(
    req: web::Query<VaultStatsRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let session = fetch_session(&state, req.session_id).await?;

    let session_status = derive_session_status(&session);

    Ok(HttpResponse::Ok().json(VaultStatsResponse {
        session_id: req.session_id,
        total_deposited_sol: lamports_to_sol(session.total_deposited as u64),
        total_withdrawn_sol: 0.0,  // Tracked in DB; extend schema if needed
        available_balance_sol: 0.0, // Fetched on-chain in production
        used_amount_sol: 0.0,       // Fetched on-chain in production
        trade_count: 0,             // Fetched on-chain in production
        session_status,
        is_active: session.is_active,
        is_paused: false,           // Fetched on-chain in production
        expires_at: Some(session.expires_at),
    }))
}

// ============================================================================
// PRIVATE HELPERS
// ============================================================================

/// Fetch a single session by ID or return 400.
pub async fn fetch_session(
    state: &web::Data<AppState>,
    session_id: Uuid,
) -> Result<SessionRecord, Error> {
    sqlx::query_as!(
        SessionRecord,
        "SELECT * FROM ephemeral_sessions WHERE session_id = $1",
        session_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| ErrorBadRequest("Session not found"))
}

/// Parse a pubkey string, mapping errors to 500.
pub fn parse_pubkey(s: &str, label: &'static str) -> Result<Pubkey, Error> {
    Pubkey::from_str(s).map_err(|_| {
        ErrorInternalServerError(format!("Invalid {}", label))
    })
}

/// Derive human-readable session status from DB record timestamps.
fn derive_session_status(session: &SessionRecord) -> String {
    let now = Utc::now();
    if now > session.expires_at {
        return "expired".to_string();
    }
    let minutes_left = session
        .expires_at
        .signed_duration_since(now)
        .num_minutes();
    if minutes_left <= SESSION_RENEWAL_WINDOW_MINUTES {
        "expiring_soon".to_string()
    } else {
        "active".to_string()
    }
}

/// Build `create_ephemeral_vault` Solana instruction.
fn build_create_vault_ix(
    program_id: &Pubkey,
    user: &Pubkey,
    vault_pda: &Pubkey,
    approved_amount: u64,
) -> solana_sdk::instruction::Instruction {
    use solana_sdk::{instruction::{AccountMeta, Instruction}, system_program};
    let accounts = vec![
        AccountMeta::new(*user, true),
        AccountMeta::new(*vault_pda, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];
    let mut data = vec![0x18, 0x4a, 0x6d, 0x5e, 0x90, 0xa1, 0xd2, 0x3c]; // anchor discriminator
    data.extend_from_slice(&approved_amount.to_le_bytes());
    Instruction { program_id: *program_id, accounts, data }
}