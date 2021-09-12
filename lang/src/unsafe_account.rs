use crate::error::ErrorCode;
use crate::{Accounts, AccountsExit, Key, ToAccountInfo, ToAccountInfos, ToAccountMetas};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::ops::Deref;

/// Explicit wrapper for AccountInfo types.
#[derive(Clone)]
pub struct UnsafeAccount<'info>(AccountInfo<'info>);

impl<'info> Accounts<'info> for UnsafeAccount<'info> {
    fn try_accounts(
        _program_id: &Pubkey,
        accounts: &mut &[AccountInfo<'info>],
        _ix_data: &[u8],
    ) -> Result<Self, ProgramError> {
        if accounts.is_empty() {
            return Err(ErrorCode::AccountNotEnoughKeys.into());
        }
        let account = &accounts[0];
        *accounts = &accounts[1..];
        Ok(UnsafeAccount(account.clone()))
    }
}

impl<'info> ToAccountMetas for UnsafeAccount<'info> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        let is_signer = is_signer.unwrap_or(self.is_signer);
        let meta = match self.is_writable {
            false => AccountMeta::new_readonly(*self.key, is_signer),
            true => AccountMeta::new(*self.key, is_signer),
        };
        vec![meta]
    }
}

impl<'info> ToAccountInfos<'info> for UnsafeAccount<'info> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        vec![self.0.clone()]
    }
}

impl<'info> ToAccountInfo<'info> for UnsafeAccount<'info> {
    fn to_account_info(&self) -> AccountInfo<'info> {
        self.0.clone()
    }
}

impl<'info> AccountsExit<'info> for UnsafeAccount<'info> {
    fn exit(&self, _program_id: &Pubkey) -> ProgramResult {
        // no-op
        Ok(())
    }
}

impl<'info> Key for UnsafeAccount<'info> {
    fn key(&self) -> Pubkey {
        *self.key
    }
}

impl<'info> Deref for UnsafeAccount<'info> {
    type Target = AccountInfo<'info>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
