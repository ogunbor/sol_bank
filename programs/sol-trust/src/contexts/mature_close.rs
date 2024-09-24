use crate::errors::VaultError;
use crate::state::VaultState;
use crate::utils::reward_calculator::calculate_reward;
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use bank_rewards::{cpi, cpi::accounts::Withdraw as BankWithdraw, program::BankRewards};
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
    #[account(mut)]
    pub bank_vault: SystemAccount<'info>, // Vault in the bank_rewards program
    #[account(mut)]
    pub bank_vault_state: Account<'info, VaultState>, // Vault state in the bank_rewards program
    #[account(address = bank_rewards::ID)] // The program ID of the bank_rewards program
    pub bank_rewards_program: Program<'info, BankRewards>,
    pub system_program: Program<'info, System>,
}
impl<'info> MatureClose<'info> {
    pub fn mature_close(&mut self) -> Result<()> {
        // Get the current on-chain timestamp
        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;
        // Ensure the vault has reached its expiration time
        require!(
            current_timestamp >= self.vault_state.expiration,
            VaultError::VaultNotYetExpired
        );
        // Calculate the reward based on vault_state information
        let reward = calculate_reward(&self.vault_state)?;
        // =====================================
        // Step 1: CPI to bank_rewards withdraw function
        // =====================================
        let cpi_program1 = self.bank_rewards_program.to_account_info();
        let cpi_accounts1 = BankWithdraw {
            user: self.user.to_account_info(), // User calling the mature_close
            vault: self.bank_vault.to_account_info(), // Vault in the bank_rewards program
            vault_state: self.bank_vault_state.to_account_info(), // Vault state in the bank_rewards program
            system_program: self.system_program.to_account_info(),
        };
        // Use signer seeds for bank vault (bank_rewards program) as a PDA signer
        let seeds1 = &[
            b"vault",
            self.bank_vault_state.to_account_info().key.as_ref(),
            &[self.bank_vault_state.vault_bump],
        ];
        let signer_seeds1 = &[&seeds1[..]];
        let cpi_ctx1 =
            CpiContext::new_with_signer(cpi_program1, cpi_accounts1.into(), signer_seeds1);
        // Call the withdraw function from bank_rewards program, withdrawing the rewards
        cpi::withdraw(cpi_ctx1, reward)?;
        // =====================================
        // Step 2: Transfer remaining SOL from vault to user
        // =====================================
        let lamports_to_transfer = self.vault.to_account_info().lamports();
        let cpi_program2 = self.system_program.to_account_info();
        let cpi_accounts2 = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };
        // Use signer seeds for vault (current program) as a PDA signer
        let seeds2 = &[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];
        let signer_seeds2 = &[&seeds2[..]];
        let cpi_ctx2 = CpiContext::new_with_signer(cpi_program2, cpi_accounts2, signer_seeds2);
        // Transfer all SOL from the vault to the user
        transfer(cpi_ctx2, lamports_to_transfer)?;
        Ok(())
    }
}
