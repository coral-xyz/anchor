use anchor_lang::solana_program::account_info::AccountInfo;
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::Result;
use anchor_lang::{context::CpiContext, Accounts};

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Clone, Copy, BorshDeserialize, BorshSerialize)]
pub struct GroupPointerInitializeArgs {
    pub authority: Option<Pubkey>,
    pub group_address: Option<Pubkey>,
}

pub fn group_pointer_initialize<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, GroupPointerInitialize<'info>>,
    args: GroupPointerInitializeArgs,
) -> Result<()> {
    let ix = spl_token_2022::extension::group_pointer::instruction::initialize(
        ctx.accounts.token_program_id.key,
        ctx.accounts.mint.key,
        args.authority,
        args.group_address,
    )?;
    solana_program::program::invoke_signed(
        &ix,
        &[ctx.accounts.token_program_id, ctx.accounts.mint],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct GroupPointerInitialize<'info> {
    pub token_program_id: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
}
