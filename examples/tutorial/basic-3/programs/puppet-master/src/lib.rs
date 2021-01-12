#![feature(proc_macro_hygiene)]

use anchor::prelude::*;
use puppet::Puppet;

#[program]
mod puppet_master {
    use super::*;
    pub fn pull_strings(ctx: Context<PullStrings>, data: u64) -> ProgramResult {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct PullStrings {
//		pub puppet: Puppet;
}

/*
impl puppet::cpi::SetDataAccounts for PullStrings {
		pub fn accounts(&self) -> puppet::SetData {

		}
}
*/

// Usage:
//
// let client = spl_token::cpi::Transfer::accounts(ctx.accounts);
// client.transfer(...args);
//
//
// ALso need signer seed variant

/*
trait spl_token::cpi::Transfer {
		fn transfer(&self, ...args) -> Result<()> {
				let accounts = self.accounts();
				let ix = IX{...args};

		}
}

impl<'info> spl_token::cpi::Transfer<'info> for Initialize<'info> {
    pub fn accounts(&self) -> Transfer<'info> {
        Transfer {
            my_account: self.my_account.clone(),
        }
    }
}
*/
