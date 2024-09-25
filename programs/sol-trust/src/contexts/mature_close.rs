use crate::errors::VaultError;
use crate::state::VaultState;
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

#[derive(Accounts)]
pub struct MatureClose<'info> {
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
    pub system_program: Program<'info, System>,
}

impl<'info> MatureClose<'info> {
    pub fn mature_close(&mut self) -> Result<()> {
        // Ensure the vault is owned by the correct user
        require_keys_eq!(
            self.vault_state.user,
            *self.user.key,
            VaultError::Unauthorized
        );

        // Get the current on-chain timestamp
        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;
        // Ensure the vault has reached its expiration time
        require!(
            current_timestamp >= self.vault_state.expiration,
            VaultError::VaultNotYetExpired
        );

        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let seeds = &[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];

        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        // Transfer all SOL in the vault to the user
        transfer(cpi_ctx, self.vault.to_account_info().lamports())?;

        Ok(())
    }
}
