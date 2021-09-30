//! An IDO pool program implementing the Mango Markets token sale design here:
//! https://docs.mango.markets/litepaper#token-sale.

use anchor_lang::prelude::*;
// use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::{self, Burn, Mint, MintTo, TokenAccount, Transfer};

use std::ops::Deref;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const DECIMALS: u8 = 6;

#[program]
pub mod ido_pool {
    use super::*;

    // #[access_control(InitializePool::accounts(&ctx, nonce) future_start_time(&ctx, start_ido_ts))]
    #[access_control(future_start_time(&ctx, start_ido_ts))]
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        bumps: PoolBumps,
        pool_id: String,
        num_ido_tokens: u64,
        start_ido_ts: i64,
        end_deposits_ts: i64,
        end_ido_ts: i64,
    ) -> Result<()> {
        if !(start_ido_ts < end_deposits_ts && end_deposits_ts < end_ido_ts) {
            return Err(ErrorCode::SeqTimes.into());
        }

        let pool_account = &mut ctx.accounts.pool_account;

        let id_bytes = pool_id.as_bytes();
        let mut id_data = [b' '; 10];
        id_data[..id_bytes.len()].copy_from_slice(id_bytes);

        pool_account.pool_id = id_data;
        pool_account.bumps = bumps;
        pool_account.distribution_authority = *ctx.accounts.distribution_authority.key;

        pool_account.redeemable_mint = *ctx.accounts.redeemable_mint.to_account_info().key;
        pool_account.pool_watermelon = *ctx.accounts.pool_watermelon.to_account_info().key;
        pool_account.watermelon_mint = ctx.accounts.pool_watermelon.mint;
        pool_account.pool_usdc = *ctx.accounts.pool_usdc.to_account_info().key;

        pool_account.num_ido_tokens = num_ido_tokens;
        pool_account.start_ido_ts = start_ido_ts;
        pool_account.end_deposits_ts = end_deposits_ts;
        pool_account.end_ido_ts = end_ido_ts;

        // Transfer Watermelon from creator to pool account.
        let cpi_accounts = Transfer {
            from: ctx.accounts.creator_watermelon.to_account_info(),
            to: ctx.accounts.pool_watermelon.to_account_info(),
            authority: ctx.accounts.distribution_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, num_ido_tokens)?;

        Ok(())
    }

    #[access_control(unrestricted_phase(&ctx))]
    pub fn exchange_usdc_for_redeemable(
        ctx: Context<ExchangeUsdcAndRedeemable>,
        amount: u64,
    ) -> Result<()> {
        // While token::transfer will check this, we prefer a verbose err msg.
        if ctx.accounts.user_usdc.amount < amount {
            return Err(ErrorCode::LowUsdc.into());
        }

        // Transfer user's USDC to pool USDC account.
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_usdc.to_account_info(),
            to: ctx.accounts.pool_usdc.to_account_info(),
            authority: ctx.accounts.user_authority.clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Mint Redeemable to user Redeemable account.
        let pool_id = ctx.accounts.pool_account.pool_id.as_ref();
        let seeds = &[pool_id.trim_ascii_whitespace(),
            &[ctx.accounts.pool_account.bumps.pool_account],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = MintTo {
            mint: ctx.accounts.redeemable_mint.to_account_info(),
            to: ctx.accounts.user_redeemable.to_account_info(),
            authority: ctx.accounts.pool_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::mint_to(cpi_ctx, amount)?;

        Ok(())
    }

    #[access_control(withdraw_only_phase(&ctx))]
    pub fn exchange_redeemable_for_usdc(
        ctx: Context<ExchangeUsdcAndRedeemable>,
        amount: u64,
    ) -> Result<()> {
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
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::burn(cpi_ctx, amount)?;

        // Transfer USDC from pool account to user.
        let pool_id = ctx.accounts.pool_account.pool_id.as_ref();
        let seeds = &[pool_id.trim_ascii_whitespace(),
            &[ctx.accounts.pool_account.bumps.pool_account],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_usdc.to_account_info(),
            to: ctx.accounts.user_usdc.to_account_info(),
            authority: ctx.accounts.pool_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    #[access_control(ido_over(&ctx.accounts.pool_account, &ctx.accounts.clock))]
    pub fn exchange_redeemable_for_watermelon(
        ctx: Context<ExchangeRedeemableForWatermelon>,
        amount: u64,
    ) -> Result<()> {
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
        let pool_id = ctx.accounts.pool_account.pool_id.as_ref();
        let seeds = &[pool_id.trim_ascii_whitespace(),
            &[ctx.accounts.pool_account.bumps.pool_account],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_watermelon.to_account_info(),
            to: ctx.accounts.user_watermelon.to_account_info(),
            authority: ctx.accounts.pool_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, watermelon_amount as u64)?;

        Ok(())
    }

    #[access_control(ido_over(&ctx.accounts.pool_account, &ctx.accounts.clock))]
    pub fn withdraw_pool_usdc(ctx: Context<WithdrawPoolUsdc>) -> Result<()> {
        // Transfer total USDC from pool account to creator account.
        let pool_id = ctx.accounts.pool_account.pool_id.as_ref();
        let seeds = &[pool_id.trim_ascii_whitespace(),
            &[ctx.accounts.pool_account.bumps.pool_account],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_usdc.to_account_info(),
            to: ctx.accounts.creator_usdc.to_account_info(),
            authority: ctx.accounts.pool_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, ctx.accounts.pool_usdc.amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bumps: PoolBumps, pool_id: String)]
pub struct InitializePool<'info> {
    // Authority accounts
    #[account(signer)]
    pub distribution_authority: AccountInfo<'info>,
    #[account(mut, constraint = creator_watermelon.owner == *distribution_authority.key)]
    pub creator_watermelon: Box<Account<'info, TokenAccount>>,
    // Pool accounts
    #[account(init,
        seeds = [pool_id.as_bytes()],
        bump = bumps.pool_account,
        payer = distribution_authority)]
    pub pool_account: Box<Account<'info, PoolAccount>>,
    pub watermelon_mint: Box<Account<'info, Mint>>,
    #[account(init,
        mint::decimals = DECIMALS, 
        mint::authority = pool_account,
        seeds = [pool_id.as_bytes(), b"redeemable_mint".as_ref()],
        bump = bumps.redeemable_mint,
        payer = distribution_authority)]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    // TODO USDC should be a known mint on mainnet so add a check to confirm that
    #[account(constraint = usdc_mint.decimals == redeemable_mint.decimals)]
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(init,
        token::mint = watermelon_mint,
        token::authority = pool_account,
        seeds = [pool_id.as_bytes(), b"pool_watermelon"],
        bump = bumps.pool_watermelon,
        payer = distribution_authority)]
    pub pool_watermelon: Box<Account<'info, TokenAccount>>,
    #[account(init,
        token::mint = usdc_mint,
        token::authority = pool_account,
        seeds = [pool_id.as_bytes(), b"pool_usdc"],
        bump = bumps.pool_usdc,
        payer = distribution_authority)]
    pub pool_usdc: Box<Account<'info, TokenAccount>>,
    pub system_program: AccountInfo<'info>,
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

// impl<'info> InitializePool<'info> {
//     fn accounts(ctx: &Context<InitializePool<'info>>, nonce: u8) -> Result<()> {
//         let expected_signer = Pubkey::create_program_address(
//             &[ctx.accounts.pool_watermelon.mint.as_ref(), &[nonce]],
//             ctx.program_id,
//         )
//         .map_err(|_| ErrorCode::InvalidNonce)?;
//         if ctx.accounts.pool_signer.key != &expected_signer {
//             return Err(ErrorCode::InvalidNonce.into());
//         }
//         Ok(())
//     }
// }

#[derive(Accounts)]
pub struct ExchangeUsdcAndRedeemable<'info> {
    // User Accounts
    #[account(signer)]
    pub user_authority: AccountInfo<'info>,
    #[account(mut, constraint = user_usdc.owner == *user_authority.key)]
    pub user_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = user_redeemable.owner == *user_authority.key)]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,
    // Pool Accounts
    #[account(seeds = [pool_account.pool_id.as_ref().trim_ascii_whitespace()],
        bump = pool_account.bumps.pool_account)]
    pub pool_account: Box<Account<'info, PoolAccount>>,
    pub watermelon_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [pool_account.pool_id.as_ref().trim_ascii_whitespace(), b"redeemable_mint"],
        bump = pool_account.bumps.redeemable_mint)]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [pool_account.pool_id.as_ref().trim_ascii_whitespace(), b"pool_usdc"],
        bump = pool_account.bumps.pool_usdc)]
    pub pool_usdc: Box<Account<'info, TokenAccount>>,
    // Programs and Sysvars
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
}

// #[derive(Accounts)]
// pub struct ExchangeUsdcAndRedeemable<'info> {
//     #[account(has_one = redeemable_mint, has_one = pool_usdc)]
//     pub pool_account: Box<Account<'info, PoolAccount>>,
//     #[account(
//         seeds = [pool_account.watermelon_mint.as_ref()],
//         bump = pool_account.nonce,
//     )]
//     pool_signer: AccountInfo<'info>,
//     pub watermelon_mint: Box<Account<'info, Mint>>,
//     #[account(
//         mut,
//         constraint = redeemable_mint.mint_authority == COption::Some(*pool_signer.key)
//     )]
//     pub redeemable_mint: Box<Account<'info, Mint>>,
//     #[account(mut, constraint = pool_usdc.owner == *pool_signer.key)]
//     pub pool_usdc: Box<Account<'info, TokenAccount>>,
//     #[account(signer)]
//     pub user_authority: AccountInfo<'info>,
//     #[account(mut, constraint = user_usdc.owner == *user_authority.key)]
//     pub user_usdc: Box<Account<'info, TokenAccount>>,
//     #[account(mut, constraint = user_redeemable.owner == *user_authority.key)]
//     pub user_redeemable: Box<Account<'info, TokenAccount>>,
//     #[account(constraint = token_program.key == &token::ID)]
//     pub token_program: AccountInfo<'info>,
//     pub clock: Sysvar<'info, Clock>,
// }

#[derive(Accounts)]
pub struct ExchangeRedeemableForWatermelon<'info> {
    // User Accounts
    #[account(signer)]
    pub user_authority: AccountInfo<'info>,
    #[account(mut, constraint = user_watermelon.owner == *user_authority.key)]
    pub user_watermelon: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = user_redeemable.owner == *user_authority.key)]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,
    // Pool Accounts
    #[account(seeds = [pool_account.pool_id.as_ref().trim_ascii_whitespace()],
        bump = pool_account.bumps.pool_account)]
    pub pool_account: Box<Account<'info, PoolAccount>>,
    pub watermelon_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [pool_account.pool_id.as_ref().trim_ascii_whitespace(), b"redeemable_mint"],
        bump = pool_account.bumps.redeemable_mint)]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [pool_account.pool_id.as_ref().trim_ascii_whitespace(), b"pool_watermelon"],
        bump = pool_account.bumps.pool_usdc)]
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
    pub distribution_authority: AccountInfo<'info>,
    #[account(mut, constraint = creator_usdc.owner == *distribution_authority.key)]
    pub creator_usdc: Box<Account<'info, TokenAccount>>,
    // Pool Accounts
    #[account(seeds = [pool_account.pool_id.as_ref().trim_ascii_whitespace()],
        bump = pool_account.bumps.pool_account)]
    pub pool_account: Box<Account<'info, PoolAccount>>,
    pub watermelon_mint: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [pool_account.pool_id.as_ref().trim_ascii_whitespace(), b"pool_usdc"],
        bump = pool_account.bumps.pool_usdc)]
    pub pool_usdc: Box<Account<'info, TokenAccount>>,
    // Program and Sysvars
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[account]
#[derive(Default)]
pub struct PoolAccount {
    pub pool_id: [u8; 10], // Setting arbitrary max of ten characters in the seed id
    pub bumps: PoolBumps,
    pub distribution_authority: Pubkey,

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
    pub pool_account: u8,
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
fn future_start_time<'info>(ctx: &Context<InitializePool<'info>>, start_ido_ts: i64) -> Result<()> {
    if !(ctx.accounts.clock.unix_timestamp < start_ido_ts) {
        return Err(ErrorCode::IdoFuture.into());
    }
    Ok(())
}

// Asserts the IDO is in the first phase.
fn unrestricted_phase<'info>(ctx: &Context<ExchangeUsdcAndRedeemable<'info>>) -> Result<()> {
    if !(ctx.accounts.pool_account.start_ido_ts < ctx.accounts.clock.unix_timestamp) {
        return Err(ErrorCode::StartIdoTime.into());
    } else if !(ctx.accounts.clock.unix_timestamp < ctx.accounts.pool_account.end_deposits_ts) {
        return Err(ErrorCode::EndDepositsTime.into());
    }
    Ok(())
}

// Asserts the IDO is in the second phase.
fn withdraw_only_phase(ctx: &Context<ExchangeUsdcAndRedeemable>) -> Result<()> {
    if !(ctx.accounts.pool_account.start_ido_ts < ctx.accounts.clock.unix_timestamp) {
        return Err(ErrorCode::StartIdoTime.into());
    } else if !(ctx.accounts.clock.unix_timestamp < ctx.accounts.pool_account.end_ido_ts) {
        return Err(ErrorCode::EndIdoTime.into());
    }
    Ok(())
}

// Asserts the IDO sale period has ended, based on the current timestamp.
fn ido_over<'info>(
    pool_account: &Account<'info, PoolAccount>,
    clock: &Sysvar<'info, Clock>,
) -> Result<()> {
    if !(pool_account.end_ido_ts < clock.unix_timestamp) {
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