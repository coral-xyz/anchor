//! This example demonstrates the ability to use optional accounts in
//! structs deriving `Accounts`.

use anchor_lang::prelude::*;
use anchor_lang::AccountsClose;
pub use context::*;

pub mod account;
pub mod context;
declare_id!("FNqz6pqLAwvMSds2FYjR4nKV3moVpPNtvkfGFrqLKrgG");

#[program]
mod optional {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, value: u64, key: Pubkey) -> Result<()> {
        let optional_pda = &mut ctx.accounts.optional_pda;
        let optional_account = &mut ctx.accounts.optional_account;
        let required = &mut ctx.accounts.required;

        required.data = 0;

        if let Some(data_account) = optional_account {
            data_account.data = value;
        }

        if let Some(data_pda) = optional_pda {
            if let Some(data_account) = optional_account {
                data_pda.data_account = data_account.key();
            } else {
                data_pda.data_account = key;
            }
        }

        Ok(())
    }

    pub fn update(ctx: Context<Update>, value: u64, key: Pubkey, _pda_bump: u8) -> Result<()> {
        if let Some(data_account) = &mut ctx.accounts.optional_account {
            data_account.data = value;
        }
        if let Some(data_account) = &mut ctx.accounts.optional_pda {
            data_account.data_account = key;
        }
        Ok(())
    }

    pub fn realloc(ctx: Context<Realloc>) -> Result<()> {
        let optional_pda = &ctx.accounts.optional_pda;
        if let Some(acc) = optional_pda {
            let len = acc.to_account_info().data_len();
            if len != 50 {
                return err!(OptionalErrors::ReallocFailed);
            }
        }
        Ok(())
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        if let Some(data_pda) = &ctx.accounts.optional_pda {
            data_pda.close(ctx.accounts.payer.as_ref().unwrap().to_account_info())?;
        }
        Ok(())
    }
}

#[error_code]
pub enum OptionalErrors {
    #[msg("Failed realloc")]
    ReallocFailed,
}
