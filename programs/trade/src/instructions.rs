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

pub fn buy_usdc_with_sol(ctx: Context<UserTrade>, amount: u64) -> Result<()> {
    let accts = ctx.accounts;

    require!(accts.token_mint.key() == accts.global_state.usdc_token_address, TradeError::InvalidTokenAddress);
    require!(accts.global_state.usdc_balance > 0, TradeError::NoBalance);

    let current_timestamp = accts.clock.unix_timestamp;
    // 1-Fetch sol price
    let price_sol_account_info = &accts.price_sol_feed;
    let price_sol_feed = load_price_feed_from_account_info(&price_sol_account_info).unwrap();
    let current_sol_price = price_sol_feed
        .get_price_no_older_than(current_timestamp, STALENESS_THRESHOLD)
        .unwrap();

    let sol_price = u64::try_from(current_sol_price.price).unwrap();
    let sol_price_expo = 10u64.pow(u32::try_from(-current_sol_price.expo).unwrap());

    let usdc_amount = 10000_u64 * amount / sol_price_expo * sol_price / 10000_u64;

    require!(accts.global_state.usdc_balance > usdc_amount, TradeError::Insufficientfund);

    // deposit sol first
    invoke(
        &system_instruction::transfer(
            &accts.user.key(),
            &accts.vault.key(),
            amount
        ),
        &[
            accts.user.to_account_info().clone(),
            accts.vault.clone(),
            accts.system_program.to_account_info().clone(),
        ],
    )?;

    // update the sol balance of vault in global state
    accts.global_state.sol_balance += amount;

    // get the signer as PDA
    let (_, bump) = Pubkey::find_program_address(&[GLOBAL_STATE_SEED], ctx.program_id);
    let vault_seeds = &[GLOBAL_STATE_SEED, &[bump]];
    let signer = &[&vault_seeds[..]];

    // claim usdc from usdc vault account to user
    let cpi_ctx = CpiContext::new(
        accts.token_program.to_account_info(),
        Transfer {
            from: accts.token_vault_account.to_account_info().clone(),
            to: accts.token_owner_account.to_account_info().clone(),
            authority: accts.global_state.to_account_info().clone(),
        },
    );
    transfer(
        cpi_ctx.with_signer(signer),
        usdc_amount,
    )?;

    
    // update the usdc balance of vault in global state
    accts.global_state.usdc_balance -= usdc_amount;

    emit!(TradeUSDCWithSolEvent {
        user: accts.user.key(),
        deposit_sol_amount: amount,
        withdraw_token_amount:  usdc_amount,
    });

    Ok(())
}

pub fn buy_sol_with_usdc(ctx: Context<UserTrade>, amount: u64) -> Result<()> {
    let accts = ctx.accounts;

    require!(accts.token_mint.key() == accts.global_state.usdc_token_address, TradeError::InvalidTokenAddress);
    require!(accts.global_state.sol_balance > 0, TradeError::NoBalance);

    let current_timestamp = accts.clock.unix_timestamp;
    // 1-Fetch sol price
    let price_sol_account_info = &accts.price_sol_feed;
    let price_sol_feed = load_price_feed_from_account_info(&price_sol_account_info).unwrap();
    let current_sol_price = price_sol_feed
        .get_price_no_older_than(current_timestamp, STALENESS_THRESHOLD)
        .unwrap();

    let sol_price = u64::try_from(current_sol_price.price).unwrap();
    let sol_price_expo = 10u64.pow(u32::try_from(-current_sol_price.expo).unwrap());

    let sol_amount = 10000_u64 * amount / sol_price * sol_price_expo / 10000_u64;

    require!(accts.global_state.sol_balance > sol_amount, TradeError::Insufficientfund);

    // deposit usdc to token vault
    let cpi_ctx = CpiContext::new(
        accts.token_program.to_account_info(),
        Transfer {
            from: accts.token_owner_account.to_account_info().clone(),
            to: accts.token_vault_account.to_account_info().clone(),
            authority: accts.user.to_account_info().clone(),
        },
    );
    transfer(cpi_ctx, amount)?;

    accts.global_state.usdc_balance += amount;

    // get the signer as PDA
    let (_, bump) = Pubkey::find_program_address(&[VAULT_SEED], &crate::ID);
    // withdraw sol from vault to user
    invoke_signed(
        &system_instruction::transfer(&accts.vault.key(), &accts.user.key(), sol_amount),
        &[
            accts.vault.to_account_info().clone(),
            accts.user.to_account_info().clone(),
            accts.system_program.to_account_info().clone(),
        ],
        &[&[VAULT_SEED, &[bump]]],
    )?;
    
    accts.global_state.sol_balance -= sol_amount;

    emit!(TradeSolWithUSDCEvent {
        user: accts.user.key(),
        deposit_usdc_amount: amount,
        withdraw_sol_amount:  sol_amount,
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

#[derive(Accounts)]
pub struct UserTrade<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
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
        mut,
        token::mint = token_mint,
        token::authority = global_state,
    )]
    token_vault_account: Account<'info, TokenAccount>,

    #[account(mut)]
    token_owner_account: Account<'info, TokenAccount>,

    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(address = Pubkey::from_str(SOL_USD_FEED).unwrap() @ TradeError::InvalidPriceFeed)]
    pub price_sol_feed: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}