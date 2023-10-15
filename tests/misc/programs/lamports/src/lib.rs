use anchor_lang::prelude::*;

declare_id!("Lamports11111111111111111111111111111111111");

#[program]
pub mod lamports {
    use super::*;

    pub fn test_lamports_trait(ctx: Context<TestLamportsTrait>, amount: u64) -> Result<()> {
        let pda = &ctx.accounts.pda;
        let signer = &ctx.accounts.signer;

        // Transfer **to** PDA
        {
            // Get the balance of the PDA **before** the transfer to PDA
            let pda_balance_before = pda.get_lamports();

            // Transfer to the PDA
            anchor_lang::system_program::transfer(
                CpiContext::new(
                    ctx.accounts.system_program.to_account_info(),
                    anchor_lang::system_program::Transfer {
                        from: signer.to_account_info(),
                        to: pda.to_account_info(),
                    },
                ),
                amount,
            )?;

            // Get the balance of the PDA **after** the transfer to PDA
            let pda_balance_after = pda.get_lamports();

            // Validate balance
            require_eq!(pda_balance_after, pda_balance_before + amount);
        }

        // Transfer **from** PDA
        {
            // Get the balance of the PDA **before** the transfer from PDA
            let pda_balance_before = pda.get_lamports();

            // Transfer from the PDA
            pda.sub_lamports(amount)?;
            signer.add_lamports(amount)?;

            // Get the balance of the PDA **after** the transfer from PDA
            let pda_balance_after = pda.get_lamports();

            // Validate balance
            require_eq!(pda_balance_after, pda_balance_before - amount);
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct TestLamportsTrait<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = 8,
        seeds = [b"lamports"],
        bump
    )]
    pub pda: Account<'info, LamportsPda>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct LamportsPda {}
