use anchor_lang::prelude::*;

#[event]
pub struct InitializeEvent {
    #[index]
    pub owner: Pubkey,
    pub vault: Pubkey,
    pub token_vault: Pubkey,
    pub usdc_token_address: Pubkey,
}
