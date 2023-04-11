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

pub fn advance_nonce_account<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, AdvanceNonceAccount<'info>>,
) -> Result<()> {
    let ix = crate::solana_program::system_instruction::advance_nonce_account(
        ctx.accounts.nonce.key,
        ctx.accounts.authorized.key,
    );
    crate::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.nonce.to_owned(),
            ctx.accounts.recent_blockhashes.to_owned(),
            ctx.accounts.authorized.to_owned(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct AdvanceNonceAccount<'info> {
    pub nonce: &'info AccountInfo<'info>,
    pub authorized: &'info AccountInfo<'info>,
    pub recent_blockhashes: &'info AccountInfo<'info>,
}

pub fn allocate<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, Allocate<'info>>,
    space: u64,
) -> Result<()> {
    let ix = crate::solana_program::system_instruction::allocate(
        ctx.accounts.account_to_allocate.key,
        space,
    );
    crate::solana_program::program::invoke_signed(
        &ix,
        &[ctx.accounts.account_to_allocate.to_owned()],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct Allocate<'info> {
    pub account_to_allocate: &'info AccountInfo<'info>,
}

pub fn allocate_with_seed<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, AllocateWithSeed<'info>>,
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
        &[
            ctx.accounts.account_to_allocate.to_owned(),
            ctx.accounts.base.to_owned(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct AllocateWithSeed<'info> {
    pub account_to_allocate: &'info AccountInfo<'info>,
    pub base: &'info AccountInfo<'info>,
}

pub fn assign<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, Assign<'info>>,
    owner: &Pubkey,
) -> Result<()> {
    let ix = crate::solana_program::system_instruction::assign(
        ctx.accounts.account_to_assign.key,
        owner,
    );
    crate::solana_program::program::invoke_signed(
        &ix,
        &[ctx.accounts.account_to_assign.to_owned()],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct Assign<'info> {
    pub account_to_assign: &'info AccountInfo<'info>,
}

pub fn assign_with_seed<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, AssignWithSeed<'info>>,
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
        &[
            ctx.accounts.account_to_assign.to_owned(),
            ctx.accounts.base.to_owned(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct AssignWithSeed<'info> {
    pub account_to_assign: &'info AccountInfo<'info>,
    pub base: &'info AccountInfo<'info>,
}

pub fn authorize_nonce_account<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, AuthorizeNonceAccount<'info>>,
    new_authority: &Pubkey,
) -> Result<()> {
    let ix = crate::solana_program::system_instruction::authorize_nonce_account(
        ctx.accounts.nonce.key,
        ctx.accounts.authorized.key,
        new_authority,
    );
    crate::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.nonce.to_owned(),
            ctx.accounts.authorized.to_owned(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct AuthorizeNonceAccount<'info> {
    pub nonce: &'info AccountInfo<'info>,
    pub authorized: &'info AccountInfo<'info>,
}

pub fn create_account<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, CreateAccount<'info>>,
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
        &[ctx.accounts.from.to_owned(), ctx.accounts.to.to_owned()],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct CreateAccount<'info> {
    pub from: &'info AccountInfo<'info>,
    pub to: &'info AccountInfo<'info>,
}

pub fn create_account_with_seed<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, CreateAccountWithSeed<'info>>,
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
        &[
            ctx.accounts.from.to_owned(),
            ctx.accounts.to.to_owned(),
            ctx.accounts.base.to_owned(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct CreateAccountWithSeed<'info> {
    pub from: &'info AccountInfo<'info>,
    pub to: &'info AccountInfo<'info>,
    pub base: &'info AccountInfo<'info>,
}

pub fn create_nonce_account<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, CreateNonceAccount<'info>>,
    lamports: u64,
    authority: &Pubkey,
) -> Result<()> {
    let ixs = crate::solana_program::system_instruction::create_nonce_account(
        ctx.accounts.from.key,
        ctx.accounts.nonce.key,
        authority,
        lamports,
    );
    crate::solana_program::program::invoke_signed(
        &ixs[0],
        &[ctx.accounts.from.to_owned(), ctx.accounts.nonce.to_owned()],
        ctx.signer_seeds,
    )?;

    crate::solana_program::program::invoke_signed(
        &ixs[1],
        &[
            ctx.accounts.nonce.to_owned(),
            ctx.accounts.recent_blockhashes.to_owned(),
            ctx.accounts.rent.to_owned(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct CreateNonceAccount<'info> {
    pub from: &'info AccountInfo<'info>,
    pub nonce: &'info AccountInfo<'info>,
    pub recent_blockhashes: &'info AccountInfo<'info>,
    pub rent: &'info AccountInfo<'info>,
}

pub fn create_nonce_account_with_seed<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, CreateNonceAccountWithSeed<'info>>,
    lamports: u64,
    seed: &str,
    authority: &Pubkey,
) -> Result<()> {
    let ixs = crate::solana_program::system_instruction::create_nonce_account_with_seed(
        ctx.accounts.from.key,
        ctx.accounts.nonce.key,
        ctx.accounts.base.key,
        seed,
        authority,
        lamports,
    );
    crate::solana_program::program::invoke_signed(
        &ixs[0],
        &[
            ctx.accounts.from.to_owned(),
            ctx.accounts.nonce.to_owned(),
            ctx.accounts.base.to_owned(),
        ],
        ctx.signer_seeds,
    )?;

    crate::solana_program::program::invoke_signed(
        &ixs[1],
        &[
            ctx.accounts.nonce.to_owned(),
            ctx.accounts.recent_blockhashes.to_owned(),
            ctx.accounts.rent.to_owned(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct CreateNonceAccountWithSeed<'info> {
    pub from: &'info AccountInfo<'info>,
    pub nonce: &'info AccountInfo<'info>,
    pub base: &'info AccountInfo<'info>,
    pub recent_blockhashes: &'info AccountInfo<'info>,
    pub rent: &'info AccountInfo<'info>,
}

pub fn transfer<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, Transfer<'info>>,
    lamports: u64,
) -> Result<()> {
    let ix = crate::solana_program::system_instruction::transfer(
        ctx.accounts.from.key,
        ctx.accounts.to.key,
        lamports,
    );
    crate::solana_program::program::invoke_signed(
        &ix,
        &[ctx.accounts.from.to_owned(), ctx.accounts.to.to_owned()],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    pub from: &'info AccountInfo<'info>,
    pub to: &'info AccountInfo<'info>,
}

pub fn transfer_with_seed<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, TransferWithSeed<'info>>,
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
        &[
            ctx.accounts.from.to_owned(),
            ctx.accounts.base.to_owned(),
            ctx.accounts.to.to_owned(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct TransferWithSeed<'info> {
    pub from: &'info AccountInfo<'info>,
    pub base: &'info AccountInfo<'info>,
    pub to: &'info AccountInfo<'info>,
}

pub fn withdraw_nonce_account<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, WithdrawNonceAccount<'info>>,
    lamports: u64,
) -> Result<()> {
    let ix = crate::solana_program::system_instruction::withdraw_nonce_account(
        ctx.accounts.nonce.key,
        ctx.accounts.authorized.key,
        ctx.accounts.to.key,
        lamports,
    );
    crate::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.nonce.to_owned(),
            ctx.accounts.to.to_owned(),
            ctx.accounts.recent_blockhashes.to_owned(),
            ctx.accounts.rent.to_owned(),
            ctx.accounts.authorized.to_owned(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct WithdrawNonceAccount<'info> {
    pub nonce: &'info AccountInfo<'info>,
    pub to: &'info AccountInfo<'info>,
    pub recent_blockhashes: &'info AccountInfo<'info>,
    pub rent: &'info AccountInfo<'info>,
    pub authorized: &'info AccountInfo<'info>,
}
