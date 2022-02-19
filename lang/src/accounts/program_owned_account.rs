//! Type validating that the account is owned by the given program

use crate::error::ErrorCode;
use crate::*;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::system_program;
use std::collections::BTreeMap;
use std::ops::Deref;

/// Type validating that the account is owned by the system program
///
/// Checks:
///
/// - `ProgramOwnedAccount.info.owner == SystemProgram`
#[derive(Debug, Clone)]
pub struct ProgramOwnedAccount<'info, T: Id + Clone> {
    info: AccountInfo<'info>,
}

impl<'info, T: Id + Clone> ProgramOwnedAccount<'info, T> {
    fn new(info: AccountInfo<'info>) -> ProgramOwnedAccount<'info> {
        Self { info }
    }

    #[inline(never)]
    pub fn try_from(info: &AccountInfo<'info>) -> Result<ProgramOwnedAccount<'info>, ProgramError> {
        if info.owner != &T::id() {
            return Err(ErrorCode::AccountOwnedByWrongProgram.into());
        }
        Ok(ProgramOwnedAccount::new(info.clone()))
    }
}

impl<'info, T: Id + Clone> Accounts<'info> for ProgramOwnedAccount<'info, T> {
    #[inline(never)]
    fn try_accounts(
        _program_id: &Pubkey,
        accounts: &mut &[AccountInfo<'info>],
        _ix_data: &[u8],
        _bumps: &mut BTreeMap<String, u8>,
    ) -> Result<Self, ProgramError> {
        if accounts.is_empty() {
            return Err(ErrorCode::AccountNotEnoughKeys.into());
        }
        let account = &accounts[0];
        *accounts = &accounts[1..];
        ProgramOwnedAccount::try_from(account)
    }
}

impl<'info, T: Id + Clone> AccountsExit<'info> for ProgramOwnedAccount<'info, T> {}

impl<'info, T: Id + Clone> ToAccountMetas for ProgramOwnedAccount<'info, T> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        let is_signer = is_signer.unwrap_or(self.info.is_signer);
        let meta = match self.info.is_writable {
            false => AccountMeta::new_readonly(*self.info.key, is_signer),
            true => AccountMeta::new(*self.info.key, is_signer),
        };
        vec![meta]
    }
}

impl<'info, T: Id + Clone> ToAccountInfos<'info> for ProgramOwnedAccount<'info, T> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        vec![self.info.clone()]
    }
}

impl<'info, T: Id + Clone> AsRef<AccountInfo<'info>> for ProgramOwnedAccount<'info, T> {
    fn as_ref(&self) -> &AccountInfo<'info> {
        &self.info
    }
}

impl<'info, T: Id + Clone> Deref for ProgramOwnedAccount<'info, T> {
    type Target = AccountInfo<'info>;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}
