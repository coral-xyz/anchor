use crate::error::ErrorCode;
use crate::{AccountDeserialize, AccountDeserializeWithHeader, AccountSerialize, Owner, Result, AccountSerializeWithHeader};
use solana_program::{
    bpf_loader_upgradeable::UpgradeableLoaderState, program_error::ProgramError, pubkey::Pubkey,
};

#[derive(Clone)]
pub struct ProgramData {
    pub slot: u64,
    pub upgrade_authority_address: Option<Pubkey>,
}

impl AccountDeserializeWithHeader for ProgramData {}

impl AccountDeserialize for ProgramData {
    fn try_deserialize(buf: &mut &[u8]) -> Result<Self> {
        let program_state = AccountDeserializeWithHeader::try_deserialize_unchecked(buf)?;

        match program_state {
            UpgradeableLoaderState::Uninitialized => Err(ErrorCode::AccountNotProgramData.into()),
            UpgradeableLoaderState::Buffer {
                authority_address: _,
            } => Err(ErrorCode::AccountNotProgramData.into()),
            UpgradeableLoaderState::Program {
                programdata_address: _,
            } => Err(ErrorCode::AccountNotProgramData.into()),
            UpgradeableLoaderState::ProgramData {
                slot,
                upgrade_authority_address,
            } => Ok(ProgramData {
                slot,
                upgrade_authority_address,
            }),
        }
    }
}

impl AccountSerializeWithHeader for ProgramData {}
impl AccountSerialize for ProgramData {}

impl Owner for ProgramData {
    fn owner() -> solana_program::pubkey::Pubkey {
        anchor_lang::solana_program::bpf_loader_upgradeable::ID
    }
}

impl Owner for UpgradeableLoaderState {
    fn owner() -> Pubkey {
        anchor_lang::solana_program::bpf_loader_upgradeable::ID
    }
}

impl AccountSerializeWithHeader for UpgradeableLoaderState {}
impl AccountSerialize for UpgradeableLoaderState {}

impl AccountDeserializeWithHeader for UpgradeableLoaderState {}
impl AccountDeserialize for UpgradeableLoaderState {
    fn try_deserialize(buf: &mut &[u8]) -> Result<Self> {
        bincode::deserialize(buf).map_err(|_| ProgramError::InvalidAccountData.into())
    }
}
