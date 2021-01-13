#![feature(proc_macro_hygiene)]

use anchor::prelude::*;
use puppet::Puppet;

#[program]
mod puppet_master {
    use super::*;
    pub fn pull_strings(ctx: Context<PullStrings>, data: u64) -> ProgramResult {
        let cpi_ctx: CpiContext<puppet::SetData> =
            CpiContext::new(ctx.accounts.into(), ctx.accounts.puppet_program.clone());
        puppet::cpi::set_data(cpi_ctx, data)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct PullStrings<'info> {
    #[account(mut)]
    pub puppet: CpiAccount<'info, Puppet>,
    pub puppet_program: AccountInfo<'info>,
}

impl<'info> From<&mut PullStrings<'info>> for puppet::SetData<'info> {
    fn from(accounts: &mut PullStrings<'info>) -> puppet::SetData<'info> {
        Self {
            puppet: accounts.puppet.clone(),
        }
    }
}
