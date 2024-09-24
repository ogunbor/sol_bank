use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

declare_id!("7p4EhrzHyxyMuDStn4rPn5x8MMtwSrrNfzjbJChtaA3Y");

#[program]
pub mod rewards_bank {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, vault_seed: u64) -> Result<()> {
        ctx.accounts.initialize_vault(vault_seed, &ctx.bumps)?;
        Ok(())
    }
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(vault_seed: u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mint::token_program = token_program)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = admin,
        space = Vault::INIT_SPACE,
        seeds = [b"vault", admin.key().as_ref(), vault_seed.to_le_bytes().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program
    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize_vault(&mut self, vault_seed: u64, bumps: &InitializeBumps) -> Result<()> {
        self.vault.set_inner(Vault {
            vault_seed,
            admin: self.admin.key(),
            mint: self.mint.key(),
            amount: 0,
            bump: bumps.vault,
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mint::token_program = token_program)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"vault", vault.admin.as_ref(), vault.vault_seed.to_le_bytes().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        associated_token::mint = vault.mint,
        associated_token::authority = admin,
        associated_token::token_program = token_program
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = vault.mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program
    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, deposit_amount: u64) -> Result<()> {
        let transfer_accounts = TransferChecked {
            from: self.user_ata.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.vault_ata.to_account_info(),
            authority: self.admin.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

        transfer_checked(cpi_ctx, deposit_amount, self.mint.decimals)?;

        self.vault.amount += deposit_amount;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub pda: Signer<'info>,
    #[account(mint::token_program = token_program)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"vault", vault.admin.as_ref(), vault.vault_seed.to_le_bytes().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        associated_token::mint = vault.mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program
    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = vault.mint,
        associated_token::authority = pda,
        associated_token::token_program = token_program
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, withdraw_amount: u64) -> Result<()> {
        if self.vault.amount < withdraw_amount {
            return Err(ProgramError::InsufficientFunds.into());
        }

        let transfer_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.user_ata.to_account_info(),
            authority: self.pda.to_account_info(),
        };

        // Convert the vault seed to a byte array and prepare the seeds
        let binding = self.vault.vault_seed.to_le_bytes();
        let seeds: &[&[u8]] = &[b"vault", self.vault.admin.as_ref(), &binding];
        let seeds: &[&[&[u8]]] = &[seeds]; // Wrap seeds in an additional array

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_accounts,
            seeds,
        );

        // Perform the token transfer
        transfer_checked(cpi_ctx, withdraw_amount, self.mint.decimals)?;

        // Update the vault's token amount
        self.vault.amount -= withdraw_amount;

        Ok(())
    }
}

#[account]
pub struct Vault {
    pub vault_seed: u64,
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub bump: u8,
}

impl Vault {
    const INIT_SPACE: usize = 8 + 8 + 32 + 32 + 8 + 1;
}
