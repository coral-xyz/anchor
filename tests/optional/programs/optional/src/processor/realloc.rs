use anchor_lang::prelude::*;

use crate::state::*;
use crate::OptionalErrors;

#[derive(Accounts)]
pub struct Realloc<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, realloc = 20, realloc::payer = payer, realloc::zero = false)]
    pub optional_1: Option<Account<'info, Data1>>,
    pub required: Account<'info, Data2>,
    pub system_program: Option<Program<'info, System>>,
}

pub fn handle_realloc(ctx: Context<Realloc>) -> Result<()> {
    let optional = &ctx.accounts.optional_1;
    if let Some(acc) = optional {
        let len = acc.to_account_info().data_len();
        if len != 20 {
            return err!(OptionalErrors::ReallocFailed);
        }
    }
    Ok(())
}
