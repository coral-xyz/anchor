use anchor_lang::{Accounts, CpiContext};

pub fn create_associated_token_account<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, CreateAssociatedTokenAccount<'info>>,
) -> ProgramResult {
    let ix = spl_associated_token_account::create_associated_token_account(
        funding_address: &Pubkey,
        wallet_address: &Pubkey,
        spl_token_mint_address: &Pubkey
    )
   
    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.fee_payer.clone(),
            ctx.accounts.assocated_token_account.clone(),
            ctx.accounts.base_account.clone(),
            ctx.accounts.mint.clone(),
            ctx.accounts.system_program.clone(),
            ctx.accounts.token_program.clone(),
            ctx.accounts.rent.clone(),
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
