use crate::{Accounts, AccountsExit, ToAccountInfos, ToAccountMetas};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::ops::Deref;

impl<'info, T: Accounts<'info>> Accounts<'info> for Box<T> {
    fn try_accounts(
        program_id: &Pubkey,
        accounts: &mut &[AccountInfo<'info>],
    ) -> Result<Self, ProgramError> {
        T::try_accounts(program_id, accounts).map(Box::new)
    }
}

impl<'info, T: AccountsExit<'info>> AccountsExit<'info> for Box<T> {
    fn exit(&self, program_id: &Pubkey) -> ProgramResult {
        T::exit(Deref::deref(self), program_id)
    }
}

impl<'info, T: ToAccountInfos<'info>> ToAccountInfos<'info> for Box<T> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        T::to_account_infos(self)
    }
}

impl<T: ToAccountMetas> ToAccountMetas for Box<T> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        T::to_account_metas(self, is_signer)
    }
}
