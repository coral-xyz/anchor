use anchor_lang::prelude::*;
use anchor_lang::context::CpiContext;
use spl_token_2022::extension::memo_transfer::{self};
use anchor_lang::solana_program::program::invoke_signed;

pub mod memo_wrapper {
    use super::*;
    pub fn enable_required_transfer_memos<'a, 'b,'c,'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info, Enable<'info>>
    ) -> Result<()> {
        let ix = memo_transfer::instruction::enable_required_transfer_memos(
            &spl_token_2022::ID, 
            &ctx.accounts.token_account.key, 
            &ctx.accounts.owner.key, 
            &[],
        )?;
        invoke_signed( 
            &ix,
            &[
                ctx.accounts.token_account.clone(),
                ctx.accounts.owner.clone()
            ],
            ctx.signer_seeds,
        ).map_err(Into::into)

    
    }
    pub fn disable_required_transfer_memos<'a, 'b, 'c,'info>(
        ctx: CpiContext<'a,'b,'c,'info, Disable<'info>>
    ) -> Result<()>{
        let ix = memo_transfer::instruction::disable_required_transfer_memos(
            &spl_token_2022::ID, 
            &ctx.accounts.token_account.key, 
            &ctx.accounts.owner.key, 
            &[]
        )?;

        invoke_signed(
            &ix, 
            &[
                ctx.accounts.token_account.clone(),
                ctx.accounts.owner.clone(),
            ], 
            ctx.signer_seeds,
        ).map_err(Into::into)

    }
}

#[derive(Accounts)]
pub struct Enable<'info> {
    pub token_account: AccountInfo<'info>,
    pub owner: AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct Disable<'info>{
    pub token_account: AccountInfo<'info>,
    pub owner: AccountInfo<'info>,
}
