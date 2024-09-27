use crate::errors::VaultError;
use crate::state::VaultState;
use crate::utils::reward_calculator::calculate_reward;
use anchor_lang::prelude::*;
use bank_rewards::{cpi, cpi::accounts::Withdraw, program::BankRewards};

#[derive(Accounts)]
pub struct Rewards<'info> {
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
    )]
    pub vault_state: Account<'info, VaultState>, // Reference to your VaultState struct

    #[account(address = bank_rewards::ID)] // The program ID of the bank_rewards program
    pub bank_rewards_program: Program<'info, BankRewards>,
    pub system_program: Program<'info, System>,
}
impl<'info> Rewards<'info> {
    pub fn rewards(&mut self) -> Result<()> {
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
            VaultError::NoRewardsAccess
        );
        // Calculate the reward based on vault_state information
        let reward = calculate_reward(&self.vault_state)?;

        let cpi_program = self.bank_rewards_program.to_account_info();
        let cpi_accounts = Withdraw {
            user: self.user.to_account_info(), // User calling the mature_close
            vault: self.vault.to_account_info(), // Vault in the bank_rewards program
            vault_state: self.vault_state.to_account_info(), // Vault state in the bank_rewards program
            system_program: self.system_program.to_account_info(),
        };
        // Use signer seeds for bank vault (bank_rewards program) as a PDA signer
        let seeds = &[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts.into(), signer_seeds);
        // Call the withdraw function from bank_rewards program, withdrawing the rewards
        cpi::withdraw(cpi_ctx, reward)?;

        Ok(())
    }
}
