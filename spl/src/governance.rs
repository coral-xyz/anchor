use anchor_lang::prelude::*;
use spl_governance::addins::voter_weight::{
    VoterWeightAccountType, VoterWeightRecord as SplVoterWeightRecord,
};
use std::ops::{Deref, DerefMut};

/// Anchor wrapper for the SPL governance program's VoterWeightRecord type.
#[derive(Clone)]
pub struct VoterWeightRecord(SplVoterWeightRecord);

impl anchor_lang::AccountDeserialize for VoterWeightRecord {
    fn try_deserialize(buf: &mut &[u8]) -> std::result::Result<Self, ProgramError> {
        let mut data = buf;
        let vwr: SplVoterWeightRecord = anchor_lang::AnchorDeserialize::deserialize(&mut data)
            .map_err(|_| anchor_lang::__private::ErrorCode::AccountDidNotDeserialize)?;
        if vwr.account_type != VoterWeightAccountType::VoterWeightRecord {
            return Err(anchor_lang::__private::ErrorCode::AccountDidNotSerialize.into());
        }
        Ok(VoterWeightRecord(vwr))
    }

    fn try_deserialize_unchecked(buf: &mut &[u8]) -> std::result::Result<Self, ProgramError> {
        let mut data = buf;
        let vwr: SplVoterWeightRecord = anchor_lang::AnchorDeserialize::deserialize(&mut data)
            .map_err(|_| anchor_lang::__private::ErrorCode::AccountDidNotDeserialize)?;
        if vwr.account_type != VoterWeightAccountType::Uninitialized {
            return Err(anchor_lang::__private::ErrorCode::AccountDidNotSerialize.into());
        }
        Ok(VoterWeightRecord(vwr))
    }
}

impl anchor_lang::AccountSerialize for VoterWeightRecord {
    fn try_serialize<W: std::io::Write>(
        &self,
        writer: &mut W,
    ) -> std::result::Result<(), ProgramError> {
        AnchorSerialize::serialize(&self.0, writer)
            .map_err(|_| anchor_lang::__private::ErrorCode::AccountDidNotSerialize)?;
        Ok(())
    }
}

impl anchor_lang::Owner for VoterWeightRecord {
    fn owner() -> Pubkey {
        crate::ID
    }
}

impl Deref for VoterWeightRecord {
    type Target = SplVoterWeightRecord;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VoterWeightRecord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
