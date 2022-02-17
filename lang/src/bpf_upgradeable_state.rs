use crate::error::ErrorCode;
use crate::{AccountDeserialize, AccountSerialize, AnchorResult, Owner};
use solana_program::{
    bpf_loader_upgradeable::UpgradeableLoaderState, program_error::ProgramError, pubkey::Pubkey,
};

#[derive(Clone)]
pub struct ProgramData {
    pub slot: u64,
    pub upgrade_authority_address: Option<Pubkey>,
}

impl AccountDeserialize for ProgramData {
    fn try_deserialize(buf: &mut &[u8]) -> AnchorResult<Self> {
        ProgramData::try_deserialize_unchecked(buf)
    }

    fn try_deserialize_unchecked(buf: &mut &[u8]) -> AnchorResult<Self> {
        let program_state = AccountDeserialize::try_deserialize_unchecked(buf)?;

        match program_state {
            UpgradeableLoaderState::Uninitialized => {
                anchor_attribute_error::error!(ErrorCode::AccountNotProgramData)
            }
            UpgradeableLoaderState::Buffer {
                authority_address: _,
            } => anchor_attribute_error::error!(ErrorCode::AccountNotProgramData),
            UpgradeableLoaderState::Program {
                programdata_address: _,
            } => anchor_attribute_error::error!(ErrorCode::AccountNotProgramData),
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

impl AccountSerialize for ProgramData {
    fn try_serialize<W: std::io::Write>(&self, _writer: &mut W) -> AnchorResult<()> {
        // no-op
        Ok(())
    }
}

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

impl AccountSerialize for UpgradeableLoaderState {
    fn try_serialize<W: std::io::Write>(&self, _writer: &mut W) -> AnchorResult<()> {
        // no-op
        Ok(())
    }
}

impl AccountDeserialize for UpgradeableLoaderState {
    fn try_deserialize(buf: &mut &[u8]) -> AnchorResult<Self> {
        UpgradeableLoaderState::try_deserialize_unchecked(buf)
    }

    fn try_deserialize_unchecked(buf: &mut &[u8]) -> AnchorResult<Self> {
        bincode::deserialize(buf).map_err(|_| ProgramError::InvalidAccountData.into())
    }
}
