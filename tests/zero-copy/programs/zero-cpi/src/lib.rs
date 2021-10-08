use anchor_lang::prelude::*;
use zero_copy::program::ZeroCopy;
use zero_copy::{self, Bar, Foo, UpdateBar};

declare_id!("ErjUjtqKE5AGWUsjseSJCVLtddM6rhaMbDqmhzraF9h6");

#[program]
pub mod zero_cpi {
    use super::*;
    pub fn check_cpi(ctx: Context<CheckCpi>, data: u64) -> ProgramResult {
        let cpi_program = ctx.accounts.zero_copy_program.to_account_info();
        let cpi_accounts = UpdateBar {
            authority: ctx.accounts.authority.clone(),
            bar: ctx.accounts.bar.clone(),
            foo: ctx.accounts.foo.clone(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        zero_copy::cpi::update_bar(cpi_ctx, data);
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
