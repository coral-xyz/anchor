use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod anchor_cpi_return {
    use super::*;

    pub struct StructReturn {
        pub data: u64,
    }

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn return_u64(_ctx: Context<CpiReturn>) -> Result<u64> {
        Ok(10)
    }

    /*
    pub fn return_struct(ctx: Context<CpiReturn>) -> Result<StructReturn> {
        let s = StructReturn { data: 10 };
        Ok(s)
    }
    */
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 8)]
    pub account: Account<'info, CpiReturnAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CpiReturn<'info> {
    #[account(mut)]
    pub account: Account<'info, CpiReturnAccount>,
}

#[account]
pub struct CpiReturnAccount {
    pub value: u64,
}
