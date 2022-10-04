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

        required.data_pda = Pubkey::default();

        if let Some(data) = optional_pda {
            data.data = value;
        }

        if let Some(data2) = optional_account {
            if let Some(optional) = optional_pda {
                data2.data_pda = optional.key();
            } else {
                data2.data_pda = key;
            }
        }

        Ok(())
    }

    pub fn update(ctx: Context<Update>, value: u64, key: Pubkey, _pda_bump: u8) -> Result<()> {
        if let Some(data_pda) = &mut ctx.accounts.optional_pda {
            data_pda.data = value;
        }
        if let Some(data_account) = &mut ctx.accounts.optional_account {
            data_account.data_pda = key;
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
        if let Some(data_account) = &ctx.accounts.data_pda {
            data_account.close(ctx.accounts.payer.as_ref().unwrap().to_account_info())?;
        }
        Ok(())
    }
}

#[error_code]
pub enum OptionalErrors {
    #[msg("Failed realloc")]
    ReallocFailed,
}
