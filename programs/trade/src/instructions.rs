use anchor_lang::prelude::*;
use crate::{constants::*, errors::*, events::*, state::*};
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};
use pyth_sdk_solana::load_price_feed_from_account_info;
use solana_program::{program::{invoke, invoke_signed}, system_instruction};
use std::mem::size_of;
use std::str::FromStr;

const SOL_USD_FEED: &str = "J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix";

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let accts = ctx.accounts;
    
    // init the global state account
    accts.global_state.owner = accts.owner.key();
    accts.global_state.vault = accts.vault.key();
    accts.global_state.token_vault = accts.token_vault_account.key();
    accts.global_state.usdc_token_address = accts.token_mint.key();

    // Emitting the enhanced initialize event
    emit!(InitializeEvent {
        owner: accts.global_state.owner,
        vault: accts.global_state.vault,
        token_vault: accts.global_state.token_vault,
        usdc_token_address:  accts.global_state.usdc_token_address
    });
    
    Ok(())
}

pub fn deposit_sol_vault(ctx: Context<DepositSolVault>, amount: u64) -> Result<()> {
    let accts = ctx.accounts;

    require!(accts.global_state.owner == accts.owner.key(), TradeError::NotAllowedOwner);

    require!(amount > 0, TradeError::ZeroAmount);

    // depost amount sol to the vault
    invoke(
        &system_instruction::transfer(
            &accts.owner.key(),
            &accts.vault.key(),
            amount
        ),
        &[
            accts.owner.to_account_info().clone(),
            accts.vault.clone(),
            accts.system_program.to_account_info().clone(),
        ],
    )?;

    // update the sol balance of vault in global state
    accts.global_state.sol_balance += amount;

    // Emitting the deposit sol  event
    emit!(DepositSolVaultEvent {
        owner: accts.global_state.owner,
        deposit_amount: amount,
        current_balance:  accts.global_state.sol_balance
    });

    Ok(())
}

pub fn deposit_usdc_vault(ctx: Context<DepositUsdcVault>, amount: u64) -> Result<()> {
    let accts = ctx.accounts;

    require!(accts.global_state.owner == accts.owner.key(), TradeError::NotAllowedOwner);

    require!(accts.token_mint.key() == accts.global_state.usdc_token_address, TradeError::InvalidTokenAddress);

    require!(amount > 0, TradeError::ZeroAmount);

    // owner deposits USDC to vault account
  
    let cpi_ctx = CpiContext::new(
        accts.token_program.to_account_info(),
        Transfer {
            from: accts.token_owner_account.to_account_info().clone(),
            to: accts.token_vault_account.to_account_info().clone(),
            authority: accts.owner.to_account_info().clone(),
        },
    );
    transfer(cpi_ctx, amount)?;

    // update the usdc balance of vault in global state
    accts.global_state.usdc_balance += amount;

    // Emitting the deposit sol  event
    emit!(DepositUSDCVaultEvent {
        owner: accts.global_state.owner,
        deposit_amount: amount,
        current_balance:  accts.global_state.usdc_balance
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init_if_needed,
        payer = owner,
        seeds = [GLOBAL_STATE_SEED],
        bump,
        space = 8 + size_of::<GlobalState>()
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump
    )]
    /// CHECK: this should be set by admin
    pub vault: AccountInfo<'info>,  // to receive SOL

    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = owner,
        seeds = [TOKEN_VAULT_SEED, token_mint.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = global_state,
    )]
    token_vault_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DepositSolVault<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(
        mut,
        address = global_state.vault
    )]
    pub vault: AccountInfo<'info>,  // to receive SOL

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositUsdcVault<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    pub token_mint: Account<'info, Mint>,
    #[account(
        mut,
        address = global_state.token_vault
    )]
    token_vault_account: Account<'info, TokenAccount>,

    #[account(mut)]
    token_owner_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}