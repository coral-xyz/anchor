use crate::error::ErrorCode;
use crate::*;
use solana_program::bpf_loader_upgradeable;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::{account_info::AccountInfo, bpf_loader_upgradeable::UpgradeableLoaderState};
use std::ops::Deref;

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct ProgramDataInner {
    slot: u64,
    upgrade_authority_address: Option<Pubkey>,
}

impl ProgramDataInner {
    pub fn slot(&self) -> &u64 {
        &self.slot
    }

    pub fn upgrade_authority_address(&self) -> &Option<Pubkey> {
        &self.upgrade_authority_address
    }
}

#[derive(Debug, Clone)]
pub struct ProgramData<'info> {
    program_data_inner: ProgramDataInner,
    info: AccountInfo<'info>,
}

impl<'info> ProgramData<'info> {
    fn new(info: AccountInfo<'info>, program_data_inner: ProgramDataInner) -> ProgramData<'info> {
        Self {
            info,
            program_data_inner,
        }
    }

    #[inline(never)]
    pub fn try_from(info: &AccountInfo<'info>) -> Result<ProgramData<'info>, ProgramError> {
        if *info.owner != bpf_loader_upgradeable::ID {
            return Err(ErrorCode::AccountNotUpgradableBPFOwned.into());
        }
        let program_state: bpf_loader_upgradeable::UpgradeableLoaderState =
            bincode::deserialize(&info.try_borrow_data()?)
                .map_err(|_| ProgramError::InvalidAccountData)?;
        match program_state {
            UpgradeableLoaderState::Uninitialized => {
                Err(anchor_lang::error::ErrorCode::AccountNotProgramData.into())
            }
            UpgradeableLoaderState::Buffer {
                authority_address: _,
            } => Err(anchor_lang::error::ErrorCode::AccountNotProgramData.into()),
            UpgradeableLoaderState::Program {
                programdata_address: _,
            } => Err(anchor_lang::error::ErrorCode::AccountNotProgramData.into()),
            UpgradeableLoaderState::ProgramData {
                slot,
                upgrade_authority_address,
            } => Ok(ProgramData::new(
                info.clone(),
                ProgramDataInner {
                    slot,
                    upgrade_authority_address,
                },
            )),
        }
    }
}

impl<'info> Accounts<'info> for ProgramData<'info> {
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
        ProgramData::try_from(account)
    }
}

impl<'info> AccountsExit<'info> for ProgramData<'info> {
    fn exit(&self, _program_id: &Pubkey) -> ProgramResult {
        // No-op.
        Ok(())
    }
}

impl<'info> ToAccountMetas for ProgramData<'info> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        let is_signer = is_signer.unwrap_or(self.info.is_signer);
        let meta = match self.info.is_writable {
            false => AccountMeta::new_readonly(*self.info.key, is_signer),
            true => AccountMeta::new(*self.info.key, is_signer),
        };
        vec![meta]
    }
}

impl<'info> ToAccountInfos<'info> for ProgramData<'info> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        vec![self.info.clone()]
    }
}

impl<'info> ToAccountInfo<'info> for ProgramData<'info> {
    fn to_account_info(&self) -> AccountInfo<'info> {
        self.info.clone()
    }
}

impl<'info> AsRef<AccountInfo<'info>> for ProgramData<'info> {
    fn as_ref(&self) -> &AccountInfo<'info> {
        &self.info
    }
}

impl<'info> AsRef<ProgramDataInner> for ProgramData<'info> {
    fn as_ref(&self) -> &ProgramDataInner {
        &self.program_data_inner
    }
}

impl<'info> Deref for ProgramData<'info> {
    type Target = ProgramDataInner;

    fn deref(&self) -> &Self::Target {
        &self.program_data_inner
    }
}

impl<'info> Key for ProgramData<'info> {
    fn key(&self) -> Pubkey {
        *self.info.key
    }
}
