use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct GlobalState {
    pub owner: Pubkey, // the pubkey of owner
    pub vault: Pubkey, // keep sol
    pub token_vault: Pubkey, // keep usdc token
    pub usdc_token_address: Pubkey,
}