//! This example demonstrates the ability to use optional accounts in
//! structs deriving `Accounts`.

use anchor_lang::prelude::*;

declare_id!("EHthziFziNoac9LBGxEaVN47Y3uUiRoXvqAiR6oes4iU");

#[program]
mod optional {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>, value: u64) -> Result<()> {
        // let dummy = &mut ctx.accounts.dummy
        // todo!()
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    #[account(init, payer=payer, space=16)]
    pub dummy: Option<Account<'info, Dummy>>,
    #[account()]
    pub dummy2: Account<'info, Dummy>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Dummy {
    pub data: u64,
}
