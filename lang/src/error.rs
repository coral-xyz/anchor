use crate::error;

// Error codes that can be returned by internal framework code.
#[error(offset = 0)]
pub enum ErrorCode {
    // Instructions.
    #[msg("8 byte instruction identifier not provided")]
    InstructionMissing = 100,
    #[msg("Fallback functions are not supported")]
    InstructionFallbackNotFound,
    #[msg("The program could not deserialize the given instruction")]
    InstructionDidNotDeserialize,
    #[msg("The program could not serialize the given instruction")]
    InstructionDidNotSerialize,

    // IDL instructions.
    #[msg("The program was compiled without idl instructions")]
    IdlInstructionStub = 120,
    #[msg("Invalid program given to the IDL instruction")]
    IdlInstructionInvalidProgram,

    // Constraints.
    #[msg("A mut constraint was violated")]
    ConstraintMut = 140,
    #[msg("A has one constraint was violated")]
    ConstraintHasOne,
    #[msg("A signer constraint as violated")]
    ConstraintSigner,
    #[msg("A raw constraint was violated")]
    ConstraintRaw,
    #[msg("An owner constraint was violated")]
    ConstraintOwner,
    #[msg("A rent exemption constraint was violated")]
    ConstraintRentExempt,
    #[msg("A seeds constraint was violated")]
    ConstraintSeeds,
    #[msg("An executable constraint was violated")]
    ConstraintExecutable,
    #[msg("A state constraint was violated")]
    ConstraintState,
    #[msg("An associated constraint was violated")]
    ConstraintAssociated,
    #[msg("An associated init constraint was violated")]
    ConstraintAssociatedInit,
    #[msg("A close constraint was violated")]
    ConstraintClose,
    #[msg("An address constraint was violated")]
    ConstraintAddress,

    // Accounts.
    #[msg("The account discriminator was already set on this account")]
    AccountDiscriminatorAlreadySet = 160,
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
    #[msg("The given account is not owned by the executing program")]
    AccountNotProgramOwned,

    // State.
    #[msg("The given state account does not have the correct address")]
    StateInvalidAddress = 180,

    // Used for APIs that shouldn't be used anymore.
    #[msg("The API being used is deprecated and should no longer be used")]
    Deprecated = 299,
}
