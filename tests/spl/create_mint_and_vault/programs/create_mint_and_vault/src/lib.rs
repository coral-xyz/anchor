use anchor_lang::prelude::*;
use anchor_spl::{self, associated_token::AssociatedToken, token::{ self, Mint, TokenAccount, Token }};

declare_id!("88f8Tx22T5oggNQZBwjA1MkRAMyr68fPxvxhjnRk8zZY");

#[program]
pub mod create_mint_and_vault {
    use super::*;

    pub fn create_mint_and_vault(ctx: Context<Initialize>, decimals: u8, amount: u64) -> ProgramResult {
        
        let mint_to_ctx = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info()
        };
        return token::mint_to(CpiContext::new(ctx.accounts.token_program.to_account_info(), mint_to_ctx), amount);

    }
}

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(init, payer = authority, mint::decimals = decimals, mint::authority = authority, mint::freeze_authority = authority)]
    pub mint: Account<'info, Mint>,

    #[account(init, payer = authority, associated_token::mint = mint, associated_token::authority = authority)]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
