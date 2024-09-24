use crate::errors::VaultError;
use crate::state::VaultState;
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

#[derive(Accounts)]
pub struct PrematureClose<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump,
        close = user,
    )]
    pub vault_state: Account<'info, VaultState>,
    /// CHECK: Admin's wallet is used only for transfer purposes and does not require validation.
    #[account(mut)]
    pub admin_wallet: UncheckedAccount<'info>, // Admin's wallet public key
    pub system_program: Program<'info, System>,
}

impl<'info> PrematureClose<'info> {
    pub fn premature_close(&mut self) -> Result<()> {
        // Ensure the vault is owned by the correct user
        require_keys_eq!(
            self.vault_state.user,
            *self.user.key,
            VaultError::Unauthorized
        );
        // Get the current on-chain timestamp
        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;

        require!(
            current_timestamp < self.vault_state.expiration,
            VaultError::VaultExpired
        );

        // Calculate the penalty (10%) and the remaining balance (90%)
        let penalty_amount = self.vault_state.amount / 10; // 10% penalty

        let cpi_program = self.system_program.to_account_info();

        // Transfer the penalty amount to the admin wallet
        let cpi_accounts_admin = Transfer {
            from: self.vault.to_account_info(),
            to: self.admin_wallet.to_account_info(),
        };

        let seeds = &[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx_admin =
            CpiContext::new_with_signer(cpi_program.clone(), cpi_accounts_admin, signer_seeds);

        transfer(cpi_ctx_admin, penalty_amount)?;

        // Transfer the remaining to the user and close the account
        let cpi_accounts_user = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let cpi_ctx_user =
            CpiContext::new_with_signer(cpi_program, cpi_accounts_user, signer_seeds);

        transfer(cpi_ctx_user, self.vault.to_account_info().lamports())?;

        Ok(())
    }
}
