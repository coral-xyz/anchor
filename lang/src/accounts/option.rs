//! Option<T> type for optional accounts.
//!
//! # Example
//! ```ignore
//! #[derive(Accounts)]
//! pub struct Example {
//!     pub my_acc: Option<Account<'info, MyData>>
//! }
//! ```

use std::collections::{BTreeMap, BTreeSet};

use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

use anchor_lang::error::ErrorCode;

use crate::{Accounts, AccountsClose, AccountsExit, Result, ToAccountInfos, ToAccountMetas};

impl<'info, T: Accounts<'info>> Accounts<'info> for Option<T> {
    fn try_accounts(
        program_id: &Pubkey,
        accounts: &mut &[AccountInfo<'info>],
        ix_data: &[u8],
        bumps: &mut BTreeMap<String, u8>,
        reallocs: &mut BTreeSet<Pubkey>,
    ) -> Result<Self> {
        if accounts.is_empty() {
            return Err(ErrorCode::AccountNotEnoughKeys.into());
        }
        let account = &accounts[0];
        if account.key == program_id {
            *accounts = &accounts[1..];
            Ok(None)
        } else {
            T::try_accounts(program_id, accounts, ix_data, bumps, reallocs).map(Some)
        }
    }
}

impl<'info, T: AccountsExit<'info>> AccountsExit<'info> for Option<T> {
    fn exit(&self, program_id: &Pubkey) -> Result<()> {
        self.as_ref()
            .expect("Cannot run `exit` on None")
            .exit(program_id)
    }
}

impl<'info, T: ToAccountInfos<'info>> ToAccountInfos<'info> for Option<T> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        self.as_ref()
            .expect("Cannot run `to_account_infos` on None")
            .to_account_infos()
    }
}

impl<T: ToAccountMetas> ToAccountMetas for Option<T> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        self.as_ref()
            .expect("Cannot run `to_account_metas` on None")
            .to_account_metas(is_signer)
    }
}

impl<'info, T: AccountsClose<'info>> AccountsClose<'info> for Option<T> {
    fn close(&self, sol_destination: AccountInfo<'info>) -> Result<()> {
        self.as_ref()
            .expect("Cannot run `close` on None")
            .close(sol_destination)
    }
}
