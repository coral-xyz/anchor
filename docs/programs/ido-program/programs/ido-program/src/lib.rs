use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::{self, Burn, Mint, MintTo, TokenAccount, Transfer, Token};

declare_id!("GYpxvUxtesyBSn69gnbfQChoUyJ7qdsG9nXS2Y2dQNH6");

#[program]
pub mod ido_program {
    use super::*;

    #[access_control(InitializePool::accounts(&ctx, bump) pre_ido_phase(&ctx, start_ido_ts))]
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        total_native_tokens: u64, 
        start_ido_ts: i64, 
        end_ido_ts: i64,
        withdraw_fiat_ts: i64,
        bump: u8,
    ) -> Result<()> {

        if !(start_ido_ts < end_ido_ts
                && end_ido_ts < withdraw_fiat_ts)
        {
            return Err(ErrorCode::NonSequentialTimestamps.into());
        }

        if total_native_tokens == 0 {
            return Err(ErrorCode::InvalidParameter.into());
        }

        let pool = &mut ctx.accounts.pool;

        pool.pool_authority = *ctx.accounts.authority.key;
        pool.redeemable_mint = ctx.accounts.redeemable_mint.key();
        pool.native_mint = ctx.accounts.native_mint.key();
        pool.fiat_mint = ctx.accounts.fiat_mint.key();
        pool.pool_native = ctx.accounts.pool_native.key();
        pool.pool_fiat = ctx.accounts.pool_fiat.key();
        pool.total_native_tokens = total_native_tokens;
        pool.start_ido_ts = start_ido_ts;
        pool.end_ido_ts = end_ido_ts;
        pool.withdraw_fiat_ts = withdraw_fiat_ts;

        pool.bump = bump;

        //Transfer Native tokens from Creator to Pool Account
        let cpi_accounts = Transfer {
            from: ctx.accounts.creator_native.to_account_info(),
            to: ctx.accounts.pool_native.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info().clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, total_native_tokens)?;

        Ok(())
    }

    #[access_control(unrestricted_phase(&ctx))]
    pub fn exchange_fiat_for_redeemable(
        ctx: Context<ExchangeFiatForRedeemable>,
        amount: u64,
    ) -> Result<()> {
        if amount == 0 {
            return Err(ErrorCode::InvalidParameter.into());
        }
        // While token::transfer will check this, we prefer a verbose error msg
        if ctx.accounts.investor_fiat.amount < amount {
            return Err(ErrorCode::LowFiat.into());
        }
        
        // Transfer investor's fiat to pool fiat account.
        let cpi_accounts = Transfer {
            from: ctx.accounts.investor_fiat.to_account_info(),
            to: ctx.accounts.pool_fiat.to_account_info(),
            authority: ctx.accounts.authority.to_account_info().clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        
        // Mint Redeemable to investor Redeemable account.
        let seeds = &[
        ctx.accounts.pool.native_mint.as_ref(),
        &[ctx.accounts.pool.bump],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = MintTo {
            mint: ctx.accounts.redeemable_mint.to_account_info(),
            to: ctx.accounts.investor_redeemable.to_account_info(),
            authority: ctx.accounts.pool_signer.clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info().clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::mint_to(cpi_ctx, amount)?;
        
        Ok(())
    }

    #[access_control(ido_over(&ctx.accounts.pool, &ctx.accounts.clock))]
    pub fn exchange_redeemable_for_native(
        ctx: Context<ExchangeRedeemableForNative>,
    ) -> Result<()> {
        let native_amount = (ctx.accounts.investor_redeemable.amount as u128)
        .checked_mul(ctx.accounts.pool_native.amount as u128)
        .unwrap()
        .checked_div(ctx.accounts.redeemable_mint.supply as u128)
        .unwrap();

        let cpi_accounts = Burn {
            mint: ctx.accounts.redeemable_mint.to_account_info(),
            from: ctx.accounts.investor_redeemable.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info().clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::burn(cpi_ctx, ctx.accounts.investor_redeemable.amount)?;


        let seeds = &[
            ctx.accounts.pool.native_mint.as_ref(),
            &[ctx.accounts.pool.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_native.to_account_info(),
            to: ctx.accounts.investor_native.to_account_info(),
            authority: ctx.accounts.pool_signer.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info().clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::transfer(cpi_ctx, native_amount as u64)?;

        Ok(())
    }

    #[access_control(can_withdraw_fiat(&ctx.accounts.pool, &ctx.accounts.clock))]
        pub fn withdraw_pool_fiat(ctx: Context<WithdrawPoolFiat>) -> Result<()> {            
            let seeds = &[
                ctx.accounts.pool.native_mint.as_ref(),
                &[ctx.accounts.pool.bump],
            ];
            let signer = &[&seeds[..]];
            let cpi_accounts = Transfer {
                from: ctx.accounts.pool_fiat.to_account_info(),
                to: ctx.accounts.creator_fiat.to_account_info(),
                authority: ctx.accounts.pool_signer.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info().clone();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, ctx.accounts.pool_fiat.amount)?;

            Ok(())
        }
}



#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init, payer = authority, space = PoolAccount::LEN)]
    pub pool: Box<Account<'info, PoolAccount>>,

    /// CHECK: This is not dangerous
    #[account(mut)]
    pub pool_signer: AccountInfo<'info>,

    #[account(
        constraint = redeemable_mint.mint_authority == COption::Some(*pool_signer.key), 
        constraint = redeemable_mint.supply == 0
    )]
    pub redeemable_mint: Box<Account<'info, Mint>>,

    #[account(constraint = fiat_mint.decimals == redeemable_mint.decimals)]
    pub fiat_mint: Box<Account<'info, Mint>>,

    #[account(constraint = pool_native.mint == *native_mint.to_account_info().key)]
    pub native_mint: Box<Account<'info, Mint>>,

    #[account(mut, constraint = pool_native.owner == *pool_signer.key)]
    pub pool_native: Box<Account<'info, TokenAccount>>,

    #[account(constraint = pool_fiat.owner == *pool_signer.key)]
    pub pool_fiat: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub creator_native: Box<Account<'info, TokenAccount>>,

    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: Program<'info, Token>,

    pub rent: Sysvar<'info, Rent>,

    pub clock: Sysvar<'info, Clock>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitializePool<'info> {
    fn accounts(ctx: &Context<InitializePool<'info>>, bump: u8) -> Result<()> {
        let expected_signer = Pubkey::create_program_address(
            &[ctx.accounts.pool_native.mint.as_ref(), &[bump]],
            ctx.program_id,
        )
        .map_err(|_| ErrorCode::InvalidBump)?;
        if ctx.accounts.pool_signer.key != &expected_signer {
            return Err(ErrorCode::InvalidBump.into());
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct ExchangeFiatForRedeemable<'info> {
    #[account(mut, has_one = redeemable_mint, has_one = pool_fiat)]
    pub pool: Box<Account<'info, PoolAccount>>,

    ///CHECK: This is not dangerous
    #[account(seeds = [pool.native_mint.as_ref()], bump = pool.bump)]
    pool_signer: AccountInfo<'info>,

    #[account(
        mut,
        constraint = redeemable_mint.mint_authority == COption::Some(*pool_signer.key)
    )]
    pub redeemable_mint: Account<'info, Mint>,

    #[account(mut, constraint = pool_fiat.mint == *fiat_mint.to_account_info().key)]
    pub fiat_mint: Account<'info, Mint>,

    #[account(mut, constraint = pool_fiat.owner == *pool_signer.key)]
    pub pool_fiat: Account<'info, TokenAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, constraint = investor_fiat.owner == *authority.key)]
    pub investor_fiat: Account<'info, TokenAccount>,

    #[account(mut, constraint = investor_redeemable.owner == *authority.key)]
    pub investor_redeemable: Account<'info, TokenAccount>,

    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: Program<'info, Token>,

    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct ExchangeRedeemableForNative<'info> {
    #[account(has_one = redeemable_mint, has_one = pool_native)]
    pub pool: Box<Account<'info, PoolAccount>>,

    /// CHECK: This is not dangerous
    #[account(seeds = [pool.native_mint.as_ref()], bump = pool.bump)]
    pool_signer: AccountInfo<'info>,

    #[account(
        mut,
        constraint = redeemable_mint.mint_authority == COption::Some(*pool_signer.key)
    )]
    pub redeemable_mint: Account<'info, Mint>,

    #[account(mut, constraint = pool_native.owner == *pool_signer.key)]
    pub pool_native: Account<'info, TokenAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, constraint = investor_native.owner == *authority.key)]
    pub investor_native: Account<'info, TokenAccount>,

    #[account(mut, constraint = investor_redeemable.owner == *authority.key)]
    pub investor_redeemable: Account<'info, TokenAccount>,

    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: Program<'info, Token>,

    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct WithdrawPoolFiat<'info> {
    #[account(has_one = pool_fiat)]
    pub pool: Box<Account<'info, PoolAccount>>,

    ///CHECK: This is not dangerous
    #[account(seeds = [pool.native_mint.as_ref()], bump = pool.bump)]
    pub pool_signer: AccountInfo<'info>,

    #[account(mut, constraint = pool_fiat.mint == *fiat_mint.to_account_info().key)]
    pub fiat_mint: Account<'info, Mint>,

    #[account(mut, constraint = pool_fiat.owner == *pool_signer.key)]
    pub pool_fiat: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub creator_fiat: Account<'info, TokenAccount>,

    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: Program<'info, Token>,

    pub clock: Sysvar<'info, Clock>,
}

#[account]
pub struct PoolAccount {
    /// Authority of the Pool
    pub pool_authority: Pubkey,

    /// Mint of redeemable tokens (Intermediate tokens which will be exchanged for native tokens)
    pub redeemable_mint: Pubkey,
    
    /// Mint of project tokens
    pub native_mint: Pubkey,
    
    /// Mint of fiat tokens
    pub fiat_mint: Pubkey,

    /// Token Account of Pool associated with the project token mint
    pub pool_native: Pubkey,
    
    /// Token Account of Pool associated with fiat mint
    pub pool_fiat: Pubkey,

    /// Total number of native tokens being distributed
    pub total_native_tokens: u64,
    
    /// Unix timestamp for starting IDO
    pub start_ido_ts: i64,
    
    /// Unix timestamp for ending IDO
    pub end_ido_ts: i64,
    
    /// Unix timestamp for withdrawing Fiat from pool
    pub withdraw_fiat_ts: i64,

    /// Bump
    pub bump: u8,
}


impl PoolAccount {
    pub const LEN: usize = DISCRIMINATOR_LENGTH   // Discriminator Length
        + PUBKEY_LENGTH                           // Pool Authority
        + PUBKEY_LENGTH                           // Redeemable Mint
        + PUBKEY_LENGTH                           // Fiat Mint
        + PUBKEY_LENGTH                           // Pool Native Token Account
        + PUBKEY_LENGTH                           // Native Mint
        + PUBKEY_LENGTH                           // Pool fiat Token Account
        + DATA_LENGTH_64                          // Total Native Token Amount
        + DATA_LENGTH_64                          // Start IDO TS
        + DATA_LENGTH_64                          // End IDO TS
        + DATA_LENGTH_64                          // Withdraw Fiat TS
        + DATA_LENGTH_8;                          // Bump
}

const DISCRIMINATOR_LENGTH: usize = 8;
const PUBKEY_LENGTH: usize = 32;
const DATA_LENGTH_64: usize = 8;
const DATA_LENGTH_8: usize = 1;

#[error_code]
pub enum ErrorCode {
    #[msg("Timestamps are not Sequential")]
    NonSequentialTimestamps,
    #[msg("Invalid Parameter")]
    InvalidParameter,
    #[msg("Invalid Bump")]
    InvalidBump,
    #[msg("IDO has not begun yet")]
    IdoFuture,
    #[msg("Not the correct time to invest")]
    WrongInvestingTime,
    #[msg("Insufficient Fiat Tokens")]
    LowFiat,
    #[msg("IDO has not ended yet")]
    IdoNotOver,
    #[msg("Cannot withdraw Fiat yet")]
    CannotWithdrawYet
}


// Access Control Modifiers

// IDO Starts in the Future
fn pre_ido_phase<'info>(ctx: &Context<InitializePool<'info>>, start_ido_ts: i64) -> Result<()> {
    if !(ctx.accounts.clock.unix_timestamp < start_ido_ts) {
        return Err(ErrorCode::IdoFuture.into());
    }
    Ok(())
}

// Unrestricted Phase
fn unrestricted_phase<'info>(ctx: &Context<ExchangeFiatForRedeemable<'info>>) -> Result<()> {
    if !(
            ctx.accounts.pool.start_ido_ts < ctx.accounts.clock.unix_timestamp
            && 
            ctx.accounts.pool.end_ido_ts > ctx.accounts.clock.unix_timestamp
        ) {
        return Err(ErrorCode::WrongInvestingTime.into());
    }
    Ok(())
}

//iDO Over
fn ido_over<'info>(
    pool_account: &Account<'info, PoolAccount>,
    clock: &Sysvar<'info, Clock>,
) -> Result<()> {
    if !(pool_account.end_ido_ts < clock.unix_timestamp) {
        return Err(ErrorCode::IdoNotOver.into());
    }
    Ok(())
}

//Can Withdraw fiat
fn can_withdraw_fiat<'info>(
    pool_account: &Account<'info, PoolAccount>,
    clock: &Sysvar<'info, Clock>,
) -> Result<()> {
    if !(pool_account.withdraw_fiat_ts < clock.unix_timestamp) {
        return Err(ErrorCode::CannotWithdrawYet.into());
    }
    Ok(())
}