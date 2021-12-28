use crate::error;

/// Error codes that can be returned by internal framework code.
#[error(offset = 0)]
pub enum ErrorCode {
    // Instructions
    /// 100 - 8 byte instruction identifier not provided
    #[msg("8 byte instruction identifier not provided")]
    InstructionMissing = 100,
    /// 101 - Fallback functions are not supported
    #[msg("Fallback functions are not supported")]
    InstructionFallbackNotFound,
    /// 102 - The program could not deserialize the given instruction
    #[msg("The program could not deserialize the given instruction")]
    InstructionDidNotDeserialize,
    /// 103 - The program could not serialize the given instruction
    #[msg("The program could not serialize the given instruction")]
    InstructionDidNotSerialize,

    // IDL instructions
    /// 1000 - The program was compiled without idl instructions
    #[msg("The program was compiled without idl instructions")]
    IdlInstructionStub = 1000,
    /// 1001 - Invalid program given to the IDL instruction
    #[msg("Invalid program given to the IDL instruction")]
    IdlInstructionInvalidProgram,

    // Constraints
    /// 2000 - A mut constraint was violated
    #[msg("A mut constraint was violated")]
    ConstraintMut = 2000,
    /// 2001
    #[msg("A has one constraint was violated")]
    ConstraintHasOne,
    /// 2002
    #[msg("A signer constraint was violated")]
    ConstraintSigner,
    /// 2003
    #[msg("A raw constraint was violated")]
    ConstraintRaw,
    /// 2004
    #[msg("An owner constraint was violated")]
    ConstraintOwner,
    /// 2005
    #[msg("A rent exemption constraint was violated")]
    ConstraintRentExempt,
    /// 2006
    #[msg("A seeds constraint was violated")]
    ConstraintSeeds,
    /// 2007
    #[msg("An executable constraint was violated")]
    ConstraintExecutable,
    /// 2008
    #[msg("A state constraint was violated")]
    ConstraintState,
    /// 2009
    #[msg("An associated constraint was violated")]
    ConstraintAssociated,
    /// 2010
    #[msg("An associated init constraint was violated")]
    ConstraintAssociatedInit,
    /// 2011
    #[msg("A close constraint was violated")]
    ConstraintClose,
    /// 2012
    #[msg("An address constraint was violated")]
    ConstraintAddress,
    /// 2013
    #[msg("Expected zero account discriminant")]
    ConstraintZero,
    /// 2014
    #[msg("A token mint constraint was violated")]
    ConstraintTokenMint,
    /// 2015
    #[msg("A token owner constraint was violated")]
    ConstraintTokenOwner,
    /// The mint mint is intentional -> a mint authority for the mint.
    ///
    /// 2016
    #[msg("A mint mint authority constraint was violated")]
    ConstraintMintMintAuthority,
    /// 2017
    #[msg("A mint freeze authority constraint was violated")]
    ConstraintMintFreezeAuthority,
    /// 2018
    #[msg("A mint decimals constraint was violated")]
    ConstraintMintDecimals,
    /// 2019
    #[msg("A space constraint was violated")]
    ConstraintSpace,

    // Accounts.
    #[msg("The account discriminator was already set on this account")]
    AccountDiscriminatorAlreadySet = 3000,
    #[msg("No 8 byte discriminator was found on the account")]
    AccountDiscriminatorNotFound,
    #[msg("8 byte discriminator did not match what was expected")]
    AccountDiscriminatorMismatch,
    #[msg("Failed to deserialize the account")]
    AccountDidNotDeserialize,
    #[msg("Failed to serialize the account")]
    AccountDidNotSerialize,
    #[msg("Not enough account keys given to the instruction")]
    AccountNotEnoughKeys,
    #[msg("The given account is not mutable")]
    AccountNotMutable,
    #[msg("The given account is owned by a different program than expected")]
    AccountOwnedByWrongProgram,
    #[msg("Program ID was not as expected")]
    InvalidProgramId,
    #[msg("Program account is not executable")]
    InvalidProgramExecutable,
    #[msg("The given account did not sign")]
    AccountNotSigner,
    #[msg("The given account is not owned by the system program")]
    AccountNotSystemOwned,
    #[msg("The program expected this account to be already initialized")]
    AccountNotInitialized,
    #[msg("The given account is not a program data account")]
    AccountNotProgramData,

    // State.
    #[msg("The given state account does not have the correct address")]
    StateInvalidAddress = 4000,

    // Used for APIs that shouldn't be used anymore.
    #[msg("The API being used is deprecated and should no longer be used")]
    Deprecated = 5000,
}
