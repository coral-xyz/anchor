use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};


#[program]
pub mod ido_pool {
    use super::*;
    pub fn initialize_pool(ctx: Context<InitializePool>, amount: u64) -> ProgramResult {
        let pool_account = &mut ctx.accounts.pool_account;
        pool_account.amount = amount;

        let cpi_accounts = Transfer {
            from: ctx.accounts.creator_watermelon.clone(),
            to: ctx.accounts.pool_watermelon.clone(),
            authority: ctx.accounts.distribution_authority.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
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
    pub creator_watermelon: AccountInfo<'info>,
    // How can we make sure this has the right mint?
    #[account(mut)]
    pub pool_watermelon: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}


#[account]
pub struct PoolAccount {
    pub amount: u64,
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
