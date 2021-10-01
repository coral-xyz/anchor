//! An IDO pool program implementing the Mango Markets token sale design here:
//! https://docs.mango.markets/litepaper#token-sale.
// #![warn(clippy::all)]

use anchor_lang::prelude::*;
// use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::{self, Burn, Mint, MintTo, TokenAccount, Token, Transfer};

use std::ops::Deref;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const DECIMALS: u8 = 6;

#[program]
pub mod ido_pool {
    use super::*;

    #[access_control(future_start_time(&ctx, start_ido_ts))]
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        ido_name: String,
        bumps: PoolBumps,
        num_ido_tokens: u64,
        start_ido_ts: i64,
        end_deposits_ts: i64,
        end_ido_ts: i64,
    ) -> ProgramResult {
        if !(start_ido_ts < end_deposits_ts && end_deposits_ts < end_ido_ts) {
            return Err(ErrorCode::SeqTimes.into());
        }

        let ido_account = &mut ctx.accounts.ido_account;

        let name_bytes = ido_name.as_bytes();
        let mut name_data = [b' '; 10];
        name_data[..name_bytes.len()].copy_from_slice(name_bytes);

        ido_account.ido_name = name_data;
        ido_account.bumps = bumps;
        ido_account.ido_authority = *ctx.accounts.ido_authority.key;

        ido_account.redeemable_mint = *ctx.accounts.redeemable_mint.to_account_info().key;
        ido_account.pool_watermelon = *ctx.accounts.pool_watermelon.to_account_info().key;
        ido_account.watermelon_mint = ctx.accounts.pool_watermelon.mint;
        ido_account.pool_usdc = *ctx.accounts.pool_usdc.to_account_info().key;

        ido_account.num_ido_tokens = num_ido_tokens;
        ido_account.start_ido_ts = start_ido_ts;
        ido_account.end_deposits_ts = end_deposits_ts;
        ido_account.end_ido_ts = end_ido_ts;

        // Transfer Watermelon from ido_authority to pool account.
        let cpi_accounts = Transfer {
            from: ctx.accounts.ido_authority_watermelon.to_account_info(),
            to: ctx.accounts.pool_watermelon.to_account_info(),
            authority: ctx.accounts.ido_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, num_ido_tokens)?;

        Ok(())
    }

    // TODO are there any security issues with allowing init user redeemable
    // to be called at any time? Maybe for UX reasons limit it to before the
    // end of the deposit period
    pub fn init_user_redeemable(
        _ctx: Context<InitUserRedeemable>
    ) -> ProgramResult {
        Ok(())
    }

    #[access_control(unrestricted_phase(&ctx))]
    pub fn exchange_usdc_for_redeemable(
        ctx: Context<ExchangeUsdcAndRedeemable>,
        amount: u64,
    ) -> ProgramResult {
        // While token::transfer will check this, we prefer a verbose err msg.
        if ctx.accounts.user_usdc.amount < amount {
            return Err(ErrorCode::LowUsdc.into());
        }

        // Transfer user's USDC to pool USDC account.
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_usdc.to_account_info(),
            to: ctx.accounts.pool_usdc.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Mint Redeemable to user Redeemable account.
        let ido_name = ctx.accounts.ido_account.ido_name.as_ref();
        let seeds = &[
            ido_name.trim_ascii_whitespace(),
            &[ctx.accounts.ido_account.bumps.ido_account],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = MintTo {
            mint: ctx.accounts.redeemable_mint.to_account_info(),
            to: ctx.accounts.user_redeemable.to_account_info(),
            authority: ctx.accounts.ido_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::mint_to(cpi_ctx, amount)?;

        Ok(())
    }

    #[access_control(withdraw_only_phase(&ctx))]
    pub fn exchange_redeemable_for_usdc(
        ctx: Context<ExchangeUsdcAndRedeemable>,
        amount: u64,
    ) -> ProgramResult {
        // While token::burn will check this, we prefer a verbose err msg.
        if ctx.accounts.user_redeemable.amount < amount {
            return Err(ErrorCode::LowRedeemable.into());
        }

        // Burn the user's redeemable tokens.
        let cpi_accounts = Burn {
            mint: ctx.accounts.redeemable_mint.to_account_info(),
            to: ctx.accounts.user_redeemable.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::burn(cpi_ctx, amount)?;

        // Transfer USDC from pool account to user.
        let ido_name = ctx.accounts.ido_account.ido_name.as_ref();
        let seeds = &[
            ido_name.trim_ascii_whitespace(),
            &[ctx.accounts.ido_account.bumps.ido_account],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_usdc.to_account_info(),
            to: ctx.accounts.user_usdc.to_account_info(),
            authority: ctx.accounts.ido_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    #[access_control(ido_over(&ctx.accounts.ido_account, &ctx.accounts.clock))]
    pub fn exchange_redeemable_for_watermelon(
        ctx: Context<ExchangeRedeemableForWatermelon>,
        amount: u64,
    ) -> ProgramResult {
        // While token::burn will check this, we prefer a verbose err msg.
        if ctx.accounts.user_redeemable.amount < amount {
            return Err(ErrorCode::LowRedeemable.into());
        }

        // Calculate watermelon tokens due.
        let watermelon_amount = (amount as u128)
            .checked_mul(ctx.accounts.pool_watermelon.amount as u128)
            .unwrap()
            .checked_div(ctx.accounts.redeemable_mint.supply as u128)
            .unwrap();

        // Burn the user's redeemable tokens.
        let cpi_accounts = Burn {
            mint: ctx.accounts.redeemable_mint.to_account_info(),
            to: ctx.accounts.user_redeemable.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::burn(cpi_ctx, amount)?;

        // Transfer Watermelon from pool account to user.
        let ido_name = ctx.accounts.ido_account.ido_name.as_ref();
        let seeds = &[
            ido_name.trim_ascii_whitespace(),
            &[ctx.accounts.ido_account.bumps.ido_account],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_watermelon.to_account_info(),
            to: ctx.accounts.user_watermelon.to_account_info(),
            authority: ctx.accounts.ido_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, watermelon_amount as u64)?;

        Ok(())
    }

    #[access_control(ido_over(&ctx.accounts.ido_account, &ctx.accounts.clock))]
    pub fn withdraw_pool_usdc(ctx: Context<WithdrawPoolUsdc>) -> ProgramResult {
        // Transfer total USDC from pool account to ido_authority account.
        let ido_name = ctx.accounts.ido_account.ido_name.as_ref();
        let seeds = &[
            ido_name.trim_ascii_whitespace(),
            &[ctx.accounts.ido_account.bumps.ido_account],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_usdc.to_account_info(),
            to: ctx.accounts.ido_authority_usdc.to_account_info(),
            authority: ctx.accounts.ido_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, ctx.accounts.pool_usdc.amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(ido_name: String, bumps: PoolBumps)]
pub struct InitializePool<'info> {
    // IDO Authority accounts
    #[account(mut)]
    pub ido_authority: Signer<'info>,
    #[account(mut, 
        constraint = ido_authority_watermelon.owner == *ido_authority.key)]
    pub ido_authority_watermelon: Box<Account<'info, TokenAccount>>,
    // Pool accounts
    #[account(init,
        seeds = [ido_name.as_bytes()],
        bump = bumps.ido_account,
        payer = ido_authority)]
    pub ido_account: Box<Account<'info, IdoAccount>>,
    // TODO USDC should be a known mint on mainnet so could add a check to confirm that
    #[account(constraint = usdc_mint.decimals == DECIMALS)]
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(init,
        mint::decimals = DECIMALS,
        mint::authority = ido_account,
        seeds = [ido_name.as_bytes(), b"redeemable_mint".as_ref()],
        bump = bumps.redeemable_mint,
        payer = ido_authority)]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    #[account(constraint = watermelon_mint.key() == ido_authority_watermelon.mint)]
    pub watermelon_mint: Box<Account<'info, Mint>>,
    #[account(init,
        token::mint = watermelon_mint,
        token::authority = ido_account,
        seeds = [ido_name.as_bytes(), b"pool_watermelon"],
        bump = bumps.pool_watermelon,
        payer = ido_authority)]
    pub pool_watermelon: Box<Account<'info, TokenAccount>>,
    #[account(init,
        token::mint = usdc_mint,
        token::authority = ido_account,
        seeds = [ido_name.as_bytes(), b"pool_usdc"],
        bump = bumps.pool_usdc,
        payer = ido_authority)]
    pub pool_usdc: Box<Account<'info, TokenAccount>>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct InitUserRedeemable<'info> {
    // User Accounts
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(init,
        token::mint = redeemable_mint,
        token::authority = user_authority,
        seeds = [user_authority.key.as_ref(), 
            ido_account.ido_name.as_ref().trim_ascii_whitespace(), 
            b"user_redeemable"],
        bump,
        payer = user_authority)]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,
    // Pool Accounts
    #[account(seeds = [ido_account.ido_name.as_ref().trim_ascii_whitespace()],
        bump = ido_account.bumps.ido_account)]
    pub ido_account: Box<Account<'info, IdoAccount>>,
    #[account(seeds = [ido_account.ido_name.as_ref().trim_ascii_whitespace(), b"redeemable_mint"],
        bump = ido_account.bumps.redeemable_mint)]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct ExchangeUsdcAndRedeemable<'info> {
    // User Accounts
    // #[account(mut)] TODO user shouldn't need to pay for any solana stuff?
    pub user_authority: Signer<'info>,
    // TODO replace these with the ATA constraints when possible
    #[account(mut, 
        constraint = user_usdc.owner == *user_authority.key,
        constraint = user_usdc.mint == usdc_mint.key())]
    pub user_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut,
        seeds = [user_authority.key.as_ref(),
            ido_account.ido_name.as_ref().trim_ascii_whitespace(),
            b"user_redeemable"],
        bump)]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,
    // Pool Accounts
    #[account(seeds = [ido_account.ido_name.as_ref().trim_ascii_whitespace()],
        bump = ido_account.bumps.ido_account)]
    pub ido_account: Box<Account<'info, IdoAccount>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub watermelon_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [ido_account.ido_name.as_ref().trim_ascii_whitespace(), b"redeemable_mint"],
        bump = ido_account.bumps.redeemable_mint)]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [ido_account.ido_name.as_ref().trim_ascii_whitespace(), b"pool_usdc"],
        bump = ido_account.bumps.pool_usdc)]
    pub pool_usdc: Box<Account<'info, TokenAccount>>,
    // Programs and Sysvars
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct ExchangeRedeemableForWatermelon<'info> {
    // User Accounts
    #[account(signer)]
    pub user_authority: AccountInfo<'info>,
    #[account(mut, constraint = user_watermelon.owner == *user_authority.key)]
    pub user_watermelon: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = user_redeemable.owner == *user_authority.key)]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,
    // IDO Accounts
    #[account(seeds = [ido_account.ido_name.as_ref().trim_ascii_whitespace()],
        bump = ido_account.bumps.ido_account)]
    pub ido_account: Box<Account<'info, IdoAccount>>,
    #[account(constraint = watermelon_mint.key() == user_watermelon.mint)]
    pub watermelon_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [ido_account.ido_name.as_ref().trim_ascii_whitespace(), b"redeemable_mint"],
        bump = ido_account.bumps.redeemable_mint)]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [ido_account.ido_name.as_ref().trim_ascii_whitespace(), b"pool_watermelon"],
        bump = ido_account.bumps.pool_usdc)]
    pub pool_watermelon: Box<Account<'info, TokenAccount>>,
    // Programs and Sysvars
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct WithdrawPoolUsdc<'info> {
    // Creator Accounts
    #[account(signer)]
    pub ido_authority: AccountInfo<'info>,
    #[account(mut, constraint = ido_authority_usdc.owner == *ido_authority.key)]
    pub ido_authority_usdc: Box<Account<'info, TokenAccount>>,
    // Pool Accounts
    #[account(seeds = [ido_account.ido_name.as_ref().trim_ascii_whitespace()],
        bump = ido_account.bumps.ido_account)]
    pub ido_account: Box<Account<'info, IdoAccount>>,
    pub watermelon_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [ido_account.ido_name.as_ref().trim_ascii_whitespace(), b"pool_usdc"],
        bump = ido_account.bumps.pool_usdc)]
    pub pool_usdc: Box<Account<'info, TokenAccount>>,
    // Program and Sysvars
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[account]
#[derive(Default)]
pub struct IdoAccount {
    pub ido_name: [u8; 10], // Setting arbitrary max of ten characters in the seed id
    pub bumps: PoolBumps,
    pub ido_authority: Pubkey,

    pub redeemable_mint: Pubkey,
    pub pool_watermelon: Pubkey,
    pub watermelon_mint: Pubkey,
    pub pool_usdc: Pubkey,

    pub num_ido_tokens: u64,
    pub start_ido_ts: i64,
    pub end_deposits_ts: i64,
    pub end_ido_ts: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct PoolBumps {
    pub ido_account: u8,
    pub redeemable_mint: u8,
    pub pool_watermelon: u8,
    pub pool_usdc: u8,
}

#[error]
pub enum ErrorCode {
    #[msg("IDO must start in the future")]
    IdoFuture,
    #[msg("IDO times are non-sequential")]
    SeqTimes,
    #[msg("IDO has not started")]
    StartIdoTime,
    #[msg("Deposits period has ended")]
    EndDepositsTime,
    #[msg("IDO has ended")]
    EndIdoTime,
    #[msg("IDO has not finished yet")]
    IdoNotOver,
    #[msg("Insufficient USDC")]
    LowUsdc,
    #[msg("Insufficient redeemable tokens")]
    LowRedeemable,
    #[msg("USDC total and redeemable total don't match")]
    UsdcNotEqRedeem,
    #[msg("Given nonce is invalid")]
    InvalidNonce,
}

// Access control modifiers.

// Asserts the IDO starts in the future.
fn future_start_time<'info>(ctx: &Context<InitializePool<'info>>, start_ido_ts: i64) -> ProgramResult {
    if !(ctx.accounts.clock.unix_timestamp < start_ido_ts) {
        return Err(ErrorCode::IdoFuture.into());
    }
    Ok(())
}

// Asserts the IDO is in the first phase.
fn unrestricted_phase<'info>(ctx: &Context<ExchangeUsdcAndRedeemable<'info>>) -> ProgramResult {
    if !(ctx.accounts.ido_account.start_ido_ts < ctx.accounts.clock.unix_timestamp) {
        return Err(ErrorCode::StartIdoTime.into());
    } else if !(ctx.accounts.clock.unix_timestamp < ctx.accounts.ido_account.end_deposits_ts) {
        return Err(ErrorCode::EndDepositsTime.into());
    }
    Ok(())
}

// Asserts the IDO is in the second phase.
fn withdraw_only_phase(ctx: &Context<ExchangeUsdcAndRedeemable>) -> ProgramResult {
    if !(ctx.accounts.ido_account.start_ido_ts < ctx.accounts.clock.unix_timestamp) {
        return Err(ErrorCode::StartIdoTime.into());
    } else if !(ctx.accounts.clock.unix_timestamp < ctx.accounts.ido_account.end_ido_ts) {
        return Err(ErrorCode::EndIdoTime.into());
    }
    Ok(())
}

// Asserts the IDO sale period has ended, based on the current timestamp.
fn ido_over<'info>(
    ido_account: &Account<'info, IdoAccount>,
    clock: &Sysvar<'info, Clock>,
) -> ProgramResult {
    if !(ido_account.end_ido_ts < clock.unix_timestamp) {
        return Err(ErrorCode::IdoNotOver.into());
    }
    Ok(())
}

/// Trait to allow trimming ascii whitespace from a &[u8].
pub trait TrimAsciiWhitespace {
    /// Trim ascii whitespace (based on `is_ascii_whitespace()`) from the
    /// start and end of a slice.
    fn trim_ascii_whitespace(&self) -> &[u8];
}

impl<T: Deref<Target = [u8]>> TrimAsciiWhitespace for T {
    fn trim_ascii_whitespace(&self) -> &[u8] {
        let from = match self.iter().position(|x| !x.is_ascii_whitespace()) {
            Some(i) => i,
            None => return &self[0..0],
        };
        let to = self.iter().rposition(|x| !x.is_ascii_whitespace()).unwrap();
        &self[from..=to]
    }
}
