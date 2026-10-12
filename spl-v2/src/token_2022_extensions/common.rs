use {
    anchor_lang::FixedVec,
    pinocchio::address::Address,
    solana_program_error::ProgramError,
    solana_pubkey::Pubkey,
};

#[cfg(any(feature = "guardrails", test))]
use anchor_lang::{programs::Token2022, require_eq, Id};

#[cfg(feature = "guardrails")]
#[inline]
pub(crate) fn validate_token_2022_program(program: &Address) -> Result<(), ProgramError> {
    require_eq!(*program, Token2022::id(), ProgramError::IncorrectProgramId);
    Ok(())
}

#[cfg(not(feature = "guardrails"))]
#[inline]
pub(crate) fn validate_token_2022_program(_program: &Address) -> Result<(), ProgramError> {
    Ok(())
}

pub(crate) fn pubkey_refs(
    pubkeys: &[Pubkey],
) -> FixedVec<&Pubkey, { anchor_lang::MAX_ACCOUNT_METAS }> {
    let mut refs = FixedVec::new();
    for pubkey in pubkeys {
        refs.push(pubkey);
    }
    refs
}
