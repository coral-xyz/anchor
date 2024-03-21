use anchor_lang::prelude::*;

declare_id!("TryFrom111111111111111111111111111111111111");

#[program]
pub mod try_from {
    use super::*;

    pub fn init(ctx: Context<Init>, field: u8) -> Result<()> {
        ctx.accounts.my_account.field = field;
        Ok(())
    }

    pub fn try_from(ctx: Context<TryFrom>, field: u8) -> Result<()> {
        let my_account = try_from!(Account::<MyAccount>, ctx.accounts.my_account)?;
        msg!("Field: {}", my_account.field);
        require_eq!(my_account.field, field);

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Init<'info> {
    #[account(init, payer = payer, space = 8 + 1)]
    pub my_account: Account<'info, MyAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TryFrom<'info> {
    pub my_account: UncheckedAccount<'info>,
}

#[account]
pub struct MyAccount {
    pub field: u8,
}
