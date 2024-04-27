use anchor_lang::prelude::*;
use crate::{constants::*, errors::*, events::*, state::*};
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};
use pyth_sdk_solana::load_price_feed_from_account_info;
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