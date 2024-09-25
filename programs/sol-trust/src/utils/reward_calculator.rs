use crate::state::VaultState;
use anchor_lang::prelude::*;

/// Calculates the reward based on the information from `VaultState`.
pub fn calculate_reward(vault_state: &VaultState) -> Result<u64> {
    // Extract values from the vault state
    let expiration = vault_state.expiration;
    let amount = vault_state.amount;

    // Define reward rate
    let reward_rate: f64 = 0.05; // 5% base reward rate

    // Calculate the time factor based on the expiration timestamp
    let time_factor = expiration as f64;

    // Calculate the amount factor
    let amount_factor = amount as f64;

    // Calculate the reward (in SOL, scaled to lamports)
    let reward = time_factor * amount_factor * reward_rate / 1_000_000_000.0;

    // Return the reward as an integer (in lamports)
    Ok(reward as u64)
}
