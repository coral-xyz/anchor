use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};


#[program]
pub mod compute_cost {
    use super::*;
    pub fn transfer_please(ctx: Context<Initialize>, num_transfers: u8) -> ProgramResult {
        for _n in 1..num_transfers{
            {
                let cpi_accounts = Transfer {
                    from: ctx.accounts.from_usdc.to_account_info(),
                    to: ctx.accounts.to_usdc.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.clone();
                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
                token::transfer(cpi_ctx, 1000000 as u64)?;
            }
        };
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize <'info> {
    #[account(mut)]
    pub from_usdc: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub to_usdc: CpiAccount<'info, TokenAccount>,
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}