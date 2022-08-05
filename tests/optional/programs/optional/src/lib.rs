//! This example demonstrates the ability to use optional accounts in
//! structs deriving `Accounts`.

use anchor_lang::prelude::*;

declare_id!("EHthziFziNoac9LBGxEaVN47Y3uUiRoXvqAiR6oes4iU");

#[program]
mod optional {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, value: u64, key: Pubkey) -> Result<()> {
        let optional = &mut ctx.accounts.optional;
        if let Some(data) = optional {
            data.data = value;
        }
        if let Some(data2) = &mut ctx.accounts.optional2 {
            if let Ok(optional_key) = optional.try_key() {
                data2.optional = optional_key;
            } else {
                data2.optional = key;
            }
        }
        Ok(())
    }

    pub fn update(ctx: Context<Update>, value: u64, key: Pubkey) -> Result<()> {
        if let Some(data) = &mut ctx.accounts.optional {
            data.data = value;
        };
        if let Some(data2) = &mut ctx.accounts.optional2 {
            data2.optional = key;
        };
        Ok(())
    }

    pub fn realloc(ctx: Context<Realloc>) -> Result<()> {
        let optional = &ctx.accounts.optional;
        if let Some(acc) = optional {
            let len = acc.to_account_info().data_len();
            msg!("Len: {}", len);
        }
        Ok(())
    }

    pub fn close(_ctx: Context<Close>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    #[account(init, payer=payer, space=16, constraint = payer.is_some() && system_program.is_some())]
    pub optional: Option<Account<'info, Data>>,
    #[account(init, payer=payer, space=16, constraint = payer.is_some() && system_program.is_some())]
    pub optional2: Option<Account<'info, Data2>>,
    pub system_program: Option<Program<'info, System>>,
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    #[account(mut, constraint = payer.is_some())]
    pub optional: Option<Account<'info, Data>>,
    #[account(mut, constraint = payer.is_some())]
    pub optional2: Option<Account<'info, Data2>>,
}

#[derive(Accounts)]
pub struct Realloc<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, realloc = 20, realloc::payer = payer, realloc::zero = false)]
    pub optional: Option<Account<'info, Data>>,
    #[account(has_one = optional)]
    pub optional2: Option<Account<'info, Data2>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    #[account(mut, close = payer, constraint = payer.is_some() && system_program.is_some())]
    pub optional: Option<Account<'info, Data>>,
    #[account(mut, close = payer, has_one = optional, constraint = payer.is_some() && system_program.is_some())]
    pub optional2: Option<Account<'info, Data2>>,
    pub system_program: Option<Program<'info, System>>,
}

#[account]
pub struct Data {
    pub data: u64,
}

#[account]
pub struct Data2 {
    pub optional: Pubkey,
}
