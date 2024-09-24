use anchor_lang::prelude::*;

#[account]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
    pub expiration: i64,
    pub user: Pubkey,
    pub amount: u64,
}

impl VaultState {
    pub const INIT_SPACE: usize = 8 + 1 + 1 + 8 + 32 + 8;
}
