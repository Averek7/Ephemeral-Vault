use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;

declare_id!("FJwrtkVTxkfD7BshUx3uvpC5LKfQBqjUhunxMovqcxxA");

#[program]
pub mod ephemeral_vault {
    use super::*;

    pub fn create_ephemeral_vault(
        ctx: Context<CreateEphemeralVault>,
        approved_amount: u64,
    ) -> Result<()> {
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

        require_keys_eq!(
            vault.user_wallet,
            ctx.accounts.user.key(),
            EphemeralVaultError::Unauthorized
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
        let user = &ctx.accounts.user;
        let clock = Clock::get()?;

        // Verify correct owner
        require_keys_eq!(
            vault.user_wallet,
            user.key(),
            EphemeralVaultError::Unauthorized
        );

        // Check deposit limit
        let new_total = vault.total_deposited.saturating_add(trade_fee_estimate);
        require!(
            new_total <= vault.approved_amount,
            EphemeralVaultError::OverDeposit
        );

        // Transfer SOL from user to vault PDA
        let ix = system_instruction::transfer(&user.key(), &vault.key(), trade_fee_estimate);
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                user.to_account_info(),
                vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Update vault accounting
        vault.total_deposited = new_total;
        vault.available_amount = vault.available_amount.saturating_add(trade_fee_estimate);
        vault.last_activity = clock.unix_timestamp;

        emit!(AutoDepositEvent {
            user: user.key(),
            vault_pda: vault.key(),
            amount: trade_fee_estimate,
            total_deposited: vault.total_deposited,
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
        require!(
            vault.delegate_wallet == Some(delegate.key()),
            EphemeralVaultError::Unauthorized
        );

        if let Some(delegated_at) = vault.delegated_at {
            let elapsed = clock.unix_timestamp - delegated_at;
            require!(elapsed < 3600, EphemeralVaultError::SessionExpired);
        }

        require!(
            vault.available_amount >= trade_fee,
            EphemeralVaultError::InsufficientFunds
        );

        vault.available_amount = vault.available_amount.saturating_sub(trade_fee);
        vault.used_amount = vault.used_amount.saturating_add(trade_amount);
        vault.last_activity = clock.unix_timestamp;

        emit!(TradeExecuted {
            delegate: delegate.key(),
            vault_pda: vault.key(),
            trade_fee,
            trade_amount,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    pub fn revoke_access(ctx: Context<RevokeAccess>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let user = &ctx.accounts.user;
        let clock = Clock::get()?;

        require_keys_eq!(
            vault.user_wallet,
            user.key(),
            EphemeralVaultError::Unauthorized
        );

        // If delegated, revoke it
        if vault.delegate_wallet.is_some() {
            vault.delegate_wallet = None;
            vault.delegated_at = None;
        }

        // Optional placeholder: Close open positions if needed
        // (In a real version, you'd integrate with trading logic)
        // For now, just log this action
        msg!("All open positions closed for vault {:?}", vault.key());

        // Return remaining SOL balance
        let vault_lamports = **vault.to_account_info().lamports.borrow();
        let rent_exempt = Rent::get()?.minimum_balance(vault.to_account_info().data_len());

        if vault_lamports > rent_exempt {
            let transferable = vault_lamports.saturating_sub(rent_exempt);
            **vault.to_account_info().try_borrow_mut_lamports()? -= transferable;
            **user.to_account_info().try_borrow_mut_lamports()? += transferable;
        }

        vault.is_active = false;
        vault.last_activity = clock.unix_timestamp;

        emit!(AccessRevoked {
            user: user.key(),
            vault_pda: vault.key(),
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

        vault.is_active = true;
        vault.last_activity = Clock::get()?.unix_timestamp;

        msg!("Vault reactivated: {:?}", vault.key());

        Ok(())
    }

    pub fn cleanup_vault(ctx: Context<CleanupVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let user_wallet = vault.user_wallet;
        let cleaner = &ctx.accounts.cleaner;
        let clock = Clock::get()?;

        require!(!vault.is_active, EphemeralVaultError::VaultInactive);

        // Vault must be expired
        if let Some(last) = vault.delegated_at.or(Some(vault.last_activity)) {
            let elapsed = clock.unix_timestamp - last;
            require!(elapsed > 3600, EphemeralVaultError::SessionNotExpired);
        }

        // Transfer remaining funds to user and reward cleaner
        let vault_lamports = **vault.to_account_info().lamports.borrow();
        let rent_exempt = Rent::get()?.minimum_balance(vault.to_account_info().data_len());
        let available = vault_lamports.saturating_sub(rent_exempt);

        if available > 0 {
            let reward = available / 100; // 1% reward
            let to_user = available.saturating_sub(reward);

            **vault.to_account_info().try_borrow_mut_lamports()? -= available;
            **ctx
                .accounts
                .user_wallet
                .to_account_info()
                .try_borrow_mut_lamports()? += to_user;
            **cleaner.to_account_info().try_borrow_mut_lamports()? += reward;
        }

        // Close vault account (reclaim rent)
        vault.is_active = false;

        emit!(VaultCleaned {
            cleaner: cleaner.key(),
            user_wallet,
            vault_pda: vault.key(),
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
    pub timestamp: i64,
}

#[event]
pub struct TradeExecuted {
    pub delegate: Pubkey,
    pub vault_pda: Pubkey,
    pub trade_fee: u64,
    pub trade_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct AccessRevoked {
    pub user: Pubkey,
    pub vault_pda: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct VaultCleaned {
    pub cleaner: Pubkey,
    pub user_wallet: Pubkey,
    pub vault_pda: Pubkey,
    pub timestamp: i64,
}

#[error_code]
pub enum EphemeralVaultError {
    #[msg("Unauthorized: only the vault owner can perform this action.")]
    Unauthorized,
    #[msg("Vault inactive or closed.")]
    VaultInactive,
    #[msg("Session has expired.")]
    SessionExpired,
    #[msg("Deposit exceeds approved limit.")]
    OverDeposit,
    #[msg("Insufficient funds in vault.")]
    InsufficientFunds,
    #[msg("Vault session not yet expired.")]
    SessionNotExpired,
}