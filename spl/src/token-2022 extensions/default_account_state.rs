use anchor_lang::prelude::*;
use spl_token_2022::extension::default_account_state::{self};
use anchor_lang::solana_program::program::invoke_signed;
use spl_token_2022::state::AccountState;


pub mod default_account_wrapper {
    use super::*;
    pub fn initialize<'a,'b, 'c, 'info>(
        ctx: CpiContext<'a,'b,'c, 'info, Initialize<'info>>,
        state: &AccountState,    
    ) -> Result<()> {
        let ix = default_account_state::instruction::initialize_default_account_state(
            
            &spl_token_2022::ID,
            &ctx.accounts.mint.to_account_info().key,
            state,
            
        )?;
        invoke_signed(
            &ix,
            &[
                ctx.accounts.mint.clone(),
            ],
            &ctx.signer_seeds,
        ).map_err(Into::into)
    }
    pub fn update<'a,'b, 'c, 'info>(
        ctx: CpiContext<'a,'b, 'c, 'info, Update<'info>>,
        state: &AccountState,
    ) -> Result<()>{
        let ix = default_account_state::instruction::update_default_account_state(
            &spl_token_2022::ID,
            &ctx.accounts.mint.key,
            &ctx.accounts.freeze_authority.key,
            &[],
            state,
        )?;
        invoke_signed(
            &ix, 
            &[
                ctx.accounts.mint.clone(),
                ctx.accounts.freeze_authority.clone(),
            ], 
            ctx.signer_seeds
        ).map_err(Into::into)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info>{
    pub mint: AccountInfo<'info>,
    pub rent: AccountInfo<'info>
}

#[derive(Accounts)]
pub struct Update<'info>{
    pub mint: AccountInfo<'info>,
    pub freeze_authority: AccountInfo<'info>,

}
