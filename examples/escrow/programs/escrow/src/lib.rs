//! An example of an escrow program, inspired by PaulX tutorial seen here
//! https://paulx.dev/blog/2021/01/14/programming-on-solana-an-introduction/
//! This example has some changes to implementation, but more or less should be the same overall
//! Also gives examples on how to use some newer anchor features and CPI
//!
//! User (Initializer) constructs an escrow deal:
//! - SPL token (X) they will offer and amount
//! - SPL token (Y) count they want in return and amount
//! - Program will take ownership of initializer's token X account
//!
//! Once this escrow is initialised, either:
//! 1. User (Taker) can call the exchange function to exchange their Y for X
//! - This will close the escrow account and no longer be usable
//! OR
//! 2. If no one has exchanged, the initializer can close the escrow account
//! - Initializer will get back ownership of their token X account

use anchor_lang::prelude::*;
use anchor_spl::token::{self, SetAuthority, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;

#[program]
pub mod escrow {
    use super::*;

    pub fn initialize_escrow(
        ctx: Context<InitializeEscrow>,
        initializer_amount: u64,
        taker_amount: u64,
    ) -> ProgramResult {
        ctx.accounts.escrow_account.initializer_key = *ctx.accounts.initializer.key;
        ctx.accounts
            .escrow_account
            .initializer_deposit_token_account = *ctx
            .accounts
            .initializer_deposit_token_account
            .to_account_info()
            .key;
        ctx.accounts
            .escrow_account
            .initializer_receive_token_account = *ctx
            .accounts
            .initializer_receive_token_account
            .to_account_info()
            .key;
        ctx.accounts.escrow_account.initializer_amount = initializer_amount;
        ctx.accounts.escrow_account.taker_amount = taker_amount;

        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], ctx.program_id);
        let set_authority_accounts = SetAuthority {
            account_or_mint: ctx
                .accounts
                .initializer_deposit_token_account
                .to_account_info()
                .clone(),
            current_authority: ctx.accounts.initializer.clone(),
        };
        let cpi_context =
            CpiContext::new(ctx.accounts.token_program.clone(), set_authority_accounts);

        token::set_authority(cpi_context, AuthorityType::AccountOwner, Some(pda))?;
        Ok(())
    }

    pub fn cancel_escrow(ctx: Context<CancelEscrow>) -> ProgramResult {
        let (_pda, bump_seed) = Pubkey::find_program_address(&[b"escrow"], ctx.program_id);
        let set_authority_accounts = SetAuthority {
            account_or_mint: ctx
                .accounts
                .pda_deposit_token_account
                .to_account_info()
                .clone(),
            current_authority: ctx.accounts.pda_account.clone(),
        };

        let seeds = &[&b"escrow"[..], &[bump_seed]];
        let signer = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.clone(),
            set_authority_accounts,
            signer,
        );

        token::set_authority(
            cpi_context,
            AuthorityType::AccountOwner,
            Some(ctx.accounts.escrow_account.initializer_key),
        )?;

        Ok(())
    }

    pub fn exchange(ctx: Context<Exchange>) -> ProgramResult {
        // Transferring from initializer to taker
        let (_pda, bump_seed) = Pubkey::find_program_address(&[b"escrow"], ctx.program_id);

        let transfer_to_taker_accounts = Transfer {
            from: ctx
                .accounts
                .pda_deposit_token_account
                .to_account_info()
                .clone(),
            to: ctx
                .accounts
                .taker_receive_token_account
                .to_account_info()
                .clone(),
            authority: ctx.accounts.pda_account.clone(),
        };

        let seeds = &[&b"escrow"[..], &[bump_seed]];
        let signer = &[&seeds[..]];

        let transfer_to_taker_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.clone(),
            transfer_to_taker_accounts,
            signer,
        );

        token::transfer(
            transfer_to_taker_context,
            ctx.accounts.pda_deposit_token_account.amount,
        )?;

        // Transferring from taker to initializer
        let transfer_to_initializer_accounts = Transfer {
            from: ctx
                .accounts
                .taker_deposit_token_account
                .to_account_info()
                .clone(),
            to: ctx
                .accounts
                .initializer_receive_token_account
                .to_account_info()
                .clone(),
            authority: ctx.accounts.taker.clone(),
        };

        let transfer_to_initializer_context = CpiContext::new(
            ctx.accounts.token_program.clone(),
            transfer_to_initializer_accounts,
        );
        token::transfer(
            transfer_to_initializer_context,
            ctx.accounts.escrow_account.taker_amount,
        )?;

        // Set authority back to the initial owner
        let set_authority_accounts = SetAuthority {
            account_or_mint: ctx
                .accounts
                .pda_deposit_token_account
                .to_account_info()
                .clone(),
            current_authority: ctx.accounts.pda_account.clone(),
        };
        let set_authority_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.clone(),
            set_authority_accounts,
            signer,
        );
        token::set_authority(
            set_authority_context,
            AuthorityType::AccountOwner,
            Some(ctx.accounts.escrow_account.initializer_key),
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(initializer_amount: u64)]
pub struct InitializeEscrow<'info> {
    #[account(signer)]
    pub initializer: AccountInfo<'info>,
    #[account(mut,
        constraint = initializer_deposit_token_account.amount >= initializer_amount
    )]
    pub initializer_deposit_token_account: CpiAccount<'info, TokenAccount>,
    pub initializer_receive_token_account: CpiAccount<'info, TokenAccount>,
    #[account(init)]
    pub escrow_account: ProgramAccount<'info, EscrowAccount>,
    pub token_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Exchange<'info> {
    #[account(signer)]
    pub taker: AccountInfo<'info>,
    #[account(mut)]
    pub taker_deposit_token_account: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub taker_receive_token_account: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub pda_deposit_token_account: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub initializer_receive_token_account: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub initializer_main_account: AccountInfo<'info>,
    #[account(mut,
        constraint = escrow_account.taker_amount <= taker_deposit_token_account.amount,
        constraint = escrow_account.initializer_deposit_token_account == *pda_deposit_token_account.to_account_info().key,
        constraint = escrow_account.initializer_receive_token_account == *initializer_receive_token_account.to_account_info().key,
        constraint = escrow_account.initializer_key == *initializer_main_account.key,
        close = initializer_main_account
    )]
    pub escrow_account: ProgramAccount<'info, EscrowAccount>,
    pub pda_account: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CancelEscrow<'info> {
    pub initializer: AccountInfo<'info>,
    #[account(mut)]
    pub pda_deposit_token_account: CpiAccount<'info, TokenAccount>,
    pub pda_account: AccountInfo<'info>,
    #[account(mut,
        constraint = escrow_account.initializer_key == *initializer.key,
        constraint = escrow_account.initializer_deposit_token_account == *pda_deposit_token_account.to_account_info().key,
        close = initializer
    )]
    pub escrow_account: ProgramAccount<'info, EscrowAccount>,
    pub token_program: AccountInfo<'info>,
}

#[account]
pub struct EscrowAccount {
    pub initializer_key: Pubkey,
    pub initializer_deposit_token_account: Pubkey,
    pub initializer_receive_token_account: Pubkey,
    pub initializer_amount: u64,
    pub taker_amount: u64,
}
