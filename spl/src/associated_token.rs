use anchor_lang::solana_program::account_info::AccountInfo;
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::Result;
use anchor_lang::{context::CpiContext, Accounts};

pub use spl_associated_token_account::{
    get_associated_token_address, get_associated_token_address_with_program_id, ID,
};

pub fn create<'info>(ctx: CpiContext<'_, '_, '_, 'info, Create<'info>>) -> Result<()> {
    let ix = spl_associated_token_account::instruction::create_associated_token_account(
        ctx.accounts.payer.key,
        ctx.accounts.authority.key,
        ctx.accounts.mint.key,
        ctx.accounts.token_program.key,
    );
    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.payer.to_owned(),
            ctx.accounts.associated_token.to_owned(),
            ctx.accounts.authority.to_owned(),
            ctx.accounts.mint.to_owned(),
            ctx.accounts.system_program.to_owned(),
            ctx.accounts.token_program.to_owned(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn create_idempotent<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, CreateIdempotent<'info>>,
) -> Result<()> {
    let ix = spl_associated_token_account::instruction::create_associated_token_account_idempotent(
        ctx.accounts.payer.key,
        ctx.accounts.authority.key,
        ctx.accounts.mint.key,
        ctx.accounts.token_program.key,
    );
    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.payer.to_owned(),
            ctx.accounts.associated_token.to_owned(),
            ctx.accounts.authority.to_owned(),
            ctx.accounts.mint.to_owned(),
            ctx.accounts.system_program.to_owned(),
            ctx.accounts.token_program.to_owned(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct Create<'info> {
    pub payer: &'info AccountInfo<'info>,
    pub associated_token: &'info AccountInfo<'info>,
    pub authority: &'info AccountInfo<'info>,
    pub mint: &'info AccountInfo<'info>,
    pub system_program: &'info AccountInfo<'info>,
    pub token_program: &'info AccountInfo<'info>,
}

type CreateIdempotent<'info> = Create<'info>;

#[derive(Clone)]
pub struct AssociatedToken;

impl anchor_lang::Id for AssociatedToken {
    fn id() -> Pubkey {
        ID
    }
}
