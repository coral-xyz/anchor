use anchor_lang::prelude::*;

#[program]
pub mod zero_copy {
    use super::*;

    pub fn create_foo(ctx: Context<CreateFoo>) -> ProgramResult {
        let foo = &mut ctx.accounts.foo.load_init()?;
        foo.authority = *ctx.accounts.authority.key;
        foo.set_second_authority(ctx.accounts.authority.key);
        Ok(())
    }

    pub fn update_foo(ctx: Context<UpdateFoo>, data: u64) -> ProgramResult {
        let mut foo = ctx.accounts.foo.load_mut()?;
        foo.data = data;
        Ok(())
    }

    pub fn update_foo_second(ctx: Context<UpdateFooSecond>, second_data: u64) -> ProgramResult {
        let mut foo = ctx.accounts.foo.load_mut()?;
        foo.second_data = second_data;
        Ok(())
    }

    pub fn create_bar(ctx: Context<CreateBar>) -> ProgramResult {
        let bar = &mut ctx.accounts.bar.load_init()?;
        bar.authority = *ctx.accounts.authority.key;
        Ok(())
    }

    pub fn update_bar(ctx: Context<UpdateBar>, data: u64) -> ProgramResult {
        let bar = &mut ctx.accounts.bar.load_mut()?;
        bar.data = data;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateFoo<'info> {
    #[account(init)]
    foo: ProgramAccountZeroCopy<'info, Foo>,
    #[account(signer)]
    authority: AccountInfo<'info>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdateFoo<'info> {
    #[account(mut, has_one = authority)]
    foo: ProgramAccountZeroCopy<'info, Foo>,
    #[account(signer)]
    authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct UpdateFooSecond<'info> {
    #[account(mut, "&foo.load()?.get_second_authority() == second_authority.key")]
    foo: ProgramAccountZeroCopy<'info, Foo>,
    #[account(signer)]
    second_authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CreateBar<'info> {
    #[account(associated = authority, with = foo)]
    bar: ProgramAccountZeroCopy<'info, Bar>,
    #[account(signer)]
    authority: AccountInfo<'info>,
    foo: ProgramAccountZeroCopy<'info, Foo>,
    rent: Sysvar<'info, Rent>,
    system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct UpdateBar<'info> {
    #[account(mut, has_one = authority)]
    bar: ProgramAccountZeroCopy<'info, Bar>,
    #[account(signer)]
    authority: AccountInfo<'info>,
}

#[account(zero_copy)]
pub struct Foo {
    pub authority: Pubkey,
    pub data: u64,
    pub second_data: u64,
    #[accessor(Pubkey)]
    pub second_authority: [u8; 32],
}

#[associated(zero_copy)]
pub struct Bar {
    pub authority: Pubkey,
    pub data: u64,
}
