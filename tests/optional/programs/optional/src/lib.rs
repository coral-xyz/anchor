//! This example demonstrates the ability to use optional accounts in
//! structs deriving `Accounts`.

use anchor_lang::prelude::*;

declare_id!("FNqz6pqLAwvMSds2FYjR4nKV3moVpPNtvkfGFrqLKrgG");

#[program]
mod optional {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, value: u64, key: Pubkey) -> Result<()> {
        let optional = &mut ctx.accounts.optional1;
        let optional2 = &mut ctx.accounts.optional2;

        if let Some(data) = optional {
            data.data = value;
        }
        if let Some(data2) = optional2 {
            if let Ok(optional_key) = optional.try_key() {
                data2.optional1 = optional_key;
            } else {
                data2.optional1 = key;
            }
        }
        Ok(())
    }

    pub fn update(ctx: Context<Update>, value: u64, key: Pubkey) -> Result<()> {
        if let Some(data) = &mut ctx.accounts.optional1 {
            data.data = value;
        };
        if let Some(data2) = &mut ctx.accounts.optional2 {
            data2.optional1 = key;
        };
        Ok(())
    }

    pub fn realloc(ctx: Context<Realloc>) -> Result<()> {
        let optional = &ctx.accounts.optional1;
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
    #[account(init, payer=payer, space=Data1::LEN, constraint = payer.is_some() && system_program.is_some())]
    pub optional1: Option<Account<'info, Data1>>,
    #[account(init, payer=payer, space=Data2::LEN, constraint = payer.is_some() && system_program.is_some())]
    pub optional2: Option<Account<'info, Data2>>,
    pub system_program: Option<Program<'info, System>>,
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    #[account(mut, constraint = payer.is_some())]
    pub optional1: Option<Account<'info, Data1>>,
    #[account(mut, constraint = payer.is_some())]
    pub optional2: Option<Account<'info, Data2>>,
}

#[derive(Accounts)]
pub struct Realloc<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, realloc = 20, realloc::payer = payer, realloc::zero = false)]
    pub optional1: Option<Account<'info, Data1>>,
    #[account(has_one = optional1)]
    pub optional2: Option<Account<'info, Data2>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    #[account(mut, close = payer, constraint = payer.is_some() && system_program.is_some())]
    pub optional1: Option<Account<'info, Data1>>,
    #[account(mut, close = payer, has_one = optional1, constraint = payer.is_some() && system_program.is_some())]
    pub optional2: Option<Account<'info, Data2>>,
    pub system_program: Option<Program<'info, System>>,
}

#[account]
pub struct Data1 {
    pub data: u64,
}

impl Data1 {
    const LEN: usize = 8 + 8;
}

#[account]
pub struct Data2 {
    pub optional1: Pubkey,
}

impl Data2 {
    const LEN: usize = 8 + 32;
}
