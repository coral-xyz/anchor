use anchor_lang::solana_program::account_info::AccountInfo;
use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::solana_program::rent::Rent;
use anchor_lang::{Accounts, CpiContext, Sysvar, ToAccountInfo};

pub use spl_associated_token_account::ID;

pub fn create_associated_token_account<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, CreateAssociatedTokenAccount<'info>>,
) -> ProgramResult {
    let ix = spl_associated_token_account::create_associated_token_account(
        ctx.accounts.fee_payer.to_account_info().key,
        ctx.accounts.base_account.to_account_info().key,
        ctx.accounts.mint.to_account_info().key,
    );

    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.fee_payer.clone(),
            ctx.accounts.assocated_token_account.clone(),
            ctx.accounts.base_account.clone(),
            ctx.accounts.mint.clone(),
            ctx.accounts.system_program.clone(),
            ctx.accounts.token_program.clone(),
            ctx.accounts.rent.to_account_info(),
        ],
        ctx.signer_seeds,
    )
}

#[derive(Accounts)]
pub struct CreateAssociatedTokenAccount<'info> {
    // `[writeable,signer]` Funding account (must be a system account)
    #[account(mut, signer)]
    pub fee_payer: AccountInfo<'info>,
    // `[writeable]` Associated token account address to be created
    #[account(mut)]
    pub assocated_token_account: AccountInfo<'info>,
    // `[]` Wallet (base account) address for the new associated token account
    pub base_account: AccountInfo<'info>,
    // `[]` The token mint for the new associated token account
    pub mint: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}
