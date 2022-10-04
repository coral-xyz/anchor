use crate::account::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    #[account(init, payer = payer, space = DataAccount::LEN)]
    pub optional_account: Option<Account<'info, DataAccount>>,
    pub system_program: Option<Program<'info, System>>,
    #[account(zero)]
    pub required: Account<'info, DataAccount>,
    #[account(init, seeds=[DataPda::PREFIX.as_ref(), optional_account.as_ref().unwrap().key().as_ref()], bump, payer=payer, space=DataPda::LEN, constraint = payer.is_some())]
    pub optional_pda: Option<Account<'info, DataPda>>,
}

#[derive(Accounts)]
#[instruction(pda_bump: u8)]
pub struct Update<'info> {
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    #[account(mut, seeds=[DataPda::PREFIX.as_ref(), optional_account.as_ref().unwrap().key().as_ref()], bump = pda_bump)]
    pub optional_pda: Option<Account<'info, DataPda>>,
    #[account(mut, constraint = payer.is_some())]
    pub optional_account: Option<Account<'info, DataAccount>>,
}

#[derive(Accounts)]
pub struct Realloc<'info> {
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    #[account(mut, realloc = 50, realloc::payer = payer, realloc::zero = false)]
    pub optional_pda: Option<Account<'info, DataPda>>,
    pub required: Account<'info, DataAccount>,
    pub system_program: Option<Program<'info, System>>,
    #[account(mut, realloc = 50, realloc::payer = payer, realloc::zero = false)]
    pub optional_account: Option<Account<'info, DataAccount>>,
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    #[account(mut, close = payer, constraint = system_program.is_some())]
    pub data_pda: Option<Account<'info, DataPda>>,
    #[account(mut, close = payer, has_one = data_pda, constraint = payer.is_some())]
    pub optional_account: Option<Account<'info, DataAccount>>,
    pub system_program: Option<Program<'info, System>>,
}
