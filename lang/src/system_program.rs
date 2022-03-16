use crate::prelude::*;
use solana_program::pubkey::Pubkey;

pub use solana_program::system_program::ID;

#[derive(Debug, Clone)]
pub struct System;

impl anchor_lang::Id for System {
    fn id() -> Pubkey {
        ID
    }
}

pub fn advance_nonce_account<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, AdvanceNonceAccount<'info>>,
) -> Result<()> {
    let ix = crate::solana_program::system_instruction::advance_nonce_account(
        ctx.accounts.nonce.key,
        ctx.accounts.authorized.key,
    );
    crate::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.nonce,
            ctx.accounts.recent_blockhashes,
            ctx.accounts.authorized,
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct AdvanceNonceAccount<'info> {
    pub nonce: AccountInfo<'info>,
    pub authorized: AccountInfo<'info>,
    pub recent_blockhashes: AccountInfo<'info>,
}

pub fn allocate<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, Allocate<'info>>,
    space: u64,
) -> Result<()> {
    let ix = crate::solana_program::system_instruction::allocate(
        ctx.accounts.account_to_allocate.key,
        space,
    );
    crate::solana_program::program::invoke_signed(
        &ix,
        &[ctx.accounts.account_to_allocate],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct Allocate<'info> {
    pub account_to_allocate: AccountInfo<'info>,
}

pub fn allocate_with_seed<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, AllocateWithSeed<'info>>,
    seed: &str,
    space: u64,
    owner: &Pubkey,
) -> Result<()> {
    let ix = crate::solana_program::system_instruction::allocate_with_seed(
        ctx.accounts.account_to_allocate.key,
        ctx.accounts.base.key,
        seed,
        space,
        owner,
    );
    crate::solana_program::program::invoke_signed(
        &ix,
        &[ctx.accounts.account_to_allocate, ctx.accounts.base],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct AllocateWithSeed<'info> {
    pub account_to_allocate: AccountInfo<'info>,
    pub base: AccountInfo<'info>,
}

pub fn assign<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, Assign<'info>>,
    owner: &Pubkey,
) -> Result<()> {
    let ix = crate::solana_program::system_instruction::assign(
        ctx.accounts.account_to_assign.key,
        owner,
    );
    crate::solana_program::program::invoke_signed(
        &ix,
        &[ctx.accounts.account_to_assign],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct Assign<'info> {
    pub account_to_assign: AccountInfo<'info>,
}

pub fn assign_with_seed<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, AssignWithSeed<'info>>,
    seed: &str,
    owner: &Pubkey,
) -> Result<()> {
    let ix = crate::solana_program::system_instruction::assign_with_seed(
        ctx.accounts.account_to_assign.key,
        ctx.accounts.base.key,
        seed,
        owner,
    );
    crate::solana_program::program::invoke_signed(
        &ix,
        &[ctx.accounts.account_to_assign, ctx.accounts.base],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct AssignWithSeed<'info> {
    pub account_to_assign: AccountInfo<'info>,
    pub base: AccountInfo<'info>,
}

pub fn authorize_nonce_account<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, AuthorizeNonceAccount<'info>>,
    new_authority: &Pubkey,
) -> Result<()> {
    let ix = crate::solana_program::system_instruction::authorize_nonce_account(
        ctx.accounts.nonce.key,
        ctx.accounts.authorized.key,
        new_authority,
    );
    crate::solana_program::program::invoke_signed(
        &ix,
        &[ctx.accounts.nonce, ctx.accounts.authorized],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct AuthorizeNonceAccount<'info> {
    pub nonce: AccountInfo<'info>,
    pub authorized: AccountInfo<'info>,
}

pub fn create_account<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, CreateAccount<'info>>,
    lamports: u64,
    space: u64,
    owner: &Pubkey,
) -> Result<()> {
    let ix = crate::solana_program::system_instruction::create_account(
        ctx.accounts.from.key,
        ctx.accounts.to.key,
        lamports,
        space,
        owner,
    );
    crate::solana_program::program::invoke_signed(
        &ix,
        &[ctx.accounts.from, ctx.accounts.to],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct CreateAccount<'info> {
    pub from: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
}

pub fn create_account_with_seed<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, CreateAccountWithSeed<'info>>,
    seed: &str,
    lamports: u64,
    space: u64,
    owner: &Pubkey,
) -> Result<()> {
    let ix = crate::solana_program::system_instruction::create_account_with_seed(
        ctx.accounts.from.key,
        ctx.accounts.to.key,
        ctx.accounts.base.key,
        seed,
        lamports,
        space,
        owner,
    );
    crate::solana_program::program::invoke_signed(
        &ix,
        &[ctx.accounts.from, ctx.accounts.to, ctx.accounts.base],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct CreateAccountWithSeed<'info> {
    pub from: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
    pub base: AccountInfo<'info>,
}

pub fn transfer<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, Transfer<'info>>,
    lamports: u64,
) -> Result<()> {
    let ix = crate::solana_program::system_instruction::transfer(
        ctx.accounts.from.key,
        ctx.accounts.to.key,
        lamports,
    );
    crate::solana_program::program::invoke_signed(
        &ix,
        &[ctx.accounts.from, ctx.accounts.to],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    pub from: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
}

pub fn transfer_with_seed<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, TransferWithSeed<'info>>,
    from_seed: String,
    from_owner: &Pubkey,
    lamports: u64,
) -> Result<()> {
    let ix = crate::solana_program::system_instruction::transfer_with_seed(
        ctx.accounts.from.key,
        ctx.accounts.base.key,
        from_seed,
        from_owner,
        ctx.accounts.to.key,
        lamports,
    );
    crate::solana_program::program::invoke_signed(
        &ix,
        &[ctx.accounts.from, ctx.accounts.base, ctx.accounts.to],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct TransferWithSeed<'info> {
    pub from: AccountInfo<'info>,
    pub base: AccountInfo<'info>,
    pub to: AccountInfo<'info>,
}
