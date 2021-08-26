//! A cashiers check example. The funds are immediately withdrawn from a user's
//! account and sent to a program controlled `Check` account, where the funds
//! reside until they are "cashed" by the intended recipient. The creator of
//! the check can cancel the check at any time to get back the funds.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};
use std::convert::Into;

#[program]
pub mod cashiers_check {
    use super::*;

    #[access_control(CreateCheck::accounts(&ctx, nonce))]
    pub fn create_check(
        ctx: Context<CreateCheck>,
        amount: u64,
        memo: Option<String>,
        nonce: u8,
    ) -> Result<()> {
        // Transfer funds to the check.
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info().clone(),
            to: ctx.accounts.vault.to_account_info().clone(),
            authority: ctx.accounts.owner.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Print the check.
        let check = &mut ctx.accounts.check;
        check.amount = amount;
        check.from = *ctx.accounts.from.to_account_info().key;
        check.to = *ctx.accounts.to.to_account_info().key;
        check.vault = *ctx.accounts.vault.to_account_info().key;
        check.nonce = nonce;
        check.memo = memo;

        Ok(())
    }

    #[access_control(not_burned(&ctx.accounts.check))]
    pub fn cash_check(ctx: Context<CashCheck>) -> Result<()> {
        let seeds = &[
            ctx.accounts.check.to_account_info().key.as_ref(),
            &[ctx.accounts.check.nonce],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault.to_account_info().clone(),
            to: ctx.accounts.to.to_account_info().clone(),
            authority: ctx.accounts.check_signer.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, ctx.accounts.check.amount)?;
        // Burn the check for one time use.
        ctx.accounts.check.burned = true;
        Ok(())
    }

    #[access_control(not_burned(&ctx.accounts.check))]
    pub fn cancel_check(ctx: Context<CancelCheck>) -> Result<()> {
        let seeds = &[
            ctx.accounts.check.to_account_info().key.as_ref(),
            &[ctx.accounts.check.nonce],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault.to_account_info().clone(),
            to: ctx.accounts.from.to_account_info().clone(),
            authority: ctx.accounts.check_signer.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, ctx.accounts.check.amount)?;
        ctx.accounts.check.burned = true;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateCheck<'info> {
    // Check being created.
    #[account(zero)]
    check: ProgramAccount<'info, Check>,
    // Check's token vault.
    #[account(mut, "&vault.owner == check_signer.key")]
    vault: CpiAccount<'info, TokenAccount>,
    // Program derived address for the check.
    check_signer: AccountInfo<'info>,
    // Token account the check is made from.
    #[account(mut, has_one = owner)]
    from: CpiAccount<'info, TokenAccount>,
    // Token account the check is made to.
    #[account("from.mint == to.mint")]
    to: CpiAccount<'info, TokenAccount>,
    // Owner of the `from` token account.
    owner: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
}

impl<'info> CreateCheck<'info> {
    pub fn accounts(ctx: &Context<CreateCheck>, nonce: u8) -> Result<()> {
        let signer = Pubkey::create_program_address(
            &[ctx.accounts.check.to_account_info().key.as_ref(), &[nonce]],
            ctx.program_id,
        )
        .map_err(|_| ErrorCode::InvalidCheckNonce)?;
        if &signer != ctx.accounts.check_signer.to_account_info().key {
            return Err(ErrorCode::InvalidCheckSigner.into());
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CashCheck<'info> {
    #[account(mut, has_one = vault, has_one = to)]
    check: ProgramAccount<'info, Check>,
    #[account(mut)]
    vault: AccountInfo<'info>,
    #[account(seeds = [
        check.to_account_info().key.as_ref(),
        &[check.nonce],
    ])]
    check_signer: AccountInfo<'info>,
    #[account(mut, has_one = owner)]
    to: CpiAccount<'info, TokenAccount>,
    #[account(signer)]
    owner: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CancelCheck<'info> {
    #[account(mut, has_one = vault, has_one = from)]
    check: ProgramAccount<'info, Check>,
    #[account(mut)]
    vault: AccountInfo<'info>,
    #[account(seeds = [
        check.to_account_info().key.as_ref(),
        &[check.nonce],
    ])]
    check_signer: AccountInfo<'info>,
    #[account(mut, has_one = owner)]
    from: CpiAccount<'info, TokenAccount>,
    #[account(signer)]
    owner: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
}

#[account]
pub struct Check {
    from: Pubkey,
    to: Pubkey,
    amount: u64,
    memo: Option<String>,
    vault: Pubkey,
    nonce: u8,
    burned: bool,
}

#[error]
pub enum ErrorCode {
    #[msg("The given nonce does not create a valid program derived address.")]
    InvalidCheckNonce,
    #[msg("The derived check signer does not match that which was given.")]
    InvalidCheckSigner,
    #[msg("The given check has already been burned.")]
    AlreadyBurned,
}

fn not_burned(check: &Check) -> Result<()> {
    if check.burned {
        return Err(ErrorCode::AlreadyBurned.into());
    }
    Ok(())
}
