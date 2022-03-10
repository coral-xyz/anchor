use anchor_cpi_return::cpi::accounts::CpiReturn;
use anchor_cpi_return::program::AnchorCpiReturn;
use anchor_cpi_return::{self, CpiReturnAccount};
use anchor_lang::prelude::*;

declare_id!("HmbTLCmaGvZhKnn1Zfa1JVnp7vkMV4DYVxPLWBVoN65L");

#[program]
pub mod anchor_cpi_caller {
    use super::*;

    pub fn cpi_call_return_u64(ctx: Context<CpiReturnU64>) -> Result<()> {
        let cpi_program = ctx.accounts.cpi_return_program.to_account_info();
        let cpi_accounts = CpiReturn {
            account: ctx.accounts.cpi_return.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        let data = anchor_cpi_return::cpi::return_u64(cpi_ctx)?;
        Ok(())
    }

    pub fn cpi_call_return_struct(ctx: Context<CpiReturnStruct>) -> Result<()> {
        let cpi_program = ctx.accounts.cpi_return_program.to_account_info();
        let cpi_accounts = CpiReturn {
            account: ctx.accounts.cpi_return.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        let data = anchor_cpi_return::cpi::return_struct(cpi_ctx)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CpiReturnU64<'info> {
    #[account(mut)]
    pub cpi_return: Account<'info, CpiReturnAccount>,
    pub cpi_return_program: Program<'info, AnchorCpiReturn>,
}

#[derive(Accounts)]
pub struct CpiReturnStruct<'info> {
    #[account(mut)]
    pub cpi_return: Account<'info, CpiReturnAccount>,
    pub cpi_return_program: Program<'info, AnchorCpiReturn>,
}
