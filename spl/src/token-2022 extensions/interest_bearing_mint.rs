use anchor_lang::solana_program::account_info::AccountInfo;
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::{context::CpiContext, Accounts};
use anchor_lang::{solana_program::program::invoke_signed, Result};
use spl_token_2022::extension::interest_bearing_mint::{self};



pub mod interest_bearing {
    use super::*;
    pub fn initialize<'a, 'b, 'c, 'info>(
        ctx: CpiContext<'a,'b,'c,'info, Initialize<'info>>,
        rate_authority: Option<Pubkey>,
        rate: i16,
    ) -> Result<()> {
        let ix = interest_bearing_mint::instruction::initialize(
            &spl_token_2022::ID, 
            &ctx.accounts.mint.key, 
            rate_authority,
            rate
        )?;
        invoke_signed(
            &ix, 
            &[
                ctx.accounts.mint.clone(),
                ctx.accounts.token_program.clone(),
            ], 
            &ctx.signer_seeds
        ).map_err(Into::into)
    
    }
    pub fn update<'a,'b,'c,'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info, Update<'info>>,
        rate_authority: Option<Pubkey>,
        rate: i16,
    )-> Result<()>{
        let ix = interest_bearing_mint::instruction::update_rate(
            &spl_token_2022::ID, 
            &ctx.accounts.mint.key, 
            &rate_authority.unwrap(), 
            &[], 
            rate
        )?;
        invoke_signed(
            &ix, 
            &[
                ctx.accounts.mint.clone(),
                ctx.accounts.owner.clone(),
                ctx.accounts.token_program.clone(),
            ], 
            &ctx.signer_seeds,
        ).map_err(Into::into)
    }
    
}

#[derive(Accounts)]
pub struct Initialize<'info>{
    pub mint: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,

}
#[derive(Accounts)]
pub struct Update<'info>{
    pub mint: AccountInfo<'info>,
    //owner of the mint account
    pub owner: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}
