use anchor_lang::prelude::*;

declare_id!("9LA72twzmEHH6EH8oEiNnb2CsUdN9CqAtDNXCkj1c9Uw");

#[program]
pub mod new_layout {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.data.data = 2;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 16,
    )]
    data: Account<'info, Data>,
    #[account(mut)]
    payer: Signer<'info>,
    system_program: Program<'info, System>,
}

#[account]
pub struct Data {
    data: u64,
}
