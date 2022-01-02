use anchor_lang::prelude::*;
use anchor_spl::{self, token::{ self, Mint, TokenAccount, Token }};

declare_id!("88f8Tx22T5oggNQZBwjA1MkRAMyr68fPxvxhjnRk8zZY");

#[program]
pub mod create_mint_and_vault {
    use super::*;

    pub fn create_mint_and_vault(ctx: Context<Initialize>, decimals: u8, amount: u64) -> ProgramResult {
        
        // 1. initialize mint
        let init_mint_ctx = token::InitializeMint {
            mint: ctx.accounts.mint.clone(),
            rent: ctx.accounts.rent.to_account_info()
        };
        if let Err(err) = token::initialize_mint(CpiContext::new(ctx.accounts.token_program.to_account_info(), init_mint_ctx), decimals, ctx.accounts.authority.key, Some(ctx.accounts.authority.key)) {
            return Err(err);
        }
        
        // 2. initialize account
        let init_account_ctx = token::InitializeAccount {
            mint: ctx.accounts.mint.clone(),
            account: ctx.accounts.vault.clone(),
            authority: ctx.accounts.authority.clone(),
            rent: ctx.accounts.rent.to_account_info()
        };
        if let Err(err) = token::initialize_account(CpiContext::new(ctx.accounts.token_program.to_account_info(), init_account_ctx)) {
            return Err(err);
        }
            
        // 3. mint to
        let mint_to_ctx = token::MintTo {
            mint: ctx.accounts.mint.clone(),
            to: ctx.accounts.vault.clone(),
            authority: ctx.accounts.authority.clone()
        };
        if let Err(err) =  token::mint_to(CpiContext::new(ctx.accounts.token_program.to_account_info(), mint_to_ctx), amount) {
            return Err(err);
        }
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    
    #[account(init, payer = authority, owner = token::ID, space = Mint::LEN)]
    pub mint: AccountInfo<'info>,

    #[account(init, payer = authority, owner = token::ID, space = TokenAccount::LEN)]
    pub vault: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
