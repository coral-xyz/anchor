use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
mod nodup {
    use super::*;

    pub fn with_dup_constraint(_ctx: Context<WithDupConstraint>) -> ProgramResult {
        anchor_lang::solana_program::log::sol_log_compute_units();

        Ok(())
    }

    pub fn with_dup_constraint_composite(
        _ctx: Context<WithDupConstraintComposite>,
    ) -> ProgramResult {
        anchor_lang::solana_program::log::sol_log_compute_units();

        Ok(())
    }

    pub fn without_dup_constraint(_ctx: Context<WithoutDupConstraint>) -> ProgramResult {
        anchor_lang::solana_program::log::sol_log_compute_units();

        Ok(())

    }

    pub fn without_dup_constraint_composite(
        _ctx: Context<WithoutDupConstraintComposite>,
    ) -> ProgramResult {
        anchor_lang::solana_program::log::sol_log_compute_units();

        Ok(())
    }

    pub fn without_dup_constraint_composite_reverse(
        _ctx: Context<WithoutDupConstraintCompositeReverse>,
    ) -> ProgramResult {
        anchor_lang::solana_program::log::sol_log_compute_units();

        Ok(())
    }

    pub fn without_dup_constraint_two_composites(
        _ctx: Context<WithoutDupConstraintTwoComposites>,
    ) -> ProgramResult {
        anchor_lang::solana_program::log::sol_log_compute_units();

        Ok(())
    }

    pub fn with_missing_dup_constraints_three_accounts(
        _ctx: Context<WithMissingDupConstraintThreeAccounts>,
    ) -> ProgramResult {
        anchor_lang::solana_program::log::sol_log_compute_units();

        Ok(())
    }

    pub fn with_dup_constraints_three_accounts(
        _ctx: Context<WithDupConstraintsThreeAccounts>,
    ) -> ProgramResult {
        anchor_lang::solana_program::log::sol_log_compute_units();
        Ok(())
    }

    pub fn with_missing_dup_constraint_double_three_accounts(
        _ctx: Context<WithMissingDupConstraintDoubleThreeAccounts>,
    ) -> ProgramResult {
        anchor_lang::solana_program::log::sol_log_compute_units();

        Ok(())
    }

    pub fn without_dup_constraint_double_three_accounts_all_immutable(
        _ctx: Context<WithoutDupConstraintDoubleThreeAccountsAllImmutable>,
    ) -> ProgramResult {
        anchor_lang::solana_program::log::sol_log_compute_units();

        Ok(())
    }

    pub fn nested_children(
        _ctx: Context<NestedChildren>,
    ) -> ProgramResult {
        anchor_lang::solana_program::log::sol_log_compute_units();

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Child<'info> {
    pub child_account: SystemAccount<'info>,
    pub another_child_account: SystemAccount<'info>
}

#[derive(Accounts)]
pub struct ChildMut<'info> {
    #[account(mut)]
    pub child_account: SystemAccount<'info>,
    pub another_child_account: SystemAccount<'info>
}

#[derive(Accounts)]
pub struct WithDupConstraint<'info> {
    pub account1: Signer<'info>,
    #[account(mut, dup = account1)]
    pub account2: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct WithDupConstraintComposite<'info> {
    pub child: ChildMut<'info>,
    #[account(dup = child.child_account)]
    pub account1: SystemAccount<'info>,
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
pub struct WithoutDupConstraintCompositeReverse<'info> {
    #[account(mut)]
    pub account1: SystemAccount<'info>,
    pub child: Child<'info>,
}

#[derive(Accounts)]
pub struct WithoutDupConstraintTwoComposites<'info> {
    pub account: SystemAccount<'info>,
    pub child: Child<'info>,
    pub child_two: ChildMut<'info>,
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

#[derive(Accounts)]
pub struct NestedChildren<'info> {
    pub account_one: WithDupConstraintsThreeAccounts<'info>,
    pub account_two: WithoutDupConstraint<'info>,
    pub account_three: WithoutDupConstraintCompositeReverse<'info>,
    pub account_four: WithoutDupConstraintTwoComposites<'info>,
    pub account_five: SystemAccount<'info>
}
