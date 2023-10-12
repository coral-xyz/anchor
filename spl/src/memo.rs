use anchor_lang::prelude::CpiContext;
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::{Accounts, Result, ToAccountInfo};

pub use spl_memo;
pub use spl_memo::ID;

pub fn build_memo<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, BuildMemo<'info>>,
    memo: &[u8],
) -> Result<()> {
    let ix = spl_memo::build_memo(memo, &[ctx.accounts.signer.key]);
    solana_program::program::invoke_signed(
        &ix,
        &[ctx.accounts.signer.to_account_info()],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Accounts)]
pub struct BuildMemo<'info> {
    pub signer: AccountInfo<'info>,
}

#[derive(Clone)]
pub struct Memo;

impl anchor_lang::Id for Memo {
    fn id() -> Pubkey {
        ID
    }
}
