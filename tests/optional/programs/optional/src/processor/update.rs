use anchor_lang::prelude::*;

use crate::state::*;

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    #[account(mut, seeds=[Data1::PREFIX.as_ref(), optional_2.as_ref().unwrap().key().as_ref()], bump)]
    pub optional_1: Option<Account<'info, Data1>>,
    #[account(mut, constraint = payer.is_some())]
    pub optional_2: Option<Account<'info, Data2>>,
    #[account(constraint = if optional_2.is_none() {optional_1.is_none()} else {true})]
    pub required: Program<'info, System>,
}

pub fn handle_update(ctx: Context<Update>, value: u64, key: Pubkey) -> Result<()> {
    if let Some(data) = &mut ctx.accounts.optional_1 {
        data.data = value;
    };
    if let Some(data2) = &mut ctx.accounts.optional_2 {
        data2.optional_1 = key;
    };
    Ok(())
}
