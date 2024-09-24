use anchor_lang::prelude::*;

pub mod contexts;
pub mod errors;
pub mod state;
pub mod utils;

use contexts::*;

declare_id!("BBoAqxz7AfBvtkDgj2XtjG9kMmUEciSg6xmyLCJmzNGY");

#[program]
pub mod sol_trust {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, lock_duration: i64) -> Result<()> {
        ctx.accounts.initialize(lock_duration, &ctx.bumps)?;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)?;
        Ok(())
    }

    pub fn premature_close(ctx: Context<PrematureClose>) -> Result<()> {
        ctx.accounts.premature_close()?;
        Ok(())
    }

    pub fn mature_close(ctx: Context<MatureClose>) -> Result<()> {
        ctx.accounts.mature_close()?;
        Ok(())
    }
}
