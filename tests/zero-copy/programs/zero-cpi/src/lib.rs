use anchor_lang::prelude::*;
use zero_copy::cpi::accounts::{UpdateBar, UpdateFoo};
use zero_copy::program::ZeroCopy;
use zero_copy::{self, Bar, Foo};

declare_id!("ErjUjtqKE5AGWUsjseSJCVLtddM6rhaMbDqmhzraF9h6");

#[program]
pub mod zero_cpi {
    use super::*;
    pub fn check_cpi(ctx: Context<CheckCpi>, data: u64) -> ProgramResult {
        let cpi_program = ctx.accounts.zero_copy_program.to_account_info();
        let cpi_accounts = UpdateBar {
            authority: ctx.accounts.authority.clone(),
            bar: ctx.accounts.bar.to_account_info(),
            foo: ctx.accounts.foo.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        zero_copy::cpi::update_bar(cpi_ctx, data);
        Ok(())
    }

    pub fn update_foo_via_cpi(ctx: Context<UpdateFooViaCpi>, data: u64) -> ProgramResult {
        let cpi_program = ctx.accounts.zero_copy_program.to_account_info();
        let cpi_accounts = UpdateFoo {
            authority: ctx.accounts.authority.clone(),
            foo: ctx.accounts.foo.to_account_info(),
        };
        let foo = &ctx.accounts.foo;
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        zero_copy::cpi::update_foo(cpi_ctx, data);
        emit!(MyEvent { data: foo.data });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CheckCpi<'info> {
    #[account(
        mut,
        has_one = authority,
    )]
    bar: AccountLoader<'info, Bar>,
    #[account(signer)]
    authority: AccountInfo<'info>,
    foo: AccountLoader<'info, Foo>,
    zero_copy_program: Program<'info, ZeroCopy>,
}

#[derive(Accounts)]
pub struct UpdateFooViaCpi<'info> {
    #[account(mut, has_one = authority)]
    foo: AccountLoader<'info, Foo>,
    #[account(signer)]
    authority: AccountInfo<'info>,
    zero_copy_program: Program<'info, ZeroCopy>,
}

#[event]
pub struct MyEvent {
    data: u64,
}
