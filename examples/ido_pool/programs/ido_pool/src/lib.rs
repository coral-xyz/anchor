use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, TokenAccount, Mint, Burn, MintTo};

// Assume that all tokens use 6 decimal places
// const DECIMALS: u32 = 6;


#[program]
pub mod ido_pool {
    use super::*;
    pub fn initialize_pool(ctx: Context<InitializePool>, num_ido_tokens: u64, nonce: u8, start_ido_ts: i64, end_deposits_ts: i64, end_ido_ts: i64) -> ProgramResult {
        // TODO make sure the pool account hasn't already been initialised

        if !(ctx.accounts.clock.unix_timestamp < start_ido_ts &&
             start_ido_ts < end_deposits_ts && 
             end_deposits_ts <= end_ido_ts) {
            return Err(ErrorCode::InitTime.into());
        }


        let pool_account = &mut ctx.accounts.pool_account;
        // TODO just use the standard struct init syntax
        pool_account.num_ido_tokens = num_ido_tokens;
        pool_account.watermelon_mint = ctx.accounts.creator_watermelon.mint;
        pool_account.usdc_mint = ctx.accounts.creator_usdc.mint;
        pool_account.nonce = nonce;
        pool_account.start_ido_ts = start_ido_ts;
        pool_account.end_deposits_ts = end_deposits_ts;
        pool_account.end_ido_ts = end_ido_ts;

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


    pub fn exchange_usdc_for_redeemable(ctx: Context<ExchangeUsdcForRedeemable>, amount: u64) -> ProgramResult {
        // msg!("Time now {}", ctx.accounts.clock.unix_timestamp);
        if !(ctx.accounts.pool_account.start_ido_ts < ctx.accounts.clock.unix_timestamp){
            return Err(ErrorCode::StartIdoTime.into());
        } else if !(ctx.accounts.clock.unix_timestamp < ctx.accounts.pool_account.end_deposits_ts) { 
            return Err(ErrorCode::EndDepositsTime.into());
        }

        // while token::transfer will check this, we prefer a verbose err msg
        if ctx.accounts.user_usdc.amount < amount {
            return Err(ErrorCode::LowUsdc.into());
        }

        // Transfer user's USDC to pool USDC account
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_usdc.to_account_info(),
            to: ctx.accounts.pool_usdc.to_account_info(),
            authority: ctx.accounts.user_authority.clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Mint Redeemable to user Redeemable account
        let seeds = &[ctx.accounts.pool_account.watermelon_mint.as_ref(), &[ctx.accounts.pool_account.nonce]];
        let signer = &[&seeds[..]];
        let cpi_accounts = MintTo {
            mint: ctx.accounts.redeemable_mint.to_account_info(),
            to: ctx.accounts.user_redeemable.to_account_info(),
            authority: ctx.accounts.pool_signer.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::mint_to(cpi_ctx , amount)?;

        Ok(())
    }


    pub fn exchange_redeemable_for_usdc(ctx: Context<ExchangeRedeemableForUsdc>, amount: u64) -> ProgramResult {
        if !(ctx.accounts.pool_account.start_ido_ts < ctx.accounts.clock.unix_timestamp){
            return Err(ErrorCode::StartIdoTime.into());
        } else if !(ctx.accounts.clock.unix_timestamp < ctx.accounts.pool_account.end_ido_ts) { 
            return Err(ErrorCode::EndIdoTime.into());
        }

        // while token::burn will check this, we prefer a verbose err msg
        if ctx.accounts.user_redeemable.amount < amount {
            return Err(ErrorCode::LowRedeemable.into());
        }

        // Burn the user's redeemable tokens
        let cpi_accounts = Burn {
            mint: ctx.accounts.redeemable_mint.to_account_info(),
            to: ctx.accounts.user_redeemable.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::burn(cpi_ctx, amount)?;

        // Transfer USDC from pool account to user
        let seeds = &[ctx.accounts.pool_account.watermelon_mint.as_ref(), &[ctx.accounts.pool_account.nonce]];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_usdc.to_account_info(),
            to: ctx.accounts.user_usdc.to_account_info(),
            authority: ctx.accounts.pool_signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }


    // pub fn calculate_exchange_rate(ctx: Context<CalculateExchangeRate>) -> ProgramResult {
    //     if !(ctx.accounts.pool_account.end_ido_ts < ctx.accounts.clock.unix_timestamp){
    //         return Err(ErrorCode::IdoNotOver.into());
    //     } 

    //     // This seems like a sensible check to make but not sure what should
    //     // happen if it actually occurs?
    //     if !(ctx.accounts.pool_usdc.amount == ctx.accounts.redeemable_mint.supply){
    //         return Err(ErrorCode::UsdcNotEqRedeem.into());
    //     }

    //     let usdc_total = ctx.accounts.pool_usdc.amount;
    //     let scaled_ido_tokens = BASE.pow(ACCURACY) * ctx.accounts.pool_watermelon.amount as u128;
    //     let exchange_rate = scaled_ido_tokens / usdc_total as u128;

    //     let pool_account = &mut ctx.accounts.pool_account;
    //     // num pool tokens deposited * exchange rate = num watermelon tokens returned
    //     pool_account.exchange_rate = exchange_rate; 
    //     pool_account.start_distribution_ts = ctx.accounts.clock.unix_timestamp;

    //     Ok(())
    // }


    pub fn exchange_redeemable_for_watermelon(ctx: Context<ExchangeRedeemableForWatermelon>, amount: u64) -> ProgramResult {
        msg!("Time now {}, vs. End time {}", ctx.accounts.clock.unix_timestamp, ctx.accounts.pool_account.end_ido_ts);
        if !(ctx.accounts.pool_account.end_ido_ts < ctx.accounts.clock.unix_timestamp ) { 
            return Err(ErrorCode::IdoNotOver.into());
        }

        msg!("Amount {} vs. Account amount {}", amount, ctx.accounts.user_redeemable.amount);
        // while token::burn will check this, we prefer a verbose err msg
        if ctx.accounts.user_redeemable.amount < amount {
            return Err(ErrorCode::LowRedeemable.into());
        }

        let watermelon_amount = (amount as u128 * ctx.accounts.pool_watermelon.amount as u128) / ctx.accounts.redeemable_mint.supply as u128; 

        // Burn the user's redeemable tokens
        let cpi_accounts = Burn {
            mint: ctx.accounts.redeemable_mint.to_account_info(),
            to: ctx.accounts.user_redeemable.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::burn(cpi_ctx, amount)?;

        // Transfer Watermelon from pool account to user
        let seeds = &[ctx.accounts.pool_account.watermelon_mint.as_ref(), &[ctx.accounts.pool_account.nonce]];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_watermelon.to_account_info(),
            to: ctx.accounts.user_watermelon.to_account_info(),
            authority: ctx.accounts.pool_signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, watermelon_amount as u64)?;

        Ok(())
    }    


    pub fn withdraw_pool_usdc(ctx: Context<WithdrawPoolUsdc>) -> ProgramResult {
        if !(ctx.accounts.pool_account.end_ido_ts < ctx.accounts.clock.unix_timestamp) { 
            return Err(ErrorCode::IdoNotOver.into());
        }

        // Transfer total USDC from pool account to creator account
        let seeds = &[ctx.accounts.pool_account.watermelon_mint.as_ref(), &[ctx.accounts.pool_account.nonce]];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_usdc.to_account_info(),
            to: ctx.accounts.creator_usdc.to_account_info(),
            authority: ctx.accounts.pool_signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, ctx.accounts.pool_usdc.amount)?;

        Ok(())
    }
}


#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init)]
    pub pool_account: ProgramAccount<'info, PoolAccount>,
    pub pool_signer: AccountInfo<'info>,
    #[account(signer)]
    pub distribution_authority: AccountInfo<'info>,
    #[account(mut)]
    pub creator_watermelon: CpiAccount<'info, TokenAccount>,
    pub creator_usdc: CpiAccount<'info, TokenAccount>,
    pub redeemable_mint: CpiAccount<'info, Mint>,
    // How can we make sure this has the right mint?
    // We can check that they both have the same mint
    #[account(mut)]
    pub pool_watermelon: CpiAccount<'info, TokenAccount>,
    pub pool_usdc: CpiAccount<'info, TokenAccount>,
    // Add a check that this is the correct token program ID
    pub token_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}


#[derive(Accounts)]
pub struct ExchangeUsdcForRedeemable<'info> {
    pub pool_account: ProgramAccount<'info, PoolAccount>,
    #[account(seeds = [pool_account.watermelon_mint.as_ref(), &[pool_account.nonce], ])]
    pool_signer: AccountInfo<'info>,
    // Check that pool signer is the owner of the mint
    #[account(mut)]
    pub redeemable_mint: CpiAccount<'info, Mint>,
    #[account(mut)]
    pub pool_usdc: CpiAccount<'info, TokenAccount>,
    #[account(signer)]
    pub user_authority: AccountInfo<'info>,
    #[account(mut)]
    pub user_usdc: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub user_redeemable: CpiAccount<'info, TokenAccount>,
    // Add a check that this is the correct token program ID
    pub token_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
}


#[derive(Accounts)]
pub struct ExchangeRedeemableForUsdc<'info> {
    pub pool_account: ProgramAccount<'info, PoolAccount>,
    #[account(seeds = [pool_account.watermelon_mint.as_ref(), &[pool_account.nonce], ])]
    pool_signer: AccountInfo<'info>,
    // Check that pool signer is the owner of the mint
    #[account(mut)]
    pub redeemable_mint: CpiAccount<'info, Mint>,
    #[account(mut)]
    pub pool_usdc: CpiAccount<'info, TokenAccount>,
    #[account(signer)]
    pub user_authority: AccountInfo<'info>,
    #[account(mut)]
    pub user_usdc: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub user_redeemable: CpiAccount<'info, TokenAccount>,
    // Add a check that this is the correct token program ID
    pub token_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
}


// #[derive(Accounts)]
// pub struct CalculateExchangeRate<'info> {
//     #[account(mut)]
//     pub pool_account: ProgramAccount<'info, PoolAccount>,
//     pub redeemable_mint: CpiAccount<'info, Mint>,
//     pub pool_usdc: CpiAccount<'info, TokenAccount>,
//     pub pool_watermelon: CpiAccount<'info, TokenAccount>,
//     pub clock: Sysvar<'info, Clock>,
// }


#[derive(Accounts)]
pub struct ExchangeRedeemableForWatermelon<'info> {
    pub pool_account: ProgramAccount<'info, PoolAccount>,
    #[account(seeds = [pool_account.watermelon_mint.as_ref(), &[pool_account.nonce], ])]
    pool_signer: AccountInfo<'info>,
    // Check that pool signer is the owner of the mint
    #[account(mut)]
    pub redeemable_mint: CpiAccount<'info, Mint>,
    #[account(mut)]
    pub pool_watermelon: CpiAccount<'info, TokenAccount>,
    #[account(signer)]
    pub user_authority: AccountInfo<'info>,
    #[account(mut)]
    pub user_watermelon: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub user_redeemable: CpiAccount<'info, TokenAccount>,
    // Add a check that this is the correct token program ID
    pub token_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct WithdrawPoolUsdc<'info> {
    pub pool_account: ProgramAccount<'info, PoolAccount>,
    pub pool_signer: AccountInfo<'info>,
    #[account(signer)]
    pub distribution_authority: AccountInfo<'info>,
    #[account(mut)]
    pub creator_usdc: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub pool_usdc: CpiAccount<'info, TokenAccount>,
    // Add a check that this is the correct token program ID
    pub token_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
}


#[account]
pub struct PoolAccount {
    pub num_ido_tokens: u64,
    pub watermelon_mint: Pubkey,
    // might not need to store usdc mint if known in advance?
    pub usdc_mint: Pubkey,
    // We're going to assume that all mint default to 6 decimal places
    // but how can we more actively check for this?
    pub nonce: u8,
    pub start_ido_ts: i64,
    pub end_deposits_ts: i64,
    pub end_ido_ts: i64,
}


#[error]
pub enum ErrorCode{
    #[msg("IDO times are non-sequential")]
    InitTime,
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
}