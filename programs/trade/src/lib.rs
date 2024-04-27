use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;
use instructions::*;

declare_id!("CxHzmPBWHnpb5sFYcTde88DMEym63dybWMTH6smu3tkN");

#[program]
pub mod trade {
    use super::*;
    // owner functions
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)
    }

    pub fn deposit_sol_vault(ctx: Context<DepositSolVault>, amount: u64) -> Result<()> {
        instructions::deposit_sol_vault(ctx, amount)
    }

    pub fn deposit_usdc_vault(ctx: Context<DepositUsdcVault>, amount: u64) -> Result<()> {
        instructions::deposit_usdc_vault(ctx, amount)
    }
    // user functions
    pub fn buy_usdc_with_sol(ctx: Context<UserTrade>, amount: u64) -> Result<()> {
        instructions::buy_usdc_with_sol(ctx, amount)
    }

    pub fn buy_sol_with_usdc(ctx: Context<UserTrade>, amount: u64) -> Result<()> {
        instructions::buy_sol_with_usdc(ctx, amount)
    }
}


