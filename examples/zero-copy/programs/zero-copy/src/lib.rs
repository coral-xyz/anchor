use anchor_lang::prelude::*;

#[program]
pub mod zero_copy {
    use super::*;

    pub fn create_foo(ctx: Context<CreateFoo>) -> ProgramResult {
        let mut foo = &mut ctx.accounts.foo;
        foo.set_authority(ctx.accounts.authority);
        Ok(())
    }

    pub fn update_foo(ctx: Context<UpdateFoo>, data: u64, more_data: u64) -> ProgramResult {
        let mut foo = &mut ctx.accounts.foo;
        foo.data = data;
        foo.more_data = more_data;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateFoo {
    #[account(init)]
    foo: ProgramAccountZeroCopy<'info, Foo>,
    #[account(signer)]
    authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct UpdateFoo {
    #[account(mut, has_one = authority)]
    foo: ProgramAccountZeroCopy<'info, Foo>,
    #[account(signer)]
    authority: AccountInfo<'info>,
}

#[account(zero_copy)]
pub struct Foo {
    #[accessor(Pubkey)]
    pub authority: [u64; 4],
    pub data: u64,
    pub more_data: u64,
}
