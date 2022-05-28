use anchor_lang::prelude::*;
use std::str::FromStr;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub const MY_SEED_U64: u64 = 3;

#[program]
pub mod new_idl {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[account]
pub struct State {
    foo_bool: bool
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + 1,
        seeds = [
            anchor_lang::solana_program::system_program::ID.as_ref(),
            Pubkey::from_str("3tMg6nFceRK19FX3WY1Cbtu6DboaabhdVfeYP5BKqkuH").unwrap().as_ref(),
            &MY_SEED_U64.to_le_bytes(),
            b"some-seed".as_ref(),
            &[8, 2]
        ],
        bump
    )]
    state: Account<'info, State>,

    #[account(mut)]
    payer: Signer<'info>,
    system_program: Program<'info, System>,
}
