use crate::state::*;
use crate::OptionalErrors;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    #[account(zero)]
    pub optional_2: Option<Account<'info, Data2>>,
    #[account(zero)]
    pub required: Account<'info, Data2>,
    #[account(init, seeds=[Data1::PREFIX.as_ref(), optional_2.unwrap().key().as_ref()], bump, payer=payer, space=Data1::LEN, constraint = payer.is_some())]
    pub optional_1: Option<Account<'info, Data1>>,
    pub system_program: Option<Program<'info, System>>,
}

pub fn handle_initialize(ctx: Context<Initialize>, value: u64, key: Pubkey) -> Result<()> {
    let optional_1 = &mut ctx.accounts.optional_1;
    let optional_2 = &mut ctx.accounts.optional_2;
    let required = &mut ctx.accounts.required;

    required.optional_1 = Pubkey::default();

    if let Some(data) = optional_1 {
        data.data = value;
    }
    if let Some(data2) = optional_2 {
        if let Some(optional) = optional_1 {
            data2.optional_1 = optional.key();
        } else {
            data2.optional_1 = key;
        }
    }

    Ok(())
}
