#![feature(proc_macro_hygiene)]

// #region core
use anchor::prelude::*;

#[program]
mod puppet_master {
    use super::*;
    pub fn pull_strings(ctx: Context<PullStrings>, data: u64) -> ProgramResult {
        let cpi_accounts = puppet::SetData {
            puppet: ctx.accounts.puppet.clone(),
        };
        let cpi_program = ctx.accounts.puppet_program;
        let cpi_ctx = CpiContext::new(cpi_accounts, cpi_program);
        puppet::cpi::set_data(cpi_ctx, data)
    }
}

#[derive(Accounts)]
pub struct PullStrings<'info> {
    #[account(mut)]
    pub puppet: CpiAccount<'info, puppet::Puppet>,
    pub puppet_program: AccountInfo<'info>,
}
// #endregion core
