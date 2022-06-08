//! The typescript example serves to show how one would setup an Anchor
//! workspace with TypeScript tests and migrations.

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod pda_payer {
    use super::*;

    pub fn init_with_payer(ctx: Context<InitWithPayer>) -> Result<()> {
        ctx.accounts.my_program_account.foo = 42;
        ctx.accounts.my_pda_account.foo = 42;
        Ok(())
    }

    pub fn init_if_needed_with_payer(ctx: Context<InitIfNeededWithPayer>) -> Result<()> {
        ctx.accounts.my_program_account.foo = 42;
        ctx.accounts.my_pda_account.foo = 42;
        Ok(())
    }

    pub fn init_with_pda_as_payer(ctx: Context<InitWithPdaAsPayer>) -> Result<()> {
        ctx.accounts.normal_payer_program_account.foo = 42;
        ctx.accounts.normal_payer_pda_account.foo = 42;
        ctx.accounts.pda_payer_program_account.foo = 42;
        ctx.accounts.pda_payer_pda_account.foo = 42;
        Ok(())
    }

    pub fn init_if_needed_with_pda_as_payer(
        ctx: Context<InitIfNeededWithPdaAsPayer>,
    ) -> Result<()> {
        ctx.accounts.normal_payer_program_account.foo = 42;
        ctx.accounts.pda_payer_program_account.foo = 42;
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
    #[account(
        init,
        payer = normal_payer,
        space = 8 + 8,
    )]
    pub my_program_account: Account<'info, ProgramAccount>,
    #[account(
        init,
        payer = normal_payer,
        space = 8 + 8,
        seeds = [b"PdaAccountSeeds".as_ref()],
        bump,
    )]
    pub my_pda_account: Account<'info, ProgramAccount>,
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
    #[account(
        init_if_needed,
        payer = normal_payer,
        space = 8 + 8,
    )]
    pub my_program_account: Account<'info, ProgramAccount>,
    #[account(
        init_if_needed,
        payer = normal_payer,
        space = 8 + 8,
        seeds = [b"PdaAccountSeeds".as_ref()],
        bump,
    )]
    pub my_pda_account: Account<'info, ProgramAccount>,
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
        payer = normal_payer,
        space = 8 + 8,
    )]
    pub normal_payer_program_account: Account<'info, ProgramAccount>,
    #[account(
        init,
        payer = normal_payer,
        space = 8 + 8,
        seeds = [b"NormalPayerPdaAccountSeeds".as_ref()],
        bump,
    )]
    pub normal_payer_pda_account: Account<'info, ProgramAccount>,
    #[account(
        init,
        payer = pda_payer,
        token::mint = mint,
        token::authority = token_owner,
    )]
    pub pda_payer_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = pda_payer,
        space = 8 + 8,
    )]
    pub pda_payer_program_account: Account<'info, ProgramAccount>,
    #[account(
        init,
        payer = pda_payer,
        space = 8 + 8,
        seeds = [b"PdaPayerPdaAccountSeeds".as_ref()],
        bump,
    )]
    pub pda_payer_pda_account: Account<'info, ProgramAccount>,
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
        payer = normal_payer,
        space = 8 + 8,
    )]
    pub normal_payer_program_account: Account<'info, ProgramAccount>,
    #[account(
        init_if_needed,
        payer = pda_payer,
        token::mint = mint,
        token::authority = token_owner,
    )]
    pub pda_payer_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = pda_payer,
        space = 8 + 8,
    )]
    pub pda_payer_program_account: Account<'info, ProgramAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    /// CHECK
    pub other_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct ProgramAccount {
    pub foo: u64,
}
