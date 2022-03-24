use anchor_lang::prelude::*;

declare_id!("HmbTLCmaGvZhKnn1Zfa1JVnp7vkMV4DYVxPLWBVoN65L");

mod account;

use account::*;

#[program]
pub mod misc2 {
    use super::*;

    #[state]
    pub struct MyState {
        pub data: u64,
        pub auth: Pubkey,
    }

    impl MyState {
        pub fn new(ctx: Context<Auth>) -> Result<Self> {
            Ok(Self {
                data: 0,
                auth: *ctx.accounts.authority.key,
            })
        }

        pub fn set_data(&mut self, ctx: Context<Auth>, data: u64) -> Result<()> {
            if self.auth != *ctx.accounts.authority.key {
                return Err(ProgramError::Custom(1234).into()); // Arbitrary error code.
            }
            self.data = data;
            Ok(())
        }
    }

    // this exists so we can test whether it compiles
    // it's not used in a test
    pub fn empty_accounts_struct_cpi(_ctx: Context<EmptyAccountsStructCpi>) -> Result<()> {
        Ok(())
    }

    // this exists so we can test whether it compiles
    // it's not used in a test
    pub fn accounts_struct_from_other_module(_ctx: Context<StructFromOtherModule>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Auth<'info> {
    #[account(signer)]
    /// CHECK:
    pub authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct EmptyAccountsStructCpi{}
