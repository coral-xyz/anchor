use anchor_lang::prelude::*;
use std::mem::size_of;
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod remaining_accounts {
    use super::*;

    pub fn deposit(ctx: Context<Deposit>,
        bump: u8,
        amount: u64,
    ) -> Result<()> {
        let bank = &mut ctx.accounts.bank;
        let payer = &mut ctx.accounts.payer;

        bank.bump = bump;
        bank.amount = amount;

       
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &payer.to_account_info().key(),
            &bank.to_account_info().key(),
            bank.amount,
        
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                payer.to_account_info(),
                bank.to_account_info(),
            ],
        ).ok();
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let bank = &mut ctx.accounts.bank;
        let num_accounts = ctx.remaining_accounts.len() as u64;
        let amount = bank.amount/num_accounts;

        for acc in ctx.remaining_accounts {
            let from_lamports = bank.to_account_info().lamports();
            let dest_lamports = acc.to_account_info().lamports();

            **acc.to_account_info().lamports.borrow_mut() = dest_lamports.checked_add(amount).unwrap();
            **bank.to_account_info().lamports.borrow_mut() = from_lamports.checked_sub(amount).unwrap();

        }
        
        Ok(())
    }

}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Deposit<'info> {
    #[account( 
        init,
        seeds = [b"bank".as_ref(), payer.key().as_ref()],
        bump,
        payer = payer,
        space = size_of::<InitAcc>() + 32)]
    bank: Account<'info, InitAcc>,
    #[account(mut)]
    payer: Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, owner = *program_id)]
    bank: Account<'info, InitAcc>,
    system_program: Program<'info, System>
}

#[account]
#[derive(Default)]
pub struct InitAcc {
    bump: u8,
    amount: u64,
}
