//! The typescript example serves to show how one would setup an Anchor
//! workspace with TypeScript tests and migrations.

use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub const MY_SEED: [u8; 2] = *b"hi";

#[program]
pub mod pda_derivation {
    use super::*;

    pub fn init_base(ctx: Context<InitBase>) -> ProgramResult {
        Ok(())
    }

    pub fn init_my_account(ctx: Context<InitMyAccount>, seed_a: u8, bump: u8) -> ProgramResult {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitBase<'info> {
    #[account(
				init,
				payer = payer,
				space = 8+8,
		)]
    base: Account<'info, BaseAccount>,
    #[account(mut)]
    payer: Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(seed_a: u8, bump: u8)]
pub struct InitMyAccount<'info> {
    base: Account<'info, BaseAccount>,
    #[account(
				init,
				payer = payer,
				space = 8+8,
				seeds = [
						seed_a.to_le_bytes().as_ref(),
						b"another-feed".as_ref(),
						base.key().as_ref(),
						MY_SEED.as_ref(),
						base.base_data.to_le_bytes().as_ref(),
				],
				bump = bump,
		)]
    account: Account<'info, MyAccount>,
    #[account(mut)]
    payer: Signer<'info>,
    system_program: Program<'info, System>,
}

#[account]
pub struct MyAccount {
    data: u64,
}

#[account]
pub struct BaseAccount {
    base_data: u64,
}
