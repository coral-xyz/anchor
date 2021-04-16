use crate::{
    AccountDeserializeZeroCopy, Accounts, AccountsExit, AccountsInit, ToAccountInfo,
    ToAccountInfos, ToAccountMetas, ZeroCopy,
};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::cell::{Ref, RefMut};
use std::io::Write;
use std::marker::PhantomData;
use std::ops::DerefMut;

// todo: rename AccountLoader?
pub struct ProgramAccountZeroCopy<'info, T: ZeroCopy> {
    acc_info: AccountInfo<'info>,
    phantom: PhantomData<&'info T>,
}

impl<'info, T: ZeroCopy> ProgramAccountZeroCopy<'info, T> {
    pub fn new(acc_info: AccountInfo<'info>) -> ProgramAccountZeroCopy<'info, T> {
        Self {
            acc_info,
            phantom: PhantomData,
        }
    }

    #[inline(never)]
    pub fn try_from(
        acc_info: &AccountInfo<'info>,
    ) -> Result<ProgramAccountZeroCopy<'info, T>, ProgramError> {
        let data: &[u8] = &acc_info.try_borrow_data()?;

        // Discriminator must match.
        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);
        if disc_bytes != T::discriminator() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(ProgramAccountZeroCopy::new(acc_info.clone()))
    }

    #[inline(never)]
    pub fn try_from_init(
        acc_info: &AccountInfo<'info>,
    ) -> Result<ProgramAccountZeroCopy<'info, T>, ProgramError> {
        let data = acc_info.try_borrow_mut_data()?;

        // The discriminator should be zero, since we're initializing.
        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);
        let discriminator = u64::from_le_bytes(disc_bytes);
        if discriminator != 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(ProgramAccountZeroCopy::new(acc_info.clone()))
    }

    pub fn load(&self) -> Result<Ref<T>, ProgramError> {
        Ok(Ref::map(self.acc_info.try_borrow_data()?, |data| {
            anchor_lang::__private::bytemuck::from_bytes(&data[8..])
        }))
    }

    pub fn load_mut(&self) -> Result<RefMut<T>, ProgramError> {
        // AcocuntInfo api allows you to borrow mut even if the account isn't
        // writable, so add this check for a better dev experience.
        if !self.acc_info.is_writable {
            return Err(ProgramError::Custom(87)); // todo: proper error
        }
        Ok(RefMut::map(self.acc_info.try_borrow_mut_data()?, |data| {
            AccountDeserializeZeroCopy::try_deserialize(data.deref_mut()).unwrap()
        }))
    }

    pub fn load_init(&self) -> Result<RefMut<T>, ProgramError> {
        let data = self.acc_info.try_borrow_mut_data()?;

        // The discriminator should be zero, since we're initializing.
        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);
        let discriminator = u64::from_le_bytes(disc_bytes);
        if discriminator != 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(RefMut::map(data, |data| {
            // Zero copy deserialize.
            let account =
                AccountDeserializeZeroCopy::try_deserialize_unchecked(data.deref_mut()).unwrap();

            account
        }))
    }
}

impl<'info, T: ZeroCopy> Accounts<'info> for ProgramAccountZeroCopy<'info, T> {
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
        if pa.acc_info.owner != program_id {
            return Err(ProgramError::Custom(1)); // todo: proper error
        }
        Ok(pa)
    }
}

impl<'info, T: ZeroCopy> AccountsInit<'info> for ProgramAccountZeroCopy<'info, T> {
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

impl<'info, T: ZeroCopy> AccountsExit<'info> for ProgramAccountZeroCopy<'info, T> {
    // The account *cannot* be loaded when this is called.
    fn exit(&self, _program_id: &Pubkey) -> ProgramResult {
        let mut data = self.acc_info.try_borrow_mut_data()?;
        let dst: &mut [u8] = &mut data;
        let mut cursor = std::io::Cursor::new(dst);
        cursor.write_all(&T::discriminator()).unwrap();
        Ok(())
    }
}

impl<'info, T: ZeroCopy> ToAccountMetas for ProgramAccountZeroCopy<'info, T> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        let is_signer = is_signer.unwrap_or(self.acc_info.is_signer);
        let meta = match self.acc_info.is_writable {
            false => AccountMeta::new_readonly(*self.acc_info.key, is_signer),
            true => AccountMeta::new(*self.acc_info.key, is_signer),
        };
        vec![meta]
    }
}

impl<'info, T: ZeroCopy> ToAccountInfos<'info> for ProgramAccountZeroCopy<'info, T> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        vec![self.acc_info.clone()]
    }
}

impl<'info, T: ZeroCopy> ToAccountInfo<'info> for ProgramAccountZeroCopy<'info, T> {
    fn to_account_info(&self) -> AccountInfo<'info> {
        self.acc_info.clone()
    }
}
