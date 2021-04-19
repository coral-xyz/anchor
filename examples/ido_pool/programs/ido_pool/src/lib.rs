use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, TokenAccount, Mint, MintTo};


#[program]
pub mod ido_pool {
    use super::*;
    pub fn initialize_pool(ctx: Context<InitializePool>, num_ido_tokens: u64, nonce: u8) -> ProgramResult {
        let pool_account = &mut ctx.accounts.pool_account;
        pool_account.num_ido_tokens = num_ido_tokens;
        pool_account.watermelon_mint = ctx.accounts.creator_watermelon.mint;
        pool_account.usdc_mint = ctx.accounts.creator_usdc.mint;
        pool_account.nonce = nonce;

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


    pub fn exchange_usdc_for_redeemable(ctx: Context<ExchangeUsdcForRedeemable>, amount: u64) -> ProgramResult{
        // TODO add a check that the account has a sufficient amount for the transfer

        // Transfer USDC to pool USDC account
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
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init)]
    pub pool_account: ProgramAccount<'info, PoolAccount>,
    // Does this have to be an AccountInfo?
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
    pub pool_signer: AccountInfo<'info>,
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


#[account]
pub struct PoolAccount {
    pub num_ido_tokens: u64,
    pub watermelon_mint: Pubkey,
    // might not need to store usdc mint if known in advance?
    pub usdc_mint: Pubkey,
    // We're going to assume that all mint default to 6 decimal places
    // but how can we more actively check for this?
    pub nonce: u8,
}





/////  Is there anyway to make this a function we can run for more than 
/////  place, since it won't work like this
// impl<'a, 'b, 'c, 'info> From<&mut ProxyTransfer<'info>>
//     for CpiContext<'a, 'b, 'c, 'info, Transfer<'info>>
// {
//     fn from(accounts: &mut ProxyTransfer<'info>) -> CpiContext<'a, 'b, 'c, 'info, Transfer<'info>> {
//         let cpi_accounts = Transfer {
//             from: accounts.from.clone(),
//             to: accounts.to.clone(),
//             authority: accounts.authority.clone(),
//         };
//         let cpi_program = accounts.token_program.clone();
//         CpiContext::new(cpi_program, cpi_accounts)
//     }
// }
