use crate::{
		AccountDeserializeZeroCopy,
    AccountDeserialize, AccountSerialize, Accounts, AccountsExit, AccountsInit, CpiAccount,
    ToAccountInfo, ToAccountInfos, ToAccountMetas,
};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::ops::{Deref, DerefMut};
use std::cell::RefMut
use crate::__private::bytemuck::{Zeroable, Pod};
use crate::__private::safe_transmute::trivial::TriviallyTransmutable;


pub trait ZeroCopy: AccountDeserializeZeroCopy + Copy + Clone + TriviallyTransmutable + Zeroable + Pod;

#[derive(Clone)]
pub struct ProgramAccountZeroCopy<'info, T: ZeroCopy> {
    info: AccountInfo<'info>,
		// RefMut is used instead of &'mut to avoid a refcell panic if one call
		// back into the program to mutate the account via CPI.
    account: RefMut<'info, T>,
}

impl<'a, T: ZeroCopy> ProgramAccountZeroCopy<'a, T> {
    pub fn new(info: AccountInfo<'a>, account: RefMut<'a, T>) -> ProgramAccountZeroCopy<'a, T> {
        Self {
            info,
						account,
        }
    }

    #[inline(never)]
    pub fn try_from(info: &AccountInfo<'a>) -> Result<ProgramAccountZeroCopy<'a, T>, ProgramError> {
        let mut data: &[u8] = &info.try_borrow_data()?;
        Ok(ProgramAccountZeroCopy::new(
            info.clone(),
            T::try_deserialize(&mut data)?,
        ))
    }

    #[inline(never)]
    pub fn try_from_init(info: &AccountInfo<'a>) -> Result<ProgramAccountZeroCopy<'a, T>, ProgramError> {
        let mut data: &[u8] = &info.try_borrow_data()?;

        // The discriminator should be zero, since we're initializing.
        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);
        let discriminator = u64::from_le_bytes(disc_bytes);
        if discriminator != 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(ProgramAccountZeroCopy::new(
            info.clone(),
            T::try_deserialize_unchecked(&mut data)?,
        ))
    }
}

impl<'info, T> Accounts<'info> for ProgramAccountZeroCopy<'info, T>
where
    T: AccountSerialize + AccountDeserialize + Clone,
{
    #[inline(never)]
    fn try_accounts(
        program_id: &Pubkey,
        accounts: &mut &[AccountInfo<'info>],
    ) -> Result<Self, ProgramError> {
        if accounts.is_empty() {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        let account = &accounts[0];
        *accounts = &accounts[1..];
        let pa = ProgramAccountZeroCopy::try_from(account)?;
        if pa.info.owner != program_id {
            return Err(ProgramError::Custom(1)); // todo: proper error
        }
        Ok(pa)
    }
}

impl<'info, T> AccountsInit<'info> for ProgramAccountZeroCopy<'info, T>
where
    T: AccountSerialize + AccountDeserialize + Clone,
{
    #[inline(never)]
    fn try_accounts_init(
        _program_id: &Pubkey,
        accounts: &mut &[AccountInfo<'info>],
    ) -> Result<Self, ProgramError> {
        if accounts.is_empty() {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        let account = &accounts[0];
        *accounts = &accounts[1..];
        ProgramAccountZeroCopy::try_from_init(account)
    }
}

impl<'info, T: AccountSerialize + AccountDeserialize + Clone> AccountsExit<'info>
    for ProgramAccountZeroCopy<'info, T>
{
    fn exit(&self, _program_id: &Pubkey) -> ProgramResult {
        let info = self.to_account_info();
        let mut data = info.try_borrow_mut_data()?;
        let dst: &mut [u8] = &mut data;
        let mut cursor = std::io::Cursor::new(dst);
        self.account.try_serialize(&mut cursor)?;
        Ok(())
    }
}

impl<'info, T: AccountSerialize + AccountDeserialize + Clone> ToAccountMetas
    for ProgramAccountZeroCopy<'info, T>
{
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        let is_signer = is_signer.unwrap_or(self.info.is_signer);
        let meta = match self.info.is_writable {
            false => AccountMeta::new_readonly(*self.info.key, is_signer),
            true => AccountMeta::new(*self.info.key, is_signer),
        };
        vec![meta]
    }
}

impl<'info, T: AccountSerialize + AccountDeserialize + Clone> ToAccountInfos<'info>
    for ProgramAccountZeroCopy<'info, T>
{
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        vec![self.info.clone()]
    }
}

impl<'info, T: AccountSerialize + AccountDeserialize + Clone> ToAccountInfo<'info>
    for ProgramAccountZeroCopy<'info, T>
{
    fn to_account_info(&self) -> AccountInfo<'info> {
        self.info.clone()
    }
}

impl<'a, T: AccountSerialize + AccountDeserialize + Clone> Deref for ProgramAccountZeroCopy<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &(*self).account
    }
}

impl<'a, T: AccountSerialize + AccountDeserialize + Clone> DerefMut for ProgramAccountZeroCopy<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut DerefMut::deref_mut(&mut self).account
    }
}
