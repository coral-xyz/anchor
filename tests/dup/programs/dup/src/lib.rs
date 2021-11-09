use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
mod dup {
    use super::*;

    pub fn with_dup_constraint(_ctx: Context<WithDupConstraint>) -> ProgramResult {
        Ok(())
    }

    pub fn without_dup_constraint(_ctx: Context<WithoutDupConstraint>) -> ProgramResult {
        Ok(())
    }

    pub fn with_missing_dup_constraints_3_accounts(_ctx: Context<WithMissingDupConstraint3Accounts>) -> ProgramResult {
        Ok(())
    }

    pub fn with_dup_constraints_3_accounts(_ctx: Context<WithDupConstraint3Accounts>) -> ProgramResult {
        Ok(())
    }

    pub fn with_missing_dup_constraint_double_3_accounts(_ctx: Context<WithMissingDupConstraintDouble3Accounts>) -> ProgramResult {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct WithDupConstraint<'info> {
    pub authority: Signer<'info>,
    #[account(dup = authority)]
    pub wallet: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct WithoutDupConstraint<'info> {
    pub my_account: Account<'info, MyAccount>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct WithMissingDupConstraint3Accounts<'info> {
    pub my_account: Account<'info, MyAccount>,
    #[account(dup = my_account)]
    pub rent: Sysvar<'info, Rent>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithDupConstraint3Accounts<'info> {
    pub my_account: Account<'info, MyAccount>,
    #[account(dup = my_account)]
    pub rent: Sysvar<'info, Rent>,
    #[account(dup = my_account)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithMissingDupConstraintDouble3Accounts<'info> {
    pub my_account: Account<'info, MyAccount>,
    #[account(dup = my_account)]
    pub rent: Sysvar<'info, Rent>,
    #[account(dup = my_account)]
    pub authority: Signer<'info>,
    pub my_account_1: Account<'info, MyAccount>,
    #[account(dup = my_account_1)]
    pub rent_1: Sysvar<'info, Rent>,
    pub authority_1: Signer<'info>,
}

#[account]
pub struct MyAccount {
    data: u64
}
