use anchor_lang::prelude::*;

#[program]
pub mod zero_copy {
    use super::*;

    pub fn create_foo(ctx: Context<CreateFoo>) -> ProgramResult {
        let foo = &mut ctx.accounts.foo.load_mut()?;
        foo.set_authority(ctx.accounts.authority.key);
        Ok(())
    }

    pub fn update_foo(ctx: Context<UpdateFoo>, data: u64, more_data: u64) -> ProgramResult {
        let mut foo = ctx.accounts.foo.load_mut()?;
        foo.data = data;
        foo.more_data = more_data;
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
    #[account(mut)] // todo has_one = authority
    foo: ProgramAccountZeroCopy<'info, Foo>,
    #[account(signer)]
    authority: AccountInfo<'info>,
}

#[account(zero_copy)]
pub struct Foo {
    #[accessor(Pubkey)]
    pub authority: [u8; 32],
    pub data: u64,
    pub more_data: u64,
    #[accessor(Pubkey)]
    pub second_authority: [u8; 32],
}
