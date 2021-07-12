use crate::{Accounts, ToAccountInfos, ToAccountMetas};
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

impl<'info, T: ToAccountInfos<'info>> ToAccountInfos<'info> for Vec<T> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        self.iter()
            .flat_map(|item| item.to_account_infos())
            .collect()
    }
}

impl<T: ToAccountMetas> ToAccountMetas for Vec<T> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        self.iter()
            .flat_map(|item| (*item).to_account_metas(is_signer))
            .collect()
    }
}

impl<'info, T: Accounts<'info>> Accounts<'info> for Vec<T> {
    fn try_accounts(
        program_id: &Pubkey,
        accounts: &mut &[AccountInfo<'info>],
        ix_data: &[u8],
    ) -> Result<Self, ProgramError> {
        let mut vec: Vec<T> = Vec::new();
        T::try_accounts(program_id, accounts, ix_data).map(|item| vec.push(item))?;
        Ok(vec)
    }
}

#[cfg(test)]
mod tests {
    use crate::ToAccountInfo;
    use solana_program::clock::Epoch;
    use solana_program::pubkey::Pubkey;

    use super::*;

    #[derive(Accounts)]
    pub struct Test<'info> {
        #[account(signer)]
        test: AccountInfo<'info>,
    }

    #[test]
    fn test_accounts_trait_for_vec() {
        let program_id = Pubkey::default();

        let key = Pubkey::default();
        let mut lamports1 = 0;
        let mut data1 = vec![0; 10];
        let owner = Pubkey::default();
        let account1 = AccountInfo::new(
            &key,
            true,
            true,
            &mut lamports1,
            &mut data1,
            &owner,
            false,
            Epoch::default(),
        );

        let mut lamports2 = 0;
        let mut data2 = vec![0; 10];
        let account2 = AccountInfo::new(
            &key,
            true,
            true,
            &mut lamports2,
            &mut data2,
            &owner,
            false,
            Epoch::default(),
        );

        let mut accounts = &[account1, account2][..];
        let parsed_accounts = Vec::<Test>::try_accounts(&program_id, &mut accounts, &[]).unwrap();

        assert_eq!(accounts.len(), parsed_accounts.len());
    }

    #[test]
    #[should_panic]
    fn test_accounts_trait_for_vec_empty() {
        let program_id = Pubkey::default();

        let mut accounts = &[][..];
        Vec::<Test>::try_accounts(&program_id, &mut accounts, &[]).unwrap();
    }
}
