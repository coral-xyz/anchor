use anchor_lang::prelude::*;
use callee::cpi::accounts::CpiReturn;
use callee::program::Callee;
use callee::{self, CpiReturnAccount};

declare_id!("HmbTLCmaGvZhKnn1Zfa1JVnp7vkMV4DYVxPLWBVoN65L");

#[program]
pub mod caller {
    use super::*;

    pub fn cpi_call_return_u64(ctx: Context<CpiReturnU64>) -> Result<()> {
        let cpi_program = ctx.accounts.cpi_return_program.to_account_info();
        let cpi_accounts = CpiReturn {
            account: ctx.accounts.cpi_return.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        let result = callee::cpi::return_u64(cpi_ctx)?;
        let mut buffer: Vec<u8> = Vec::new();
        result.serialize(&mut buffer)?;
        msg!(&base64::encode(buffer));
        Ok(())
    }

    pub fn cpi_call_return_struct(ctx: Context<CpiReturnStruct>) -> Result<()> {
        let cpi_program = ctx.accounts.cpi_return_program.to_account_info();
        let cpi_accounts = CpiReturn {
            account: ctx.accounts.cpi_return.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        let result = callee::cpi::return_struct(cpi_ctx)?;
        let mut buffer: Vec<u8> = Vec::new();
        result.serialize(&mut buffer)?;
        msg!(&base64::encode(buffer));
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CpiReturnU64<'info> {
    #[account(mut)]
    pub cpi_return: Account<'info, CpiReturnAccount>,
    pub cpi_return_program: Program<'info, Callee>,
}

#[derive(Accounts)]
pub struct CpiReturnStruct<'info> {
    #[account(mut)]
    pub cpi_return: Account<'info, CpiReturnAccount>,
    pub cpi_return_program: Program<'info, Callee>,
}
