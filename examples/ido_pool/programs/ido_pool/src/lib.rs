use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, TokenAccount, Mint};


#[program]
pub mod ido_pool {
    use super::*;
    pub fn initialize_pool(ctx: Context<InitializePool>, num_ido_tokens: u64) -> ProgramResult {
        let pool_account = &mut ctx.accounts.pool_account;
        pool_account.num_ido_tokens = num_ido_tokens;
        pool_account.watermelon_mint = ctx.accounts.creator_watermelon.mint;
        pool_account.usdc_mint = ctx.accounts.creator_usdc.mint;

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
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init)]
    pub pool_account: ProgramAccount<'info, PoolAccount>,
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


#[account]
pub struct PoolAccount {
    pub num_ido_tokens: u64,
    pub watermelon_mint: Pubkey,
    // might not need to store usdc mint if known in advance?
    pub usdc_mint: Pubkey,
    // We're going to assume that all mint default to 6 decimal places
    // but how can we more actively check for this?
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
