use crate::error::ErrorCode;
use crate::*;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::system_program;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct SystemAccount<'info> {
    info: AccountInfo<'info>,
}

impl<'info> SystemAccount<'info> {
    fn new(info: AccountInfo<'info>) -> SystemAccount<'info> {
        Self { info }
    }

    #[inline(never)]
    pub fn try_from(info: &AccountInfo<'info>) -> Result<SystemAccount<'info>, ProgramError> {
        if *info.owner != system_program::ID {
            return Err(ErrorCode::AccountNotSystemOwned.into());
        }
        Ok(SystemAccount::new(info.clone()))
    }
}

impl<'info> AccountsExit<'info> for SystemAccount<'info> {}

impl<'info> ToAccountMetas for SystemAccount<'info> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        let is_signer = is_signer.unwrap_or(self.info.is_signer);
        let meta = match self.info.is_writable {
            false => AccountMeta::new_readonly(*self.info.key, is_signer),
            true => AccountMeta::new(*self.info.key, is_signer),
        };
        vec![meta]
    }
}

impl<'info> Deref for SystemAccount<'info> {
    type Target = AccountInfo<'info>;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl_account_info_traits!(SystemAccount<'info>);
impl_accounts_trait!(SystemAccount<'info>);
