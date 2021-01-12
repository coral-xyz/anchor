#![feature(proc_macro_hygiene)]

use anchor::prelude::*;
use puppet::Puppet;

#[program]
mod puppet_master {
    use super::*;
    pub fn pull_strings(ctx: Context<PullStrings>, data: u64) -> ProgramResult {
				//				puppet::cpi::set_data(ctx.accounts.into(), data)
				Ok(())
    }
}

#[derive(Accounts)]
pub struct PullStrings<'info> {
		pub puppet: ProgramAccount<'info, Puppet>,
}
