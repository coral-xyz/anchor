use crate::{
    AccountDeserialize, Accounts, AccountsExit, ToAccountInfo, ToAccountInfos, ToAccountMetas,
};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::ops::{Deref, DerefMut};

/// Container for any account *not* owned by the current program.
#[derive(Clone)]
pub struct CpiAccount<'a, T: AccountDeserialize + Clone> {
    info: AccountInfo<'a>,
    account: Box<T>,
}

impl<'a, T: AccountDeserialize + Clone> CpiAccount<'a, T> {
    pub fn new(info: AccountInfo<'a>, account: Box<T>) -> CpiAccount<'a, T> {
        Self { info, account }
    }

    /// Deserializes the given `info` into a `CpiAccount`.
    pub fn try_from(info: &AccountInfo<'a>) -> Result<CpiAccount<'a, T>, ProgramError> {
        let mut data: &[u8] = &info.try_borrow_data()?;
        Ok(CpiAccount::new(
            info.clone(),
            Box::new(T::try_deserialize(&mut data)?),
        ))
    }

    /// Reloads the account from storage. This is useful, for example, when
    /// observing side effects after CPI.
    pub fn reload(&self) -> Result<CpiAccount<'a, T>, ProgramError> {
        let info = self.to_account_info();
        let mut data: &[u8] = &info.try_borrow_data()?;
        Ok(CpiAccount::new(
            info.clone(),
            Box::new(T::try_deserialize(&mut data)?),
        ))
    }
}

impl<'info, T> Accounts<'info> for CpiAccount<'info, T>
where
    T: AccountDeserialize + Clone,
{
    #[inline(never)]
    fn try_accounts(
        _program_id: &Pubkey,
        accounts: &mut &[AccountInfo<'info>],
    ) -> Result<Self, ProgramError> {
        if accounts.is_empty() {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        let account = &accounts[0];
        *accounts = &accounts[1..];
        // No owner check is done here.
        let pa = CpiAccount::try_from(account)?;
        Ok(pa)
    }
}

impl<'info, T: AccountDeserialize + Clone> ToAccountMetas for CpiAccount<'info, T> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        let is_signer = is_signer.unwrap_or(self.info.is_signer);
        let meta = match self.info.is_writable {
            false => AccountMeta::new_readonly(*self.info.key, is_signer),
            true => AccountMeta::new(*self.info.key, is_signer),
        };
        vec![meta]
    }
}

impl<'info, T: AccountDeserialize + Clone> ToAccountInfos<'info> for CpiAccount<'info, T> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        vec![self.info.clone()]
    }
}

impl<'info, T: AccountDeserialize + Clone> ToAccountInfo<'info> for CpiAccount<'info, T> {
    fn to_account_info(&self) -> AccountInfo<'info> {
        self.info.clone()
    }
}

impl<'a, T: AccountDeserialize + Clone> Deref for CpiAccount<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}

impl<'a, T: AccountDeserialize + Clone> DerefMut for CpiAccount<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account
    }
}

impl<'info, T: AccountDeserialize + Clone> AccountsExit<'info> for CpiAccount<'info, T> {
    fn exit(&self, _program_id: &Pubkey) -> ProgramResult {
        // no-op
        Ok(())
    }
}
