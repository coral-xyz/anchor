#![feature(proc_macro_hygiene)]

use anchor::prelude::*;
use puppet::Puppet;

#[program]
mod puppet_master {
    use super::*;
    pub fn pull_strings(ctx: Context<PullStrings>, data: u64) -> ProgramResult {
        let cpi_ctx = ctx.accounts.into();
        puppet::cpi::set_data(cpi_ctx, data)
    }
}

#[derive(Accounts)]
pub struct PullStrings<'info> {
    #[account(mut)]
    pub puppet: CpiAccount<'info, Puppet>,
    pub puppet_program: AccountInfo<'info>,
}

impl<'a, 'b, 'c, 'info> From<&mut PullStrings<'info>>
    for CpiContext<'a, 'b, 'c, 'info, puppet::SetData<'info>>
{
    fn from(
        accounts: &mut PullStrings<'info>,
    ) -> CpiContext<'a, 'b, 'c, 'info, puppet::SetData<'info>> {
        let cpi_accounts = puppet::SetData {
            puppet: accounts.puppet.clone(),
        };
        CpiContext::new(cpi_accounts, accounts.puppet_program.clone())
    }
}
