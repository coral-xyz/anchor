use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod account_move {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init, seeds = ["test_account".as_bytes()], bump, payer = payer, space = 100)]
    pub test_account: AccountLoader<'info, TestAccount>,
    pub system_program: Program<'info, System>,
}

#[account(zero_copy)]
pub struct TestAccount {
    random_number: u64,
}
