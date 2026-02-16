use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("FJwrtkVTxkfD7BshUx3uvpC5LKfQBqjUhunxMovqcxxA");

// Constants for security limits
const SESSION_DURATION: i64 = 3600; // 1 hour
const MAX_APPROVED_AMOUNT: u64 = 1_000_000_000_000; // 1000 SOL in lamports
const MIN_DEPOSIT_AMOUNT: u64 = 1_000_000; // 0.001 SOL minimum
const CLEANUP_REWARD_BPS: u64 = 100; // 1% in basis points

#[program]
pub mod ephemeral_vault {
    use super::*;

    pub fn create_ephemeral_vault(
        ctx: Context<CreateEphemeralVault>,
        approved_amount: u64,
    ) -> Result<()> {
        require!(
            approved_amount > 0 && approved_amount <= MAX_APPROVED_AMOUNT,
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
        vault.total_deposited = 0;
        vault.is_active = true;
        vault.bump = ctx.bumps.vault;

        emit!(VaultCreated {
            user: ctx.accounts.user.key(),
            vault_pda: vault_key,
            approved_amount,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    pub fn approve_delegate(ctx: Context<ApproveDelegate>, delegate: Pubkey) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        require!(vault.is_active, EphemeralVaultError::VaultInactive);
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

        vault.delegate_wallet = Some(delegate);
        vault.delegated_at = Some(clock.unix_timestamp);
        vault.last_activity = clock.unix_timestamp;

        emit!(DelegateApproved {
            user: ctx.accounts.user.key(),
            vault_pda: vault.key(),
            delegate,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    pub fn auto_deposit_for_trade(
        ctx: Context<AutoDeposit>,
        trade_fee_estimate: u64,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        require!(vault.is_active, EphemeralVaultError::VaultInactive);
        require_keys_eq!(
            vault.user_wallet,
            ctx.accounts.user.key(),
            EphemeralVaultError::Unauthorized
        );
        require!(
            trade_fee_estimate >= MIN_DEPOSIT_AMOUNT,
            EphemeralVaultError::DepositTooSmall
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

        // Use Anchor's CPI helper for safer transfers
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

    pub fn execute_trade(
        ctx: Context<ExecuteTrade>,
        trade_fee: u64,
        trade_amount: u64,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let delegate = &ctx.accounts.delegate;
        let clock = Clock::get()?;

        require!(vault.is_active, EphemeralVaultError::VaultInactive);

        // Verify delegate is approved
        require!(
            vault.delegate_wallet == Some(delegate.key()),
            EphemeralVaultError::Unauthorized
        );

        let delegated_at = vault
            .delegated_at
            .ok_or(EphemeralVaultError::DelegateNotProperlySet)?;

        let elapsed = clock
            .unix_timestamp
            .checked_sub(delegated_at)
            .ok_or(EphemeralVaultError::MathOverflow)?;

        if elapsed >= SESSION_DURATION {
            // Auto-revoke expired session
            vault.delegate_wallet = None;
            vault.delegated_at = None;
            return Err(EphemeralVaultError::SessionExpired.into());
        }

        require!(
            vault.available_amount >= trade_fee,
            EphemeralVaultError::InsufficientFunds
        );

        // Validate trade_amount is reasonable
        require!(
            trade_amount > 0 && trade_amount <= vault.approved_amount,
            EphemeralVaultError::InvalidTradeAmount
        );

        vault.available_amount = vault
            .available_amount
            .checked_sub(trade_fee)
            .ok_or(EphemeralVaultError::MathOverflow)?;
        vault.used_amount = vault
            .used_amount
            .checked_add(trade_amount)
            .ok_or(EphemeralVaultError::MathOverflow)?;
        vault.last_activity = clock.unix_timestamp;

        emit!(TradeExecuted {
            delegate: delegate.key(),
            vault_pda: vault.key(),
            trade_fee,
            trade_amount,
            remaining_available: vault.available_amount,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    pub fn revoke_access(ctx: Context<RevokeAccess>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        require_keys_eq!(
            vault.user_wallet,
            ctx.accounts.user.key(),
            EphemeralVaultError::Unauthorized
        );

        // Revoke delegate access
        let was_delegated = vault.delegate_wallet.is_some();
        vault.delegate_wallet = None;
        vault.delegated_at = None;

        // Return remaining SOL balance using safe Anchor transfer
        let vault_lamports = vault.to_account_info().lamports();
        let rent_exempt = Rent::get()?.minimum_balance(vault.to_account_info().data_len());

        if vault_lamports > rent_exempt {
            let transferable = vault_lamports
                .checked_sub(rent_exempt)
                .ok_or(EphemeralVaultError::MathOverflow)?;

            // Use safer transfer method
            **vault.to_account_info().try_borrow_mut_lamports()? = rent_exempt;
            **ctx
                .accounts
                .user
                .to_account_info()
                .try_borrow_mut_lamports()? = ctx
                .accounts
                .user
                .to_account_info()
                .lamports()
                .checked_add(transferable)
                .ok_or(EphemeralVaultError::MathOverflow)?;

            msg!("Returned {} lamports to user", transferable);
        }

        vault.is_active = false;
        vault.last_activity = clock.unix_timestamp;

        emit!(AccessRevoked {
            user: ctx.accounts.user.key(),
            vault_pda: vault.key(),
            was_delegated,
            returned_amount: vault_lamports.saturating_sub(rent_exempt),
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    pub fn reactivate_vault(ctx: Context<ReactivateVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        require_keys_eq!(
            vault.user_wallet,
            ctx.accounts.user.key(),
            EphemeralVaultError::Unauthorized
        );

        require!(!vault.is_active, EphemeralVaultError::VaultAlreadyActive);

        vault.delegate_wallet = None;
        vault.delegated_at = None;
        vault.is_active = true;
        vault.last_activity = Clock::get()?.unix_timestamp;

        msg!("Vault reactivated: {:?}", vault.key());

        Ok(())
    }

    pub fn cleanup_vault(ctx: Context<CleanupVault>) -> Result<()> {
        let vault = &ctx.accounts.vault;
        let clock = Clock::get()?;

        require!(!vault.is_active, EphemeralVaultError::VaultStillActive);

        let check_timestamp = vault.delegated_at.unwrap_or(vault.last_activity);
        let elapsed = clock
            .unix_timestamp
            .checked_sub(check_timestamp)
            .ok_or(EphemeralVaultError::MathOverflow)?;

        require!(
            elapsed > SESSION_DURATION,
            EphemeralVaultError::SessionNotExpired
        );

        // Calculate reward and transfer amounts
        let vault_lamports = vault.to_account_info().lamports();
        let rent_exempt = Rent::get()?.minimum_balance(vault.to_account_info().data_len());
        let available = vault_lamports
            .checked_sub(rent_exempt)
            .ok_or(EphemeralVaultError::MathOverflow)?;

        if available > 0 {
            // Calculate 1% reward with minimum threshold
            let reward = available
                .checked_mul(CLEANUP_REWARD_BPS)
                .ok_or(EphemeralVaultError::MathOverflow)?
                .checked_div(10000)
                .ok_or(EphemeralVaultError::MathOverflow)?;

            let to_user = available
                .checked_sub(reward)
                .ok_or(EphemeralVaultError::MathOverflow)?;

            // Transfer using safe method
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

            msg!("Cleanup: {} to user, {} reward to cleaner", to_user, reward);
        }

        emit!(VaultCleaned {
            cleaner: ctx.accounts.cleaner.key(),
            user_wallet: vault.user_wallet,
            vault_pda: vault.key(),
            returned_to_user: available.saturating_sub(available / 100),
            cleaner_reward: available / 100,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
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
pub struct CleanupVault<'info> {
    #[account(mut)]
    pub vault: Account<'info, EphemeralVault>,
    /// CHECK: vault.user_wallet validated from vault data
    #[account(mut, address = vault.user_wallet)]
    pub user_wallet: AccountInfo<'info>,
    #[account(mut)]
    pub cleaner: Signer<'info>,
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
    pub total_deposited: u64,
    pub is_active: bool,
    pub bump: u8,
}

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
pub struct VaultCleaned {
    pub cleaner: Pubkey,
    pub user_wallet: Pubkey,
    pub vault_pda: Pubkey,
    pub returned_to_user: u64,
    pub cleaner_reward: u64,
    pub timestamp: i64,
}

#[error_code]
pub enum EphemeralVaultError {
    #[msg("Unauthorized: only the vault owner can perform this action.")]
    Unauthorized,
    #[msg("Vault inactive or closed.")]
    VaultInactive,
    #[msg("Vault is already active.")]
    VaultAlreadyActive,
    #[msg("Vault is still active and cannot be cleaned.")]
    VaultStillActive,
    #[msg("Session has expired.")]
    SessionExpired,
    #[msg("Deposit exceeds approved limit.")]
    OverDeposit,
    #[msg("Insufficient funds in vault.")]
    InsufficientFunds,
    #[msg("Vault session not yet expired.")]
    SessionNotExpired,
    #[msg("Invalid approved amount: must be > 0 and <= max limit.")]
    InvalidApprovedAmount,
    #[msg("Deposit amount too small.")]
    DepositTooSmall,
    #[msg("Math operation overflow.")]
    MathOverflow,
    #[msg("Invalid trade amount.")]
    InvalidTradeAmount,
    #[msg("Delegate wallet not properly set.")]
    DelegateNotProperlySet,
    #[msg("Cannot delegate to self.")]
    InvalidDelegate,
}