use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::Result;
use anchor_lang::{context::CpiContext, Accounts};

pub use spl_memo;
pub use spl_memo::ID;

pub fn build_memo<'info>(ctx: CpiContext<'_, '_, '_, 'info, BuildMemo>, memo: &[u8]) -> Result<()> {
    let mut signer_pubkeys: Vec<&Pubkey> = vec![];
    for i in 0..ctx.remaining_accounts.len() {
        signer_pubkeys.push(ctx.remaining_accounts[i].key);
    }

    let ix = spl_memo::build_memo(memo, &signer_pubkeys);
    solana_program::program::invoke_signed(&ix, &ctx.remaining_accounts, ctx.signer_seeds)
        .map_err(Into::into)
}

#[derive(Accounts)]
pub struct BuildMemo {}

#[derive(Clone)]
pub struct Memo;

impl anchor_lang::Id for Memo {
    fn id() -> Pubkey {
        ID
    }
}
