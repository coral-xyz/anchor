import typing
from anchorpy.error import ProgramError


class InstructionMissing(ProgramError):
    def __init__(self):
        super().__init__(100, "8 byte instruction identifier not provided")

    code = 100
    name = "InstructionMissing"
    msg = "8 byte instruction identifier not provided"


class InstructionFallbackNotFound(ProgramError):
    def __init__(self):
        super().__init__(101, "Fallback functions are not supported")

    code = 101
    name = "InstructionFallbackNotFound"
    msg = "Fallback functions are not supported"


class InstructionDidNotDeserialize(ProgramError):
    def __init__(self):
        super().__init__(102, "The program could not deserialize the given instruction")

    code = 102
    name = "InstructionDidNotDeserialize"
    msg = "The program could not deserialize the given instruction"


class InstructionDidNotSerialize(ProgramError):
    def __init__(self):
        super().__init__(103, "The program could not serialize the given instruction")

    code = 103
    name = "InstructionDidNotSerialize"
    msg = "The program could not serialize the given instruction"


class IdlInstructionStub(ProgramError):
    def __init__(self):
        super().__init__(1000, "The program was compiled without idl instructions")

    code = 1000
    name = "IdlInstructionStub"
    msg = "The program was compiled without idl instructions"


class IdlInstructionInvalidProgram(ProgramError):
    def __init__(self):
        super().__init__(
            1001, "The transaction was given an invalid program for the IDL instruction"
        )

    code = 1001
    name = "IdlInstructionInvalidProgram"
    msg = "The transaction was given an invalid program for the IDL instruction"


class ConstraintMut(ProgramError):
    def __init__(self):
        super().__init__(2000, "A mut constraint was violated")

    code = 2000
    name = "ConstraintMut"
    msg = "A mut constraint was violated"


class ConstraintHasOne(ProgramError):
    def __init__(self):
        super().__init__(2001, "A has_one constraint was violated")

    code = 2001
    name = "ConstraintHasOne"
    msg = "A has_one constraint was violated"


class ConstraintSigner(ProgramError):
    def __init__(self):
        super().__init__(2002, "A signer constraint was violated")

    code = 2002
    name = "ConstraintSigner"
    msg = "A signer constraint was violated"


class ConstraintRaw(ProgramError):
    def __init__(self):
        super().__init__(2003, "A raw constraint was violated")

    code = 2003
    name = "ConstraintRaw"
    msg = "A raw constraint was violated"


class ConstraintOwner(ProgramError):
    def __init__(self):
        super().__init__(2004, "An owner constraint was violated")

    code = 2004
    name = "ConstraintOwner"
    msg = "An owner constraint was violated"


class ConstraintRentExempt(ProgramError):
    def __init__(self):
        super().__init__(2005, "A rent exempt constraint was violated")

    code = 2005
    name = "ConstraintRentExempt"
    msg = "A rent exempt constraint was violated"


class ConstraintSeeds(ProgramError):
    def __init__(self):
        super().__init__(2006, "A seeds constraint was violated")

    code = 2006
    name = "ConstraintSeeds"
    msg = "A seeds constraint was violated"


class ConstraintExecutable(ProgramError):
    def __init__(self):
        super().__init__(2007, "An executable constraint was violated")

    code = 2007
    name = "ConstraintExecutable"
    msg = "An executable constraint was violated"


class ConstraintState(ProgramError):
    def __init__(self):
        super().__init__(2008, "A state constraint was violated")

    code = 2008
    name = "ConstraintState"
    msg = "A state constraint was violated"


class ConstraintAssociated(ProgramError):
    def __init__(self):
        super().__init__(2009, "An associated constraint was violated")

    code = 2009
    name = "ConstraintAssociated"
    msg = "An associated constraint was violated"


class ConstraintAssociatedInit(ProgramError):
    def __init__(self):
        super().__init__(2010, "An associated init constraint was violated")

    code = 2010
    name = "ConstraintAssociatedInit"
    msg = "An associated init constraint was violated"


class ConstraintClose(ProgramError):
    def __init__(self):
        super().__init__(2011, "A close constraint was violated")

    code = 2011
    name = "ConstraintClose"
    msg = "A close constraint was violated"


class ConstraintAddress(ProgramError):
    def __init__(self):
        super().__init__(2012, "An address constraint was violated")

    code = 2012
    name = "ConstraintAddress"
    msg = "An address constraint was violated"


class ConstraintZero(ProgramError):
    def __init__(self):
        super().__init__(2013, "Expected zero account discriminant")

    code = 2013
    name = "ConstraintZero"
    msg = "Expected zero account discriminant"


class ConstraintTokenMint(ProgramError):
    def __init__(self):
        super().__init__(2014, "A token mint constraint was violated")

    code = 2014
    name = "ConstraintTokenMint"
    msg = "A token mint constraint was violated"


class ConstraintTokenOwner(ProgramError):
    def __init__(self):
        super().__init__(2015, "A token owner constraint was violated")

    code = 2015
    name = "ConstraintTokenOwner"
    msg = "A token owner constraint was violated"


class ConstraintMintMintAuthority(ProgramError):
    def __init__(self):
        super().__init__(2016, "A mint mint authority constraint was violated")

    code = 2016
    name = "ConstraintMintMintAuthority"
    msg = "A mint mint authority constraint was violated"


class ConstraintMintFreezeAuthority(ProgramError):
    def __init__(self):
        super().__init__(2017, "A mint freeze authority constraint was violated")

    code = 2017
    name = "ConstraintMintFreezeAuthority"
    msg = "A mint freeze authority constraint was violated"


class ConstraintMintDecimals(ProgramError):
    def __init__(self):
        super().__init__(2018, "A mint decimals constraint was violated")

    code = 2018
    name = "ConstraintMintDecimals"
    msg = "A mint decimals constraint was violated"


class ConstraintSpace(ProgramError):
    def __init__(self):
        super().__init__(2019, "A space constraint was violated")

    code = 2019
    name = "ConstraintSpace"
    msg = "A space constraint was violated"


class RequireViolated(ProgramError):
    def __init__(self):
        super().__init__(2500, "A require expression was violated")

    code = 2500
    name = "RequireViolated"
    msg = "A require expression was violated"


class RequireEqViolated(ProgramError):
    def __init__(self):
        super().__init__(2501, "A require_eq expression was violated")

    code = 2501
    name = "RequireEqViolated"
    msg = "A require_eq expression was violated"


class RequireKeysEqViolated(ProgramError):
    def __init__(self):
        super().__init__(2502, "A require_keys_eq expression was violated")

    code = 2502
    name = "RequireKeysEqViolated"
    msg = "A require_keys_eq expression was violated"


class RequireNeqViolated(ProgramError):
    def __init__(self):
        super().__init__(2503, "A require_neq expression was violated")

    code = 2503
    name = "RequireNeqViolated"
    msg = "A require_neq expression was violated"


class RequireKeysNeqViolated(ProgramError):
    def __init__(self):
        super().__init__(2504, "A require_keys_neq expression was violated")

    code = 2504
    name = "RequireKeysNeqViolated"
    msg = "A require_keys_neq expression was violated"


class RequireGtViolated(ProgramError):
    def __init__(self):
        super().__init__(2505, "A require_gt expression was violated")

    code = 2505
    name = "RequireGtViolated"
    msg = "A require_gt expression was violated"


class RequireGteViolated(ProgramError):
    def __init__(self):
        super().__init__(2506, "A require_gte expression was violated")

    code = 2506
    name = "RequireGteViolated"
    msg = "A require_gte expression was violated"


class AccountDiscriminatorAlreadySet(ProgramError):
    def __init__(self):
        super().__init__(
            3000, "The account discriminator was already set on this account"
        )

    code = 3000
    name = "AccountDiscriminatorAlreadySet"
    msg = "The account discriminator was already set on this account"


class AccountDiscriminatorNotFound(ProgramError):
    def __init__(self):
        super().__init__(3001, "No 8 byte discriminator was found on the account")

    code = 3001
    name = "AccountDiscriminatorNotFound"
    msg = "No 8 byte discriminator was found on the account"


class AccountDiscriminatorMismatch(ProgramError):
    def __init__(self):
        super().__init__(3002, "8 byte discriminator did not match what was expected")

    code = 3002
    name = "AccountDiscriminatorMismatch"
    msg = "8 byte discriminator did not match what was expected"


class AccountDidNotDeserialize(ProgramError):
    def __init__(self):
        super().__init__(3003, "Failed to deserialize the account")

    code = 3003
    name = "AccountDidNotDeserialize"
    msg = "Failed to deserialize the account"


class AccountDidNotSerialize(ProgramError):
    def __init__(self):
        super().__init__(3004, "Failed to serialize the account")

    code = 3004
    name = "AccountDidNotSerialize"
    msg = "Failed to serialize the account"


class AccountNotEnoughKeys(ProgramError):
    def __init__(self):
        super().__init__(3005, "Not enough account keys given to the instruction")

    code = 3005
    name = "AccountNotEnoughKeys"
    msg = "Not enough account keys given to the instruction"


class AccountNotMutable(ProgramError):
    def __init__(self):
        super().__init__(3006, "The given account is not mutable")

    code = 3006
    name = "AccountNotMutable"
    msg = "The given account is not mutable"


class AccountOwnedByWrongProgram(ProgramError):
    def __init__(self):
        super().__init__(
            3007, "The given account is owned by a different program than expected"
        )

    code = 3007
    name = "AccountOwnedByWrongProgram"
    msg = "The given account is owned by a different program than expected"


class InvalidProgramId(ProgramError):
    def __init__(self):
        super().__init__(3008, "Program ID was not as expected")

    code = 3008
    name = "InvalidProgramId"
    msg = "Program ID was not as expected"


class InvalidProgramExecutable(ProgramError):
    def __init__(self):
        super().__init__(3009, "Program account is not executable")

    code = 3009
    name = "InvalidProgramExecutable"
    msg = "Program account is not executable"


class AccountNotSigner(ProgramError):
    def __init__(self):
        super().__init__(3010, "The given account did not sign")

    code = 3010
    name = "AccountNotSigner"
    msg = "The given account did not sign"


class AccountNotSystemOwned(ProgramError):
    def __init__(self):
        super().__init__(3011, "The given account is not owned by the system program")

    code = 3011
    name = "AccountNotSystemOwned"
    msg = "The given account is not owned by the system program"


class AccountNotInitialized(ProgramError):
    def __init__(self):
        super().__init__(
            3012, "The program expected this account to be already initialized"
        )

    code = 3012
    name = "AccountNotInitialized"
    msg = "The program expected this account to be already initialized"


class AccountNotProgramData(ProgramError):
    def __init__(self):
        super().__init__(3013, "The given account is not a program data account")

    code = 3013
    name = "AccountNotProgramData"
    msg = "The given account is not a program data account"


class AccountNotAssociatedTokenAccount(ProgramError):
    def __init__(self):
        super().__init__(3014, "The given account is not the associated token account")

    code = 3014
    name = "AccountNotAssociatedTokenAccount"
    msg = "The given account is not the associated token account"


class AccountSysvarMismatch(ProgramError):
    def __init__(self):
        super().__init__(
            3015, "The given public key does not match the required sysvar"
        )

    code = 3015
    name = "AccountSysvarMismatch"
    msg = "The given public key does not match the required sysvar"


class StateInvalidAddress(ProgramError):
    def __init__(self):
        super().__init__(
            4000, "The given state account does not have the correct address"
        )

    code = 4000
    name = "StateInvalidAddress"
    msg = "The given state account does not have the correct address"


class Deprecated(ProgramError):
    def __init__(self):
        super().__init__(
            5000, "The API being used is deprecated and should no longer be used"
        )

    code = 5000
    name = "Deprecated"
    msg = "The API being used is deprecated and should no longer be used"


AnchorError = typing.Union[
    InstructionMissing,
    InstructionFallbackNotFound,
    InstructionDidNotDeserialize,
    InstructionDidNotSerialize,
    IdlInstructionStub,
    IdlInstructionInvalidProgram,
    ConstraintMut,
    ConstraintHasOne,
    ConstraintSigner,
    ConstraintRaw,
    ConstraintOwner,
    ConstraintRentExempt,
    ConstraintSeeds,
    ConstraintExecutable,
    ConstraintState,
    ConstraintAssociated,
    ConstraintAssociatedInit,
    ConstraintClose,
    ConstraintAddress,
    ConstraintZero,
    ConstraintTokenMint,
    ConstraintTokenOwner,
    ConstraintMintMintAuthority,
    ConstraintMintFreezeAuthority,
    ConstraintMintDecimals,
    ConstraintSpace,
    RequireViolated,
    RequireEqViolated,
    RequireKeysEqViolated,
    RequireNeqViolated,
    RequireKeysNeqViolated,
    RequireGtViolated,
    RequireGteViolated,
    AccountDiscriminatorAlreadySet,
    AccountDiscriminatorNotFound,
    AccountDiscriminatorMismatch,
    AccountDidNotDeserialize,
    AccountDidNotSerialize,
    AccountNotEnoughKeys,
    AccountNotMutable,
    AccountOwnedByWrongProgram,
    InvalidProgramId,
    InvalidProgramExecutable,
    AccountNotSigner,
    AccountNotSystemOwned,
    AccountNotInitialized,
    AccountNotProgramData,
    AccountNotAssociatedTokenAccount,
    AccountSysvarMismatch,
    StateInvalidAddress,
    Deprecated,
]
ANCHOR_ERROR_MAP: dict[int, AnchorError] = {
    100: InstructionMissing(),
    101: InstructionFallbackNotFound(),
    102: InstructionDidNotDeserialize(),
    103: InstructionDidNotSerialize(),
    1000: IdlInstructionStub(),
    1001: IdlInstructionInvalidProgram(),
    2000: ConstraintMut(),
    2001: ConstraintHasOne(),
    2002: ConstraintSigner(),
    2003: ConstraintRaw(),
    2004: ConstraintOwner(),
    2005: ConstraintRentExempt(),
    2006: ConstraintSeeds(),
    2007: ConstraintExecutable(),
    2008: ConstraintState(),
    2009: ConstraintAssociated(),
    2010: ConstraintAssociatedInit(),
    2011: ConstraintClose(),
    2012: ConstraintAddress(),
    2013: ConstraintZero(),
    2014: ConstraintTokenMint(),
    2015: ConstraintTokenOwner(),
    2016: ConstraintMintMintAuthority(),
    2017: ConstraintMintFreezeAuthority(),
    2018: ConstraintMintDecimals(),
    2019: ConstraintSpace(),
    2500: RequireViolated(),
    2501: RequireEqViolated(),
    2502: RequireKeysEqViolated(),
    2503: RequireNeqViolated(),
    2504: RequireKeysNeqViolated(),
    2505: RequireGtViolated(),
    2506: RequireGteViolated(),
    3000: AccountDiscriminatorAlreadySet(),
    3001: AccountDiscriminatorNotFound(),
    3002: AccountDiscriminatorMismatch(),
    3003: AccountDidNotDeserialize(),
    3004: AccountDidNotSerialize(),
    3005: AccountNotEnoughKeys(),
    3006: AccountNotMutable(),
    3007: AccountOwnedByWrongProgram(),
    3008: InvalidProgramId(),
    3009: InvalidProgramExecutable(),
    3010: AccountNotSigner(),
    3011: AccountNotSystemOwned(),
    3012: AccountNotInitialized(),
    3013: AccountNotProgramData(),
    3014: AccountNotAssociatedTokenAccount(),
    3015: AccountSysvarMismatch(),
    4000: StateInvalidAddress(),
    5000: Deprecated(),
}


def from_code(code: int) -> typing.Optional[AnchorError]:
    maybe_err = ANCHOR_ERROR_MAP.get(code)
    if maybe_err is None:
        return None
    return maybe_err
