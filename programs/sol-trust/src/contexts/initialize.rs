use crate::errors::VaultError;
use crate::state::VaultState;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut, signer)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        seeds = [b"state", user.key().as_ref()],
        bump,
        space = VaultState::INIT_SPACE,
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        seeds = [b"vault", vault_state.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, lock_duration: i64, bumps: &InitializeBumps) -> Result<()> {
        //let min_lock_duration: i64 = 2_592_000; // 2,592,000 seconds ( 1 month )
        let min_lock_duration: i64 = 0;

        if lock_duration < min_lock_duration {
            return Err(VaultError::TimeTooShort.into());
        }

        self.vault_state.vault_bump = bumps.vault;
        self.vault_state.state_bump = bumps.vault_state;
        self.vault_state.expiration = Clock::get()?.unix_timestamp + lock_duration;
        self.vault_state.user = *self.user.key;
        self.vault_state.amount = 0;

        Ok(())
    }
}
