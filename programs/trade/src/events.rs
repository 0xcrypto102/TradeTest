use anchor_lang::prelude::*;

#[event]
pub struct InitializeEvent {
    #[index]
    pub owner: Pubkey,
    pub vault: Pubkey,
    pub token_vault: Pubkey,
    pub usdc_token_address: Pubkey,
}

#[event]
pub struct DepositSolVaultEvent {
    #[index]
    pub owner: Pubkey,
    pub deposit_amount: u64,
    pub current_balance: u64
}

#[event]
pub struct DepositUSDCVaultEvent {
    #[index]
    pub owner: Pubkey,
    pub deposit_amount: u64,
    pub current_balance: u64
}

#[event]
pub struct TradeUSDCWithSolEvent {
    #[index]
    pub user: Pubkey,
    pub deposit_sol_amount: u64,
    pub withdraw_token_amount: u64,
}

#[event]
pub struct TradeSolWithUSDCEvent {
    #[index]
    pub user: Pubkey,
    pub deposit_usdc_amount: u64,
    pub withdraw_sol_amount: u64
}