use crate::{Accounts, AccountsExit, AccountsInit, ToAccountInfo, ToAccountInfos, ToAccountMetas};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

impl<'info> Accounts<'info> for AccountInfo<'info> {
    fn try_accounts(
        _program_id: &Pubkey,
        accounts: &mut &[AccountInfo<'info>],
    ) -> Result<Self, ProgramError> {
        if accounts.len() == 0 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        let account = &accounts[0];
        *accounts = &accounts[1..];
        Ok(account.clone())
    }
}

impl<'info> AccountsInit<'info> for AccountInfo<'info> {
    fn try_accounts_init(
        _program_id: &Pubkey,
        accounts: &mut &[AccountInfo<'info>],
    ) -> Result<Self, ProgramError> {
        if accounts.len() == 0 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let account = &accounts[0];
        *accounts = &accounts[1..];

        // The discriminator should be zero, since we're initializing.
        let data: &[u8] = &account.try_borrow_data()?;
        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);
        let discriminator = u64::from_le_bytes(disc_bytes);
        if discriminator != 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(account.clone())
    }
}

impl<'info> ToAccountMetas for AccountInfo<'info> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        let is_signer = is_signer.unwrap_or(self.is_signer);
        let meta = match self.is_writable {
            false => AccountMeta::new_readonly(*self.key, is_signer),
            true => AccountMeta::new(*self.key, is_signer),
        };
        vec![meta]
    }
}

impl<'info> ToAccountInfos<'info> for AccountInfo<'info> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        vec![self.clone()]
    }
}

impl<'info> ToAccountInfo<'info> for AccountInfo<'info> {
    fn to_account_info(&self) -> AccountInfo<'info> {
        self.clone()
    }
}

impl<'info> AccountsExit<'info> for AccountInfo<'info> {
    fn exit(&self, _program_id: &Pubkey) -> ProgramResult {
        // no-op
        Ok(())
    }
}
