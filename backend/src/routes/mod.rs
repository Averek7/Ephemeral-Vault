use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use borsh::to_vec as borsh_to_vec;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::db;
use crate::error::{AppError, Result};
use crate::solana;
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/vault/:user_pubkey", get(get_vault))
        .route("/trades/:vault_pubkey", get(get_trades))
        .route("/tx/create_vault", post(tx_create_vault))
        .route("/tx/deposit", post(tx_deposit))
        .route("/tx/withdraw", post(tx_withdraw))
        .route("/tx/pause", post(tx_pause))
        .route("/tx/unpause", post(tx_unpause))
        .route("/tx/revoke", post(tx_revoke))
        .route("/tx/renew_session", post(tx_renew_session))
        .route("/tx/approve_delegate", post(tx_approve_delegate))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultAccountDto {
    pub address: String,
    pub owner: String,
    pub delegate: String,
    pub approved_amount: f64,
    pub current_balance: f64,
    pub total_deposited: f64,
    pub total_withdrawn: f64,
    pub trades_executed: u64,
    pub session_expiry: i64, // ms since epoch, 0 if none
    pub status: String,      // active | paused | revoked | expired
    pub created_at: i64,     // ms since epoch
}

async fn get_vault(
    State(state): State<AppState>,
    Path(user_pubkey): Path<String>,
) -> Result<Json<VaultAccountDto>> {
    let user = Pubkey::from_str(&user_pubkey)
        .map_err(|_| AppError::Internal("invalid user pubkey".into()))?;
    let (vault_pda, _bump) = solana::derive_vault_pda(&state.program_id, &user);

    let acct = state
        .rpc
        .get_account(&vault_pda)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("AccountNotFound") || msg.contains("could not find") {
                AppError::VaultNotFound(vault_pda.to_string())
            } else {
                AppError::SolanaRpc(msg)
            }
        })?;

    let decoded = solana::decode_ephemeral_vault_account(&acct.data)?;

    let now_ms = chrono::Utc::now().timestamp_millis();
    let session_expiry_ms = decoded
        .session_expires_at
        .map(|s| s.saturating_mul(1000))
        .unwrap_or(0);

    let status = if !decoded.is_active {
        "revoked"
    } else if decoded.is_paused {
        "paused"
    } else if session_expiry_ms > 0 && now_ms > session_expiry_ms {
        "expired"
    } else {
        "active"
    };

    let dto = VaultAccountDto {
        address: vault_pda.to_string(),
        owner: Pubkey::new_from_array(decoded.user_wallet).to_string(),
        delegate: decoded
            .delegate_wallet
            .map(|d| Pubkey::new_from_array(d).to_string())
            .unwrap_or_default(),
        approved_amount: solana::sol_from_lamports(decoded.approved_amount),
        current_balance: solana::sol_from_lamports(acct.lamports),
        total_deposited: solana::sol_from_lamports(decoded.total_deposited),
        total_withdrawn: solana::sol_from_lamports(decoded.total_withdrawn),
        trades_executed: decoded.trade_count,
        session_expiry: session_expiry_ms,
        status: status.to_string(),
        created_at: decoded.created_at.saturating_mul(1000),
    };

    Ok(Json(dto))
}

#[derive(Debug, Deserialize)]
struct TradesQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TradeDto {
    id: String,
    #[serde(rename = "type")]
    trade_type: String,
    amount: f64,
    fee: f64,
    status: String,
    timestamp: i64,
    tx_hash: String,
}

async fn get_trades(
    State(state): State<AppState>,
    Path(vault_pubkey): Path<String>,
    Query(q): Query<TradesQuery>,
) -> Result<Json<Vec<TradeDto>>> {
    Pubkey::from_str(&vault_pubkey)
        .map_err(|_| AppError::Internal("invalid vault pubkey".into()))?;

    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let offset = q.offset.unwrap_or(0).max(0);
    let rows = db::queries::get_trades_for_vault(&state.db, &vault_pubkey, limit, offset).await?;

    let out = rows
        .into_iter()
        .map(|r| TradeDto {
            id: r.id.to_string(),
            trade_type: r.trade_type,
            amount: r.amount_sol,
            fee: r.fee_sol,
            status: r.status,
            timestamp: r.created_at.timestamp_millis(),
            tx_hash: r.tx_hash,
        })
        .collect();

    Ok(Json(out))
}

// --------------------------
// Tx endpoints (unsigned tx builder)
// --------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TxResponse {
    transaction_base64: String,
    vault_pda: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TxCreateVaultRequest {
    user_pubkey: String,
    approved_amount_lamports: u64,
    delegate_pubkey: Option<String>,
    custom_duration_seconds: Option<i64>,
    initial_deposit_lamports: Option<u64>,
}

async fn tx_create_vault(
    State(state): State<AppState>,
    Json(req): Json<TxCreateVaultRequest>,
) -> Result<Json<TxResponse>> {
    let user = Pubkey::from_str(&req.user_pubkey)
        .map_err(|_| AppError::Internal("invalid user pubkey".into()))?;
    let (vault_pda, _bump) = solana::derive_vault_pda(&state.program_id, &user);

    let mut ixs = Vec::new();

    let create_args = solana::CreateEphemeralVaultArgs {
        approved_amount: req.approved_amount_lamports,
    };
    ixs.push(solana::build_anchor_instruction(
        state.program_id,
        "create_ephemeral_vault",
        vec![
            solana::meta_user_signer_writable(user),
            solana::meta_vault_writable(vault_pda),
            solana::meta_system_program(),
        ],
        borsh_to_vec(&create_args).map_err(|e| AppError::Internal(e.to_string()))?,
    ));

    if let Some(delegate_str) = req.delegate_pubkey.as_ref() {
        let delegate = Pubkey::from_str(delegate_str)
            .map_err(|_| AppError::Internal("invalid delegate pubkey".into()))?;
        let approve_args = solana::ApproveDelegateArgs {
            delegate,
            custom_duration: req.custom_duration_seconds,
        };
        ixs.push(solana::build_anchor_instruction(
            state.program_id,
            "approve_delegate",
            vec![
                solana::meta_vault_writable(vault_pda),
                solana::meta_user_signer_writable(user),
            ],
            borsh_to_vec(&approve_args).map_err(|e| AppError::Internal(e.to_string()))?,
        ));
    }

    if let Some(deposit) = req.initial_deposit_lamports {
        if deposit > 0 {
            let deposit_args = solana::AutoDepositArgs {
                trade_fee_estimate: deposit,
            };
            ixs.push(solana::build_anchor_instruction(
                state.program_id,
                "auto_deposit_for_trade",
                vec![
                    solana::meta_vault_writable(vault_pda),
                    solana::meta_user_signer_writable(user),
                    solana::meta_system_program(),
                ],
                borsh_to_vec(&deposit_args).map_err(|e| AppError::Internal(e.to_string()))?,
            ));
        }
    }

    let bh = solana::latest_blockhash(&state.rpc).await?;
    let tx_b64 = solana::build_unsigned_tx_base64(user, ixs, bh)?;
    Ok(Json(TxResponse {
        transaction_base64: tx_b64,
        vault_pda: vault_pda.to_string(),
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TxDepositRequest {
    user_pubkey: String,
    amount_lamports: u64,
}

async fn tx_deposit(
    State(state): State<AppState>,
    Json(req): Json<TxDepositRequest>,
) -> Result<Json<TxResponse>> {
    let user = Pubkey::from_str(&req.user_pubkey)
        .map_err(|_| AppError::Internal("invalid user pubkey".into()))?;
    let (vault_pda, _bump) = solana::derive_vault_pda(&state.program_id, &user);
    let args = solana::AutoDepositArgs {
        trade_fee_estimate: req.amount_lamports,
    };
    let ix = solana::build_anchor_instruction(
        state.program_id,
        "auto_deposit_for_trade",
        vec![
            solana::meta_vault_writable(vault_pda),
            solana::meta_user_signer_writable(user),
            solana::meta_system_program(),
        ],
        borsh_to_vec(&args).map_err(|e| AppError::Internal(e.to_string()))?,
    );

    let bh = solana::latest_blockhash(&state.rpc).await?;
    let tx_b64 = solana::build_unsigned_tx_base64(user, vec![ix], bh)?;
    Ok(Json(TxResponse {
        transaction_base64: tx_b64,
        vault_pda: vault_pda.to_string(),
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TxWithdrawRequest {
    user_pubkey: String,
    amount_lamports: u64, // 0 = withdraw all
}

async fn tx_withdraw(
    State(state): State<AppState>,
    Json(req): Json<TxWithdrawRequest>,
) -> Result<Json<TxResponse>> {
    let user = Pubkey::from_str(&req.user_pubkey)
        .map_err(|_| AppError::Internal("invalid user pubkey".into()))?;
    let (vault_pda, _bump) = solana::derive_vault_pda(&state.program_id, &user);
    let args = solana::WithdrawBalanceArgs {
        amount: req.amount_lamports,
    };
    let ix = solana::build_anchor_instruction(
        state.program_id,
        "withdraw_balance",
        vec![
            solana::meta_vault_writable(vault_pda),
            solana::meta_user_signer_writable(user),
        ],
        borsh_to_vec(&args).map_err(|e| AppError::Internal(e.to_string()))?,
    );

    let bh = solana::latest_blockhash(&state.rpc).await?;
    let tx_b64 = solana::build_unsigned_tx_base64(user, vec![ix], bh)?;
    Ok(Json(TxResponse {
        transaction_base64: tx_b64,
        vault_pda: vault_pda.to_string(),
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TxUserRequest {
    user_pubkey: String,
}

async fn tx_pause(
    State(state): State<AppState>,
    Json(req): Json<TxUserRequest>,
) -> Result<Json<TxResponse>> {
    let user = Pubkey::from_str(&req.user_pubkey)
        .map_err(|_| AppError::Internal("invalid user pubkey".into()))?;
    let (vault_pda, _bump) = solana::derive_vault_pda(&state.program_id, &user);
    let ix = solana::build_anchor_instruction(
        state.program_id,
        "emergency_pause",
        vec![
            solana::meta_vault_writable(vault_pda),
            solana::meta_user_signer_readonly(user),
        ],
        vec![],
    );

    let bh = solana::latest_blockhash(&state.rpc).await?;
    let tx_b64 = solana::build_unsigned_tx_base64(user, vec![ix], bh)?;
    Ok(Json(TxResponse {
        transaction_base64: tx_b64,
        vault_pda: vault_pda.to_string(),
    }))
}

async fn tx_unpause(
    State(state): State<AppState>,
    Json(req): Json<TxUserRequest>,
) -> Result<Json<TxResponse>> {
    let user = Pubkey::from_str(&req.user_pubkey)
        .map_err(|_| AppError::Internal("invalid user pubkey".into()))?;
    let (vault_pda, _bump) = solana::derive_vault_pda(&state.program_id, &user);
    let ix = solana::build_anchor_instruction(
        state.program_id,
        "unpause_vault",
        vec![
            solana::meta_vault_writable(vault_pda),
            solana::meta_user_signer_readonly(user),
        ],
        vec![],
    );

    let bh = solana::latest_blockhash(&state.rpc).await?;
    let tx_b64 = solana::build_unsigned_tx_base64(user, vec![ix], bh)?;
    Ok(Json(TxResponse {
        transaction_base64: tx_b64,
        vault_pda: vault_pda.to_string(),
    }))
}

async fn tx_revoke(
    State(state): State<AppState>,
    Json(req): Json<TxUserRequest>,
) -> Result<Json<TxResponse>> {
    let user = Pubkey::from_str(&req.user_pubkey)
        .map_err(|_| AppError::Internal("invalid user pubkey".into()))?;
    let (vault_pda, _bump) = solana::derive_vault_pda(&state.program_id, &user);
    let ix = solana::build_anchor_instruction(
        state.program_id,
        "revoke_access",
        vec![
            solana::meta_vault_writable(vault_pda),
            solana::meta_user_signer_writable(user),
        ],
        vec![],
    );

    let bh = solana::latest_blockhash(&state.rpc).await?;
    let tx_b64 = solana::build_unsigned_tx_base64(user, vec![ix], bh)?;
    Ok(Json(TxResponse {
        transaction_base64: tx_b64,
        vault_pda: vault_pda.to_string(),
    }))
}

async fn tx_renew_session(
    State(state): State<AppState>,
    Json(req): Json<TxUserRequest>,
) -> Result<Json<TxResponse>> {
    let user = Pubkey::from_str(&req.user_pubkey)
        .map_err(|_| AppError::Internal("invalid user pubkey".into()))?;
    let (vault_pda, _bump) = solana::derive_vault_pda(&state.program_id, &user);
    let ix = solana::build_anchor_instruction(
        state.program_id,
        "renew_session",
        vec![
            solana::meta_vault_writable(vault_pda),
            solana::meta_user_signer_readonly(user),
        ],
        vec![],
    );

    let bh = solana::latest_blockhash(&state.rpc).await?;
    let tx_b64 = solana::build_unsigned_tx_base64(user, vec![ix], bh)?;
    Ok(Json(TxResponse {
        transaction_base64: tx_b64,
        vault_pda: vault_pda.to_string(),
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TxApproveDelegateRequest {
    user_pubkey: String,
    delegate_pubkey: String,
    custom_duration_seconds: Option<i64>,
}

async fn tx_approve_delegate(
    State(state): State<AppState>,
    Json(req): Json<TxApproveDelegateRequest>,
) -> Result<Json<TxResponse>> {
    let user = Pubkey::from_str(&req.user_pubkey)
        .map_err(|_| AppError::Internal("invalid user pubkey".into()))?;
    let delegate = Pubkey::from_str(&req.delegate_pubkey)
        .map_err(|_| AppError::Internal("invalid delegate pubkey".into()))?;
    let (vault_pda, _bump) = solana::derive_vault_pda(&state.program_id, &user);

    let args = solana::ApproveDelegateArgs {
        delegate,
        custom_duration: req.custom_duration_seconds,
    };
    let ix = solana::build_anchor_instruction(
        state.program_id,
        "approve_delegate",
        vec![
            solana::meta_vault_writable(vault_pda),
            solana::meta_user_signer_writable(user),
        ],
        borsh_to_vec(&args).map_err(|e| AppError::Internal(e.to_string()))?,
    );

    let bh = solana::latest_blockhash(&state.rpc).await?;
    let tx_b64 = solana::build_unsigned_tx_base64(user, vec![ix], bh)?;
    Ok(Json(TxResponse {
        transaction_base64: tx_b64,
        vault_pda: vault_pda.to_string(),
    }))
}

