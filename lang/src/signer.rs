use crate::error::ErrorCode;
use crate::*;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::ops::Deref;

/// Type validating that the account signed the transaction. No other ownership
/// or type checks are done. If this is used, one should not try to access the
/// underlying account data.
#[derive(Debug, Clone)]
pub struct Signer<'info> {
    info: AccountInfo<'info>,
}

impl<'info> Signer<'info> {
    fn new(info: AccountInfo<'info>) -> Signer<'info> {
        Self { info }
    }

    /// Deserializes the given `info` into a `Signer`.
    #[inline(never)]
    pub fn try_from(info: &AccountInfo<'info>) -> Result<Signer<'info>, ProgramError> {
        if !info.is_signer {
            return Err(ErrorCode::AccountNotSigner.into());
        }
        Ok(Signer::new(info.clone()))
    }
}

impl<'info> AccountsExit<'info> for Signer<'info> {}

impl<'info> Deref for Signer<'info> {
    type Target = AccountInfo<'info>;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl_account_info_traits!(Signer<'info>);
impl_accounts_trait!(Signer<'info>);
impl_account_metas_trait!(Signer<'info>);
