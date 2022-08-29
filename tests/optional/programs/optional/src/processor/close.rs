use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    #[account(mut, close = payer, constraint = system_program.is_some())]
    pub optional_1: Option<Account<'info, Data1>>,
    #[account(mut, close = payer, has_one = optional_1, constraint = payer.is_some())]
    pub optional_2: Option<Account<'info, Data2>>,
    pub system_program: Option<Program<'info, System>>,
}

pub fn handle_close(_ctx: Context<Close>) -> Result<()> {
    Ok(())
}
