use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, TokenAccount, Mint, Burn, MintTo};

const BASE: u128 = 10;
// Assumes that all tokens use 6 decimal places
// const DECIMALS: u32 = 6;
// Used to scale up and scale down integer calculations
const ACCURACY: u32 = 6;


#[program]
pub mod ido_pool {
    use super::*;
    pub fn initialize_pool(ctx: Context<InitializePool>, num_ido_tokens: u64, nonce: u8) -> ProgramResult {
        let pool_account = &mut ctx.accounts.pool_account;
        pool_account.num_ido_tokens = num_ido_tokens;
        pool_account.watermelon_mint = ctx.accounts.creator_watermelon.mint;
        pool_account.usdc_mint = ctx.accounts.creator_usdc.mint;
        pool_account.nonce = nonce;
        pool_account.exchange_rate = 0; // init to 0
        // assumes that all mints have 6 decimals, how to add a check for this?
        // maybe it doesn't matter?
        // pool_account.decimals = 6 ; 
        // pool_account.size_of_scale = 10;

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
        // TODO add the time checks 

        // TODO add a check that the account has a sufficient amount for the transfer

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
        // TODO add the time checks

        // TODO check the user has sufficient redeemable tokens

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


    pub fn calculate_exchange_rate(ctx: Context<CalculateExchangeRate>) -> ProgramResult {
        // TODO add time checks 

        // TODO Check that redeemable mint and usdc pool account have the same values

        let pool_account = &mut ctx.accounts.pool_account;
        // TODO check that exchange rate hasn't already been set
        // if exchange rate > 0 then error (it's already been calculated)

        
        let usdc_total = ctx.accounts.pool_usdc.amount;
        // How many decimal places should we scale up by?
        let scaled_ido_tokens = BASE.pow(ACCURACY) * ctx.accounts.pool_watermelon.amount as u128;
        let exchange_rate = scaled_ido_tokens / usdc_total as u128;
        pool_account.exchange_rate = exchange_rate; 
        // num pool tokens deposited * exchange rate = num watermelon tokens returned

        Ok(())
    }


    pub fn exchange_redeemable_for_watermelon(ctx: Context<ExchangeRedeemableForWatermelon>, amount: u64) -> ProgramResult {
        // TODO add the time checks

        // TODO check that the exchange rate has been calculated / is non-zero

        // TODO check the user has sufficient redeemable tokens

        // Burn the user's redeemable tokens
        let cpi_accounts = Burn {
            mint: ctx.accounts.redeemable_mint.to_account_info(),
            to: ctx.accounts.user_redeemable.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::burn(cpi_ctx, amount)?;

        let watermelon_amount = ((amount as u128 * ctx.accounts.pool_account.exchange_rate) / BASE.pow(ACCURACY)) as u64;
        msg!("Amount to return {}, Exchange rate {}, redeemable token amount {}", watermelon_amount, ctx.accounts.pool_account.exchange_rate, amount);
        // watermelon_amount = watermelon_amount / ACCURACY as u128;
        // TODO Need to remove 10 decimal places
        // If we add 10 decimal places to get more accuracy from the division
        // then we always need to remove 10 decimal places. We truncate, which
        // is effectively the same as flooring it. This means there may be dust
        // Probably something like 0.000001 * (num particpant accounts / 2)
        // will be left in the account after everyone has finished withdrawing

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
}


#[derive(Accounts)]
pub struct CalculateExchangeRate<'info> {
    #[account(mut)]
    pub pool_account: ProgramAccount<'info, PoolAccount>,
    pub redeemable_mint: CpiAccount<'info, Mint>,
    pub pool_usdc: CpiAccount<'info, TokenAccount>,
    pub pool_watermelon: CpiAccount<'info, TokenAccount>,
}


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
    pub exchange_rate: u128,
    // pub decimals: u8,
    // pub size_of_scale: u32,
}