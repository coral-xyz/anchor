use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
mod dup {
    use super::*;

    pub fn with_dup_constraint(_ctx: Context<WithDupConstraint>) -> ProgramResult {
        Ok(())
    }

    pub fn with_dup_constraint_composite(_ctx: Context<WithDupConstraintComposite>) -> ProgramResult {
        Ok(())
    }

    pub fn without_dup_constraint(_ctx: Context<WithoutDupConstraint>) -> ProgramResult {
        Ok(())
    }

    pub fn without_dup_constraint_composite(_ctx: Context<WithoutDupConstraintComposite>) -> ProgramResult {
        Ok(())
    }

    pub fn with_missing_dup_constraints_three_accounts(_ctx: Context<WithMissingDupConstraintThreeAccounts>) -> ProgramResult {
        Ok(())
    }

    pub fn with_dup_constraints_three_accounts(_ctx: Context<WithDupConstraintsThreeAccounts>) -> ProgramResult {
        Ok(())
    }

    pub fn with_missing_dup_constraint_double_three_accounts(_ctx: Context<WithMissingDupConstraintDoubleThreeAccounts>) -> ProgramResult {
        Ok(())
    }

    pub fn without_dup_constraint_double_three_accounts_all_immutable(_ctx: Context<WithoutDupConstraintDoubleThreeAccountsAllImmutable>) -> ProgramResult {
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
pub struct WithDupConstraintComposite<'info> {
    #[account(dup = child.child_account)]
    pub account1: SystemAccount<'info>,
    pub child: Child<'info>
}

#[derive(Accounts)]
pub struct Child<'info> {
    pub child_account: SystemAccount<'info>
}

#[derive(Accounts)]
pub struct WithoutDupConstraint<'info> {
    pub my_account: SystemAccount<'info>,
    #[account(mut)]
    pub rent: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct WithoutDupConstraintComposite<'info> {
    pub child: Child<'info>,
    #[account(mut)]
    pub account1: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct WithMissingDupConstraintThreeAccounts<'info> {
    pub my_account: SystemAccount<'info>,
    #[account(dup = my_account, mut)]
    pub rent: SystemAccount<'info>,
    pub authority: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct WithDupConstraintsThreeAccounts<'info> {
    pub my_account: SystemAccount<'info>,
    #[account(dup = my_account)]
    pub rent: SystemAccount<'info>,
    #[account(mut, dup = my_account)]
    pub authority: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct WithMissingDupConstraintDoubleThreeAccounts<'info> {
    pub my_account: SystemAccount<'info>,
    #[account(dup = my_account)]
    pub rent: SystemAccount<'info>,
    #[account(dup = my_account)]
    pub authority: SystemAccount<'info>,
    pub my_account_1: SystemAccount<'info>,
    #[account(dup = my_account_1)]
    pub rent_1: SystemAccount<'info>,
    #[account(mut)]
    pub authority_1: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct WithoutDupConstraintDoubleThreeAccountsAllImmutable<'info> {
    pub my_account: SystemAccount<'info>,
    pub rent: SystemAccount<'info>,
    pub authority: SystemAccount<'info>,
    pub my_account_1: SystemAccount<'info>,
    pub rent_1: SystemAccount<'info>,
    pub authority_1: SystemAccount<'info>,
}
