use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("3L2LMJHHvgaGnvQ2ic7a5yu6DffLfoAQFLwFSjFJ4QQt");

const SESSION_DURATION: i64 = 3600; // 1 hour
const SESSION_RENEWAL_WINDOW: i64 = 300; // 5 minutes before expiry
const MAX_APPROVED_AMOUNT: u64 = 1_000_000_000_000; // 1000 SOL
const MIN_APPROVED_AMOUNT: u64 = 1_000_000; // 0.001 SOL
const MIN_DEPOSIT_AMOUNT: u64 = 1_000_000; // 0.001 SOL
const MAX_DEPOSIT_AMOUNT: u64 = 100_000_000_000; // 100 SOL per deposit
const CLEANUP_REWARD_BPS: u64 = 100; // 1%
const MIN_CLEANUP_REWARD: u64 = 100_000; // 0.0001 SOL minimum reward
const EMERGENCY_PAUSE_AUTHORITY: Pubkey = Pubkey::new_from_array([0; 32]); // Set in production

// Version for upgrade tracking
const PROGRAM_VERSION: u8 = 1;

#[program]
pub mod ephemeral_vault {
    use super::*;

    pub fn create_ephemeral_vault(
        ctx: Context<CreateEphemeralVault>,
        approved_amount: u64,
    ) -> Result<()> {
        require!(
            approved_amount >= MIN_APPROVED_AMOUNT && approved_amount <= MAX_APPROVED_AMOUNT,
            EphemeralVaultError::InvalidApprovedAmount
        );

        let clock = Clock::get()?;
        let vault_key = ctx.accounts.vault.key();
        let vault = &mut ctx.accounts.vault;

        vault.user_wallet = ctx.accounts.user.key();
        vault.vault_pda = vault_key;
        vault.created_at = clock.unix_timestamp;
        vault.last_activity = clock.unix_timestamp;
        vault.approved_amount = approved_amount;
        vault.used_amount = 0;
        vault.available_amount = 0;
        vault.delegate_wallet = None;
        vault.delegated_at = None;
        vault.session_expires_at = None;
        vault.total_deposited = 0;
        vault.total_withdrawn = 0;
        vault.trade_count = 0;
        vault.is_active = true;
        vault.is_paused = false;
        vault.version = PROGRAM_VERSION;
        vault.bump = ctx.bumps.vault;

        emit!(VaultCreated {
            user: ctx.accounts.user.key(),
            vault_pda: vault_key,
            approved_amount,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Approves a delegate wallet for trading with optional custom duration
    pub fn approve_delegate(
        ctx: Context<ApproveDelegate>,
        delegate: Pubkey,
        custom_duration: Option<i64>,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        require!(vault.is_active, EphemeralVaultError::VaultInactive);
        require!(!vault.is_paused, EphemeralVaultError::VaultPaused);
        require_keys_eq!(
            vault.user_wallet,
            ctx.accounts.user.key(),
            EphemeralVaultError::Unauthorized
        );
        require_keys_neq!(
            delegate,
            ctx.accounts.user.key(),
            EphemeralVaultError::InvalidDelegate
        );

        // Calculate session expiry
        let duration = custom_duration
            .unwrap_or(SESSION_DURATION)
            .min(SESSION_DURATION); // Cap at max duration
        
        let expires_at = clock
            .unix_timestamp
            .checked_add(duration)
            .ok_or(EphemeralVaultError::MathOverflow)?;

        vault.delegate_wallet = Some(delegate);
        vault.delegated_at = Some(clock.unix_timestamp);
        vault.session_expires_at = Some(expires_at);
        vault.last_activity = clock.unix_timestamp;

        emit!(DelegateApproved {
            user: ctx.accounts.user.key(),
            vault_pda: vault.key(),
            delegate,
            expires_at,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Renews an existing session before it expires
    pub fn renew_session(ctx: Context<RenewSession>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        require!(vault.is_active, EphemeralVaultError::VaultInactive);
        require!(!vault.is_paused, EphemeralVaultError::VaultPaused);
        require_keys_eq!(
            vault.user_wallet,
            ctx.accounts.user.key(),
            EphemeralVaultError::Unauthorized
        );

        let delegate = vault.delegate_wallet.ok_or(EphemeralVaultError::NoActiveSession)?;
        let expires_at = vault.session_expires_at.ok_or(EphemeralVaultError::NoActiveSession)?;

        // Check if within renewal window (5 minutes before expiry)
        let time_until_expiry = expires_at
            .checked_sub(clock.unix_timestamp)
            .ok_or(EphemeralVaultError::MathOverflow)?;
        
        require!(
            time_until_expiry <= SESSION_RENEWAL_WINDOW,
            EphemeralVaultError::SessionNotExpiringSoon
        );

        // Extend session
        let new_expires_at = clock
            .unix_timestamp
            .checked_add(SESSION_DURATION)
            .ok_or(EphemeralVaultError::MathOverflow)?;

        vault.session_expires_at = Some(new_expires_at);
        vault.last_activity = clock.unix_timestamp;

        emit!(SessionRenewed {
            user: ctx.accounts.user.key(),
            vault_pda: vault.key(),
            delegate,
            new_expires_at,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Deposits SOL into the vault for trading fees
    pub fn auto_deposit_for_trade(
        ctx: Context<AutoDeposit>,
        trade_fee_estimate: u64,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        require!(vault.is_active, EphemeralVaultError::VaultInactive);
        require!(!vault.is_paused, EphemeralVaultError::VaultPaused);
        require_keys_eq!(
            vault.user_wallet,
            ctx.accounts.user.key(),
            EphemeralVaultError::Unauthorized
        );
        require!(
            trade_fee_estimate >= MIN_DEPOSIT_AMOUNT,
            EphemeralVaultError::DepositTooSmall
        );
        require!(
            trade_fee_estimate <= MAX_DEPOSIT_AMOUNT,
            EphemeralVaultError::DepositTooLarge
        );

        // Check deposit limit
        let new_total = vault
            .total_deposited
            .checked_add(trade_fee_estimate)
            .ok_or(EphemeralVaultError::MathOverflow)?;
        require!(
            new_total <= vault.approved_amount,
            EphemeralVaultError::OverDeposit
        );

        // Transfer SOL using safe CPI
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: vault.to_account_info(),
            },
        );
        transfer(cpi_context, trade_fee_estimate)?;

        // Update vault accounting
        vault.total_deposited = new_total;
        vault.available_amount = vault
            .available_amount
            .checked_add(trade_fee_estimate)
            .ok_or(EphemeralVaultError::MathOverflow)?;
        vault.last_activity = clock.unix_timestamp;

        emit!(AutoDepositEvent {
            user: ctx.accounts.user.key(),
            vault_pda: vault.key(),
            amount: trade_fee_estimate,
            total_deposited: vault.total_deposited,
            available_amount: vault.available_amount,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Executes a trade using vault funds (called by delegate)
    pub fn execute_trade(
        ctx: Context<ExecuteTrade>,
        trade_fee: u64,
        trade_amount: u64,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let delegate = &ctx.accounts.delegate;
        let clock = Clock::get()?;

        require!(vault.is_active, EphemeralVaultError::VaultInactive);
        require!(!vault.is_paused, EphemeralVaultError::VaultPaused);
        
        // Verify delegate
        require!(
            vault.delegate_wallet == Some(delegate.key()),
            EphemeralVaultError::Unauthorized
        );

        // Check session expiry
        let expires_at = vault
            .session_expires_at
            .ok_or(EphemeralVaultError::DelegateNotProperlySet)?;
        
        if clock.unix_timestamp >= expires_at {
            // Auto-revoke expired session
            vault.delegate_wallet = None;
            vault.delegated_at = None;
            vault.session_expires_at = None;
            return Err(EphemeralVaultError::SessionExpired.into());
        }

        require!(
            vault.available_amount >= trade_fee,
            EphemeralVaultError::InsufficientFunds
        );

        // Validate trade_amount
        require!(
            trade_amount > 0 && trade_amount <= vault.approved_amount,
            EphemeralVaultError::InvalidTradeAmount
        );

        // Update vault state
        vault.available_amount = vault
            .available_amount
            .checked_sub(trade_fee)
            .ok_or(EphemeralVaultError::MathOverflow)?;
        vault.used_amount = vault
            .used_amount
            .checked_add(trade_amount)
            .ok_or(EphemeralVaultError::MathOverflow)?;
        vault.trade_count = vault
            .trade_count
            .checked_add(1)
            .ok_or(EphemeralVaultError::MathOverflow)?;
        vault.last_activity = clock.unix_timestamp;

        emit!(TradeExecuted {
            delegate: delegate.key(),
            vault_pda: vault.key(),
            trade_fee,
            trade_amount,
            remaining_available: vault.available_amount,
            trade_number: vault.trade_count,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Withdraws available balance back to user wallet
    pub fn withdraw_balance(
        ctx: Context<WithdrawBalance>,
        amount: u64,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        require_keys_eq!(
            vault.user_wallet,
            ctx.accounts.user.key(),
            EphemeralVaultError::Unauthorized
        );

        let vault_lamports = vault.to_account_info().lamports();
        let rent_exempt = Rent::get()?.minimum_balance(vault.to_account_info().data_len());
        let max_withdrawable = vault_lamports
            .checked_sub(rent_exempt)
            .ok_or(EphemeralVaultError::InsufficientFunds)?;

        // If amount is 0, withdraw all
        let withdraw_amount = if amount == 0 {
            max_withdrawable
        } else {
            require!(
                amount <= max_withdrawable,
                EphemeralVaultError::InsufficientFunds
            );
            amount
        };

        if withdraw_amount > 0 {
            // Safe transfer
            **vault.to_account_info().try_borrow_mut_lamports()? = vault_lamports
                .checked_sub(withdraw_amount)
                .ok_or(EphemeralVaultError::MathOverflow)?;
            
            **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? = ctx
                .accounts
                .user
                .to_account_info()
                .lamports()
                .checked_add(withdraw_amount)
                .ok_or(EphemeralVaultError::MathOverflow)?;

            vault.available_amount = vault
                .available_amount
                .checked_sub(withdraw_amount)
                .ok_or(EphemeralVaultError::MathOverflow)?;
            vault.total_withdrawn = vault
                .total_withdrawn
                .checked_add(withdraw_amount)
                .ok_or(EphemeralVaultError::MathOverflow)?;
        }

        vault.last_activity = clock.unix_timestamp;

        emit!(BalanceWithdrawn {
            user: ctx.accounts.user.key(),
            vault_pda: vault.key(),
            amount: withdraw_amount,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Revokes delegate access and returns all funds
    pub fn revoke_access(ctx: Context<RevokeAccess>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        require_keys_eq!(
            vault.user_wallet,
            ctx.accounts.user.key(),
            EphemeralVaultError::Unauthorized
        );

        // Revoke delegate
        let was_delegated = vault.delegate_wallet.is_some();
        vault.delegate_wallet = None;
        vault.delegated_at = None;
        vault.session_expires_at = None;

        // Return all available balance
        let vault_lamports = vault.to_account_info().lamports();
        let rent_exempt = Rent::get()?.minimum_balance(vault.to_account_info().data_len());
        let transferable = vault_lamports
            .checked_sub(rent_exempt)
            .unwrap_or(0);

        let returned_amount = if transferable > 0 {
            **vault.to_account_info().try_borrow_mut_lamports()? = rent_exempt;
            **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? = ctx
                .accounts
                .user
                .to_account_info()
                .lamports()
                .checked_add(transferable)
                .ok_or(EphemeralVaultError::MathOverflow)?;
            
            vault.available_amount = 0;
            vault.total_withdrawn = vault
                .total_withdrawn
                .checked_add(transferable)
                .ok_or(EphemeralVaultError::MathOverflow)?;
            
            transferable
        } else {
            0
        };

        vault.is_active = false;
        vault.last_activity = clock.unix_timestamp;

        emit!(AccessRevoked {
            user: ctx.accounts.user.key(),
            vault_pda: vault.key(),
            was_delegated,
            returned_amount,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Reactivates an inactive vault (clears delegate for security)
    pub fn reactivate_vault(ctx: Context<ReactivateVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        require_keys_eq!(
            vault.user_wallet,
            ctx.accounts.user.key(),
            EphemeralVaultError::Unauthorized
        );
        require!(!vault.is_active, EphemeralVaultError::VaultAlreadyActive);

        // Clear delegate for security
        vault.delegate_wallet = None;
        vault.delegated_at = None;
        vault.session_expires_at = None;
        vault.is_active = true;
        vault.is_paused = false;
        vault.last_activity = Clock::get()?.unix_timestamp;

        emit!(VaultReactivated {
            user: ctx.accounts.user.key(),
            vault_pda: vault.key(),
            timestamp: vault.last_activity,
        });

        Ok(())
    }

    /// Updates the approved amount for the vault
    pub fn update_approved_amount(
        ctx: Context<UpdateApprovedAmount>,
        new_approved_amount: u64,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        require_keys_eq!(
            vault.user_wallet,
            ctx.accounts.user.key(),
            EphemeralVaultError::Unauthorized
        );
        require!(
            new_approved_amount >= MIN_APPROVED_AMOUNT 
            && new_approved_amount <= MAX_APPROVED_AMOUNT,
            EphemeralVaultError::InvalidApprovedAmount
        );

        let old_amount = vault.approved_amount;
        vault.approved_amount = new_approved_amount;
        vault.last_activity = Clock::get()?.unix_timestamp;

        emit!(ApprovedAmountUpdated {
            user: ctx.accounts.user.key(),
            vault_pda: vault.key(),
            old_amount,
            new_amount: new_approved_amount,
            timestamp: vault.last_activity,
        });

        Ok(())
    }

    /// Emergency pause (can only be called by vault owner)
    pub fn emergency_pause(ctx: Context<EmergencyPause>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        require_keys_eq!(
            vault.user_wallet,
            ctx.accounts.user.key(),
            EphemeralVaultError::Unauthorized
        );

        vault.is_paused = true;
        vault.last_activity = Clock::get()?.unix_timestamp;

        emit!(VaultPaused {
            user: ctx.accounts.user.key(),
            vault_pda: vault.key(),
            timestamp: vault.last_activity,
        });

        Ok(())
    }

    /// Unpause vault
    /// 
    /// # Errors
    /// * `Unauthorized` - If caller is not vault owner
    pub fn unpause_vault(ctx: Context<UnpauseVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        require_keys_eq!(
            vault.user_wallet,
            ctx.accounts.user.key(),
            EphemeralVaultError::Unauthorized
        );

        vault.is_paused = false;
        vault.last_activity = Clock::get()?.unix_timestamp;

        emit!(VaultUnpaused {
            user: ctx.accounts.user.key(),
            vault_pda: vault.key(),
            timestamp: vault.last_activity,
        });

        Ok(())
    }

    /// Cleans up expired, inactive vaults (with reward)
    pub fn cleanup_vault(ctx: Context<CleanupVault>) -> Result<()> {
        let vault = &ctx.accounts.vault;
        let clock = Clock::get()?;

        require!(!vault.is_active, EphemeralVaultError::VaultStillActive);

        // Check expiry
        let check_timestamp = vault.session_expires_at
            .or(vault.delegated_at)
            .unwrap_or(vault.last_activity);
        
        let elapsed = clock
            .unix_timestamp
            .checked_sub(check_timestamp)
            .ok_or(EphemeralVaultError::MathOverflow)?;
        
        require!(
            elapsed > SESSION_DURATION,
            EphemeralVaultError::SessionNotExpired
        );

        // Calculate rewards
        let vault_lamports = vault.to_account_info().lamports();
        let rent_exempt = Rent::get()?.minimum_balance(vault.to_account_info().data_len());
        let available = vault_lamports
            .checked_sub(rent_exempt)
            .unwrap_or(0);

        let (to_user, reward) = if available > 0 {
            let reward = available
                .checked_mul(CLEANUP_REWARD_BPS)
                .ok_or(EphemeralVaultError::MathOverflow)?
                .checked_div(10000)
                .ok_or(EphemeralVaultError::MathOverflow)?
                .max(MIN_CLEANUP_REWARD)
                .min(available / 10); // Cap at 10%
            
            let to_user = available
                .checked_sub(reward)
                .ok_or(EphemeralVaultError::MathOverflow)?;

            // Transfer funds
            **vault.to_account_info().try_borrow_mut_lamports()? = rent_exempt;
            
            **ctx.accounts.user_wallet.try_borrow_mut_lamports()? = ctx
                .accounts
                .user_wallet
                .lamports()
                .checked_add(to_user)
                .ok_or(EphemeralVaultError::MathOverflow)?;
            
            **ctx.accounts.cleaner.try_borrow_mut_lamports()? = ctx
                .accounts
                .cleaner
                .lamports()
                .checked_add(reward)
                .ok_or(EphemeralVaultError::MathOverflow)?;

            (to_user, reward)
        } else {
            (0, 0)
        };

        emit!(VaultCleaned {
            cleaner: ctx.accounts.cleaner.key(),
            user_wallet: vault.user_wallet,
            vault_pda: vault.key(),
            returned_to_user: to_user,
            cleaner_reward: reward,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Gets vault statistics (view function)
    pub fn get_vault_stats(ctx: Context<GetVaultStats>) -> Result<VaultStats> {
        let vault = &ctx.accounts.vault;
        let clock = Clock::get()?;

        let session_status = if let Some(expires_at) = vault.session_expires_at {
            if clock.unix_timestamp >= expires_at {
                SessionStatus::Expired
            } else if expires_at - clock.unix_timestamp <= SESSION_RENEWAL_WINDOW {
                SessionStatus::ExpiringSoon
            } else {
                SessionStatus::Active
            }
        } else {
            SessionStatus::NoSession
        };

        Ok(VaultStats {
            total_deposited: vault.total_deposited,
            total_withdrawn: vault.total_withdrawn,
            available_amount: vault.available_amount,
            used_amount: vault.used_amount,
            trade_count: vault.trade_count,
            session_status,
            is_active: vault.is_active,
            is_paused: vault.is_paused,
        })
    }
}

#[derive(Accounts)]
pub struct CreateEphemeralVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<EphemeralVault>(),
        seeds = [b"vault", user.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, EphemeralVault>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApproveDelegate<'info> {
    #[account(mut)]
    pub vault: Account<'info, EphemeralVault>,
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct RenewSession<'info> {
    #[account(mut)]
    pub vault: Account<'info, EphemeralVault>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct AutoDeposit<'info> {
    #[account(mut)]
    pub vault: Account<'info, EphemeralVault>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteTrade<'info> {
    #[account(mut)]
    pub vault: Account<'info, EphemeralVault>,
    pub delegate: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawBalance<'info> {
    #[account(mut)]
    pub vault: Account<'info, EphemeralVault>,
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct RevokeAccess<'info> {
    #[account(mut)]
    pub vault: Account<'info, EphemeralVault>,
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct ReactivateVault<'info> {
    #[account(mut)]
    pub vault: Account<'info, EphemeralVault>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateApprovedAmount<'info> {
    #[account(mut)]
    pub vault: Account<'info, EphemeralVault>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct EmergencyPause<'info> {
    #[account(mut)]
    pub vault: Account<'info, EphemeralVault>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct UnpauseVault<'info> {
    #[account(mut)]
    pub vault: Account<'info, EphemeralVault>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct CleanupVault<'info> {
    #[account(mut)]
    pub vault: Account<'info, EphemeralVault>,
    /// CHECK: vault.user_wallet validated from vault data
    #[account(mut, address = vault.user_wallet)]
    pub user_wallet: AccountInfo<'info>,
    #[account(mut)]
    pub cleaner: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetVaultStats<'info> {
    pub vault: Account<'info, EphemeralVault>,
}

#[account]
pub struct EphemeralVault {
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
} // Total: ~191 bytes + discriminator (8) = ~199 bytes

#[event]
pub struct VaultCreated {
    pub user: Pubkey,
    pub vault_pda: Pubkey,
    pub approved_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct DelegateApproved {
    pub user: Pubkey,
    pub vault_pda: Pubkey,
    pub delegate: Pubkey,
    pub expires_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct SessionRenewed {
    pub user: Pubkey,
    pub vault_pda: Pubkey,
    pub delegate: Pubkey,
    pub new_expires_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct AutoDepositEvent {
    pub user: Pubkey,
    pub vault_pda: Pubkey,
    pub amount: u64,
    pub total_deposited: u64,
    pub available_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct TradeExecuted {
    pub delegate: Pubkey,
    pub vault_pda: Pubkey,
    pub trade_fee: u64,
    pub trade_amount: u64,
    pub remaining_available: u64,
    pub trade_number: u64,
    pub timestamp: i64,
}

#[event]
pub struct BalanceWithdrawn {
    pub user: Pubkey,
    pub vault_pda: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct AccessRevoked {
    pub user: Pubkey,
    pub vault_pda: Pubkey,
    pub was_delegated: bool,
    pub returned_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct VaultReactivated {
    pub user: Pubkey,
    pub vault_pda: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ApprovedAmountUpdated {
    pub user: Pubkey,
    pub vault_pda: Pubkey,
    pub old_amount: u64,
    pub new_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct VaultPaused {
    pub user: Pubkey,
    pub vault_pda: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct VaultUnpaused {
    pub user: Pubkey,
    pub vault_pda: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct VaultCleaned {
    pub cleaner: Pubkey,
    pub user_wallet: Pubkey,
    pub vault_pda: Pubkey,
    pub returned_to_user: u64,
    pub cleaner_reward: u64,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum SessionStatus {
    NoSession,
    Active,
    ExpiringSoon,
    Expired,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct VaultStats {
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub available_amount: u64,
    pub used_amount: u64,
    pub trade_count: u64,
    pub session_status: SessionStatus,
    pub is_active: bool,
    pub is_paused: bool,
}

#[error_code]
pub enum EphemeralVaultError {
    #[msg("Unauthorized: only the vault owner can perform this action")]
    Unauthorized,
    
    #[msg("Vault is inactive")]
    VaultInactive,
    
    #[msg("Vault is already active")]
    VaultAlreadyActive,
    
    #[msg("Vault is still active and cannot be cleaned")]
    VaultStillActive,
    
    #[msg("Vault is paused")]
    VaultPaused,
    
    #[msg("Session has expired")]
    SessionExpired,
    
    #[msg("No active session")]
    NoActiveSession,
    
    #[msg("Session not expiring soon - cannot renew yet")]
    SessionNotExpiringSoon,
    
    #[msg("Deposit exceeds approved limit")]
    OverDeposit,
    
    #[msg("Insufficient funds in vault")]
    InsufficientFunds,
    
    #[msg("Vault session not yet expired")]
    SessionNotExpired,
    
    #[msg("Invalid approved amount: must be between MIN and MAX")]
    InvalidApprovedAmount,
    
    #[msg("Deposit amount too small")]
    DepositTooSmall,
    
    #[msg("Deposit amount too large")]
    DepositTooLarge,
    
    #[msg("Math operation overflow")]
    MathOverflow,
    
    #[msg("Invalid trade amount")]
    InvalidTradeAmount,
    
    #[msg("Delegate wallet not properly set")]
    DelegateNotProperlySet,
    
    #[msg("Cannot delegate to self")]
    InvalidDelegate,
}