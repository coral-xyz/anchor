use crate::error::ErrorCode;
use crate::{
    Accounts, AccountsClose, AccountsExit, Key, Owner, ToAccountInfo, ToAccountInfos,
    ToAccountMetas, ZeroCopy,
};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::fmt;
use std::io::Write;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Account AccountLoader facilitating on demand zero copy deserialization.
/// Note that using accounts in this way is distinctly different from using,
/// for example, the [`ProgramAccount`](./struct.ProgramAccount.html). Namely,
/// one must call `load`, `load_mut`, or `load_init`, before reading or writing
/// to the account. For more details on zero-copy-deserialization, see the
/// [`account`](./attr.account.html) attribute.
///
/// When using it's important to be mindful of any calls to `load` so as not to
/// induce a `RefCell` panic, especially when sharing accounts across CPI
/// boundaries. When in doubt, one should make sure all refs resulting from a
/// call to `load` are dropped before CPI.
#[derive(Clone)]
pub struct AccountLoader<'info, T: ZeroCopy + Owner> {
    acc_info: AccountInfo<'info>,
    phantom: PhantomData<&'info T>,
    data: *mut T,
}

impl<'info, T: ZeroCopy + Owner + fmt::Debug> fmt::Debug for AccountLoader<'info, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountLoader")
            .field("acc_info", &self.acc_info)
            .field("phantom", &self.phantom)
            .field("data", unsafe { &*self.data })
            .finish()
    }
}

impl<'info, T: ZeroCopy + Owner> AccountLoader<'info, T> {
    fn new(acc_info: AccountInfo<'info>) -> Result<AccountLoader<'info, T>, ProgramError> {
        let data = bytemuck::from_bytes::<T>(&mut (acc_info.try_borrow_mut_data())?[8..])
            as *const T as *mut T;

        Ok(Self {
            acc_info,
            phantom: PhantomData,
            data,
        })
    }

    /// Constructs a new `Loader` from a previously initialized account.
    #[inline(never)]
    pub fn try_from(
        acc_info: &AccountInfo<'info>,
    ) -> Result<AccountLoader<'info, T>, ProgramError> {
        {
            if acc_info.owner != &T::owner() {
                return Err(ErrorCode::AccountNotProgramOwned.into());
            }
            let data: &[u8] = &acc_info.try_borrow_data()?;
            // Discriminator must match.
            let mut disc_bytes = [0u8; 8];
            disc_bytes.copy_from_slice(&data[..8]);
            if disc_bytes != T::discriminator() {
                return Err(ErrorCode::AccountDiscriminatorMismatch.into());
            }
        }

        Ok(AccountLoader::new(acc_info.clone())?)
    }

    /// Constructs a new `Loader` from an uninitialized account.
    #[inline(never)]
    pub fn try_from_unchecked(
        _program_id: &Pubkey,
        acc_info: &AccountInfo<'info>,
    ) -> Result<AccountLoader<'info, T>, ProgramError> {
        if acc_info.owner != &T::owner() {
            return Err(ErrorCode::AccountNotProgramOwned.into());
        }
        Ok(AccountLoader::new(acc_info.clone())?)
    }
}

impl<'info, T: ZeroCopy + Owner> Accounts<'info> for AccountLoader<'info, T> {
    #[inline(never)]
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
        let l = AccountLoader::try_from(account)?;
        Ok(l)
    }
}

impl<'info, T: ZeroCopy + Owner> AccountsExit<'info> for AccountLoader<'info, T> {
    // The account *cannot* be loaded when this is called.
    fn exit(&self, _program_id: &Pubkey) -> ProgramResult {
        let mut data = self.acc_info.try_borrow_mut_data()?;
        let dst: &mut [u8] = &mut data;
        let mut cursor = std::io::Cursor::new(dst);
        cursor.write_all(&T::discriminator()).unwrap();
        Ok(())
    }
}

impl<'info, T: ZeroCopy + Owner> AccountsClose<'info> for AccountLoader<'info, T> {
    fn close(&self, sol_destination: AccountInfo<'info>) -> ProgramResult {
        crate::common::close(self.to_account_info(), sol_destination)
    }
}

impl<'info, T: ZeroCopy + Owner> ToAccountMetas for AccountLoader<'info, T> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        let is_signer = is_signer.unwrap_or(self.acc_info.is_signer);
        let meta = match self.acc_info.is_writable {
            false => AccountMeta::new_readonly(*self.acc_info.key, is_signer),
            true => AccountMeta::new(*self.acc_info.key, is_signer),
        };
        vec![meta]
    }
}

impl<'info, T: ZeroCopy + Owner> AsRef<AccountInfo<'info>> for AccountLoader<'info, T> {
    fn as_ref(&self) -> &AccountInfo<'info> {
        &self.acc_info
    }
}

impl<'info, T: ZeroCopy + Owner> ToAccountInfos<'info> for AccountLoader<'info, T> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        vec![self.acc_info.clone()]
    }
}

impl<'info, T: ZeroCopy + Owner> ToAccountInfo<'info> for AccountLoader<'info, T> {
    fn to_account_info(&self) -> AccountInfo<'info> {
        self.acc_info.clone()
    }
}

impl<'info, T: ZeroCopy + Owner> Key for AccountLoader<'info, T> {
    fn key(&self) -> Pubkey {
        *self.acc_info.key
    }
}

impl<'info, T> Deref for AccountLoader<'info, T>
where
    T: ZeroCopy + Owner,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.data) }
    }
}

impl<'info, T> DerefMut for AccountLoader<'info, T>
where
    T: ZeroCopy + Owner,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (*self.data) }
    }
}
