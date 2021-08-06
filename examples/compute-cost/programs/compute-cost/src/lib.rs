use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};


#[program]
pub mod compute_cost {
    use super::*;
    pub fn transfer_please(ctx: Context<TransferPlease>, num_transfers: u8) -> ProgramResult {
        msg!("number of transfers is {}", num_transfers);
        for _n in 0..num_transfers{
            {
                let cpi_accounts = Transfer {
                    from: ctx.accounts.from_usdc.to_account_info(),
                    to: ctx.accounts.to_usdc.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.clone();
                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
                token::transfer(cpi_ctx, 1000000 as u64)?;
            }
        };
        Ok(())
    }

    pub fn init_signer(ctx: Context<InitSigner>, nonce: u8) -> ProgramResult {
        let pda_signer = &mut ctx.accounts.pda_signer;
        pda_signer.nonce = nonce;
        Ok(())
    }

    pub fn signed_transfer(ctx: Context<SignedTransfer>, num_transfers: u8) -> ProgramResult {
        msg!("number of transfers is {}", num_transfers);
        for _n in 0..num_transfers{
            {
                let seeds = &[
                    ctx.accounts.authority.key.as_ref(),
                    &[ctx.accounts.pda_signer.nonce],
                ];
                let signer = &[&seeds[..]];
                let cpi_accounts = Transfer {
                    from: ctx.accounts.to_usdc.to_account_info(),
                    to: ctx.accounts.from_usdc.to_account_info(),
                    authority: ctx.accounts.pda_signer.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.clone();
                let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
                token::transfer(cpi_ctx, 1000000 as u64)?;
            }
        };
        Ok(())
    }
}


#[derive(Accounts)]
pub struct TransferPlease <'info> {
    #[account(mut)]
    pub from_usdc: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub to_usdc: CpiAccount<'info, TokenAccount>,
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct InitSigner <'info> {
    #[account(init, seeds = [authority.key.as_ref(), &[nonce]], payer = authority)]
    pub pda_signer: ProgramAccount<'info, PdaSigner>,
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}


#[derive(Accounts)]
pub struct SignedTransfer <'info> {
    #[account(seeds = [authority.key.as_ref(), &[pda_signer.nonce]])]
    pub pda_signer: ProgramAccount<'info, PdaSigner>,
    #[account(mut)]
    pub from_usdc: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub to_usdc: CpiAccount<'info, TokenAccount>,
    pub authority: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}


#[account]
#[derive(Default)]
pub struct PdaSigner {
    pub nonce: u8
}