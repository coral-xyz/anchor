use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
mod dup {
    use super::*;

    pub fn with_dup_constraint(_ctx: Context<WithDupConstraint>) -> ProgramResult {
        Ok(())
    }

    pub fn with_dup_constraint_composite(
        _ctx: Context<WithDupConstraintComposite>,
    ) -> ProgramResult {
        Ok(())
    }

    pub fn without_dup_constraint(_ctx: Context<WithoutDupConstraint>) -> ProgramResult {
        Ok(())
    }

    pub fn without_dup_constraint_composite(
        _ctx: Context<WithoutDupConstraintComposite>,
    ) -> ProgramResult {
        Ok(())
    }

    pub fn with_missing_dup_constraints_three_accounts(
        _ctx: Context<WithMissingDupConstraintThreeAccounts>,
    ) -> ProgramResult {
        Ok(())
    }

    pub fn with_dup_constraints_three_accounts(
        _ctx: Context<WithDupConstraintsThreeAccounts>,
    ) -> ProgramResult {
        Ok(())
    }

    pub fn with_missing_dup_constraint_double_three_accounts(
        _ctx: Context<WithMissingDupConstraintDoubleThreeAccounts>,
    ) -> ProgramResult {
        Ok(())
    }

    pub fn without_dup_constraint_double_three_accounts_all_immutable(
        _ctx: Context<WithoutDupConstraintDoubleThreeAccountsAllImmutable>,
    ) -> ProgramResult {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct WithDupConstraint<'info> {
    pub account1: Signer<'info>,
    #[account(dup = account1)]
    pub account2: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct WithDupConstraintComposite<'info> {
    #[account(dup = child.child_account)]
    pub account1: SystemAccount<'info>,
    pub child: Child<'info>,
}

#[derive(Accounts)]
pub struct Child<'info> {
    pub child_account: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct WithoutDupConstraint<'info> {
    pub account1: SystemAccount<'info>,
    #[account(mut)]
    pub account2: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct WithoutDupConstraintComposite<'info> {
    pub child: Child<'info>,
    #[account(mut)]
    pub account1: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct WithMissingDupConstraintThreeAccounts<'info> {
    pub account1: SystemAccount<'info>,
    #[account(dup = account1, mut)]
    pub account2: SystemAccount<'info>,
    pub account3: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct WithDupConstraintsThreeAccounts<'info> {
    pub account1: SystemAccount<'info>,
    #[account(dup = account1)]
    pub account2: SystemAccount<'info>,
    #[account(mut, dup = account1)]
    pub account3: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct WithMissingDupConstraintDoubleThreeAccounts<'info> {
    pub account1: SystemAccount<'info>,
    #[account(dup = account1)]
    pub account2: SystemAccount<'info>,
    #[account(dup = account1)]
    pub account3: SystemAccount<'info>,
    pub account4: SystemAccount<'info>,
    #[account(dup = account4)]
    pub account5: SystemAccount<'info>,
    #[account(mut)]
    pub account6: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct WithoutDupConstraintDoubleThreeAccountsAllImmutable<'info> {
    pub account1: SystemAccount<'info>,
    pub account2: SystemAccount<'info>,
    pub account3: SystemAccount<'info>,
    pub account4: SystemAccount<'info>,
    pub account5: SystemAccount<'info>,
    pub account6: SystemAccount<'info>,
}
