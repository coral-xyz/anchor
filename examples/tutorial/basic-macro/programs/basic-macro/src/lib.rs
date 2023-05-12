use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
mod basic_macro {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, data: u64) -> Result<()> {
        let my_account = &mut ctx.accounts.my_account;
        my_account.data = data;
        Ok(())
    }

    pub fn update(ctx: Context<Update>, data: u64) -> Result<()> {
        let my_account = &mut ctx.accounts.my_account;
        my_account.data = data;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 8)]
    pub my_account: Account<'info, MyAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Just the dummy macro which proves that Anchor can expand macros to parse
// the accounts struct.
macro_rules! impl_update {
    ($name:ident) => {
        #[derive(Accounts)]
        pub struct $name<'info> {
            #[account(mut)]
            pub my_account: Account<'info, MyAccount>,
        }
    };
}

impl_update!(Update);

// And another one.
macro_rules! impl_account {
    ($name:ident) => {
        #[account]
        pub struct $name {
            pub data: u64,
        }
    };
}

impl_account!(MyAccount);
