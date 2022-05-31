//! The typescript example serves to show how one would setup an Anchor
//! workspace with TypeScript tests and migrations.

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod pda_payer {
    use super::*;

    pub fn init_with_payer(_ctx: Context<InitWithPayer>) -> Result<()> {
        Ok(())
    }

    pub fn init_if_needed_with_payer(_ctx: Context<InitIfNeededWithPayer>) -> Result<()> {
        Ok(())
    }

    pub fn init_with_pda_as_payer(_ctx: Context<InitWithPdaAsPayer>) -> Result<()> {
        Ok(())
    }

    pub fn init_if_needed_with_pda_as_payer(
        _ctx: Context<InitIfNeededWithPdaAsPayer>,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitWithPayer<'info> {
    #[account(mut)]
    pub normal_payer: Signer<'info>,
    pub mint: Account<'info, Mint>,
    /// CHECK
    pub token_owner: AccountInfo<'info>,
    #[account(
        init,
        payer = normal_payer,
        token::mint = mint,
        token::authority = token_owner,
    )]
    pub my_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = normal_payer,
        token::mint = mint,
        token::authority = token_owner,
    )]
    pub my_another_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitIfNeededWithPayer<'info> {
    #[account(mut)]
    pub normal_payer: Signer<'info>,
    pub mint: Account<'info, Mint>,
    /// CHECK
    pub token_owner: AccountInfo<'info>,
    #[account(
        init_if_needed,
        payer = normal_payer,
        token::mint = mint,
        token::authority = token_owner,
    )]
    pub my_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = normal_payer,
        token::mint = mint,
        token::authority = token_owner,
    )]
    pub my_another_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitWithPdaAsPayer<'info> {
    #[account(mut)]
    pub normal_payer: Signer<'info>,
    /// CHECK
    #[account(
        mut,
        seeds = [b"SomeOtherSeeds".as_ref()],
        bump,
    )]
    pub pda_payer: AccountInfo<'info>,
    /// CHECK
    #[account(
        seeds = [b"SomeSeeds".as_ref()],
        bump,
        seeds::program = other_program.key(),
    )]
    pub normal_pda: AccountInfo<'info>,
    pub mint: Account<'info, Mint>,
    /// CHECK
    pub token_owner: AccountInfo<'info>,
    #[account(
        init,
        payer = normal_payer,
        token::mint = mint,
        token::authority = token_owner,
    )]
    pub normal_payer_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = pda_payer,
        token::mint = mint,
        token::authority = token_owner,
    )]
    pub pda_payer_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    /// CHECK
    pub other_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitIfNeededWithPdaAsPayer<'info> {
    #[account(mut)]
    pub normal_payer: Signer<'info>,
    /// CHECK
    #[account(
        mut,
        seeds = [b"SomeOtherSeeds".as_ref()],
        bump,
    )]
    pub pda_payer: AccountInfo<'info>,
    /// CHECK
    #[account(
        seeds = [b"SomeSeeds".as_ref()],
        bump,
        seeds::program = other_program.key(),
    )]
    pub normal_pda: AccountInfo<'info>,
    pub mint: Account<'info, Mint>,
    /// CHECK
    pub token_owner: AccountInfo<'info>,
    #[account(
        init_if_needed,
        payer = normal_payer,
        token::mint = mint,
        token::authority = token_owner,
    )]
    pub normal_payer_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = pda_payer,
        token::mint = mint,
        token::authority = token_owner,
    )]
    pub pda_payer_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    /// CHECK
    pub other_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}
