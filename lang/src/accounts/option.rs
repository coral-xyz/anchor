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

use crate::{
    error::ErrorCode, Accounts, AccountsClose, AccountsExit, Result, ToAccountInfo, ToAccountInfos,
    ToAccountMetas, TryKey, TryToAccountInfo, TryToAccountInfos,
};

impl<'info, T: Accounts<'info>> Accounts<'info> for Option<T> {
    fn try_accounts(
        program_id: &Pubkey,
        accounts: &mut &[AccountInfo<'info>],
        ix_data: &[u8],
        bumps: &mut BTreeMap<String, u8>,
        reallocs: &mut BTreeSet<Pubkey>,
    ) -> Result<Self> {
        if accounts.is_empty() {
            return Ok(None);
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

impl<'info, T: ToAccountInfos<'info>> ToAccountInfos<'info> for Option<T> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        match self {
            Some(account) => account.to_account_infos(),
            None => panic!("Cannot run `to_account_infos` on None"),
        }
    }
}

impl<'info, T: ToAccountInfos<'info>> TryToAccountInfos<'info> for Option<T> {
    fn try_to_account_infos(&self, program: &AccountInfo<'info>) -> Vec<AccountInfo<'info>> {
        match self {
            Some(_) => self.to_account_infos(),
            None => vec![program.clone()],
        }
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
            .map_or(Ok(()), |t| T::close(t, sol_destination))
    }
}

impl<'info, T: AccountsExit<'info>> AccountsExit<'info> for Option<T> {
    fn exit(&self, program_id: &Pubkey) -> Result<()> {
        self.as_ref().map_or(Ok(()), |t| t.exit(program_id))
    }
}

impl<T: TryKey> TryKey for Option<T> {
    fn try_key(&self) -> Result<Pubkey> {
        self.as_ref()
            .map_or(Err(ErrorCode::TryKeyOnNone.into()), |t| t.try_key())
    }
}

impl<'info, T: ToAccountInfo<'info>> TryToAccountInfo<'info> for Option<T> {
    fn try_to_account_info(&self) -> Result<AccountInfo<'info>> {
        self.as_ref()
            .map_or(Err(ErrorCode::TryKeyOnNone.into()), |t| {
                Ok(t.to_account_info())
            })
    }
}
