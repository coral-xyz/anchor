import { IdlErrorMetadata } from "./program/namespace/types";

export class IdlError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "IdlError";
  }
}

// An error from a user defined program.
export class ProgramError extends Error {
  constructor(readonly code: number, readonly name: string, readonly msg: string, ...params: any[]) {
    super(...params);
  }

  public static parse(
    err: any,
    idlErrors: Map<number, IdlErrorMetadata>
  ): ProgramError | null {
    // TODO: don't rely on the error string. web3.js should preserve the error
    //       code information instead of giving us an untyped string.
    let components = err.toString().split("custom program error: ");
    if (components.length !== 2) {
      return null;
    }

    let errorCode: number;
    try {
      errorCode = parseInt(components[1]);
    } catch (parseErr) {
      return null;
    }

    // Parse user error.
    let metadata = idlErrors.get(errorCode);
    if (metadata !== undefined) {
      let errorMsg = metadata.msg ?? metadata.name
      return new ProgramError(errorCode, metadata.name, errorMsg, errorCode + ": " + errorMsg);
    }

    // Parse framework internal error.
    metadata = LangErrorMessage.get(errorCode);
    if (metadata !== undefined) {
      let errorMsg = metadata.msg ?? metadata.name
      return new ProgramError(errorCode, metadata.name, errorMsg, errorCode + ": " + errorMsg);
    }

    // Unable to parse the error. Just return the untranslated error.
    return null;
  }

  public toString(): string {
    return this.msg;
  }
}

const LangErrorCode = {
  // Instructions.
  InstructionMissing: 100,
  InstructionFallbackNotFound: 101,
  InstructionDidNotDeserialize: 102,
  InstructionDidNotSerialize: 103,

  // IDL instructions.
  IdlInstructionStub: 120,
  IdlInstructionInvalidProgram: 121,

  // Constraints.
  ConstraintMut: 140,
  ConstraintHasOne: 141,
  ConstraintSigner: 142,
  ConstraintRaw: 143,
  ConstraintOwner: 144,
  ConstraintRentExempt: 145,
  ConstraintSeeds: 146,
  ConstraintExecutable: 147,
  ConstraintState: 148,
  ConstraintAssociated: 149,
  ConstraintAssociatedInit: 150,
  ConstraintClose: 151,
  ConstraintAddress: 152,

  // Accounts.
  AccountDiscriminatorAlreadySet: 160,
  AccountDiscriminatorNotFound: 161,
  AccountDiscriminatorMismatch: 162,
  AccountDidNotDeserialize: 163,
  AccountDidNotSerialize: 164,
  AccountNotEnoughKeys: 165,
  AccountNotMutable: 166,
  AccountNotProgramOwned: 167,
  InvalidProgramId: 168,
  InvalidProgramIdExecutable: 169,

  // State.
  StateInvalidAddress: 180,

  // Used for APIs that shouldn't be used anymore.
  Deprecated: 299,
};

const LangErrorMessage = new Map<number, IdlErrorMetadata>([
  // Instructions.
  [
    LangErrorCode.InstructionMissing,
    { name: "InstructionMissing", msg: "8 byte instruction identifier not provided" },
  ],
  [
    LangErrorCode.InstructionFallbackNotFound,
    { name: "InstructionFallbackNotFound", msg: "Fallback functions are not supported" },
  ],
  [
    LangErrorCode.InstructionDidNotDeserialize,
    { name: "InstructionDidNotDeserialize", msg: "The program could not deserialize the given instruction" },
  ],
  [
    LangErrorCode.InstructionDidNotSerialize,
    { name: "InstructionDidNotSerialize", msg: "The program could not serialize the given instruction" },
  ],

  // Idl instructions.
  [
    LangErrorCode.IdlInstructionStub,
    { name: "IdlInstructionStub", msg: "The program was compiled without idl instructions" },
  ],
  [
    LangErrorCode.IdlInstructionInvalidProgram,
    { name: "IdlInstructionInvalidProgram", msg: "The transaction was given an invalid program for the IDL instruction" },
  ],

  // Constraints.
  [LangErrorCode.ConstraintMut, { name: "ConstraintMut", msg: "A mut constraint was violated" }],
  [LangErrorCode.ConstraintHasOne, { name: "ConstraintHasOne", msg: "A has_one constraint was violated" }],
  [LangErrorCode.ConstraintSigner, { name: "ConstraintSigner", msg: "A signer constraint was violated" }],
  [LangErrorCode.ConstraintRaw, { name: "ConstraintRaw", msg: "A raw constraint was violated" }],
  [LangErrorCode.ConstraintOwner, { name: "ConstraintOwner", msg: "An owner constraint was violated" }],
  [LangErrorCode.ConstraintRentExempt, { name: "ConstraintRentExempt", msg: "A rent exempt constraint was violated" }],
  [LangErrorCode.ConstraintSeeds, { name: "ConstraintSeeds", msg: "A seeds constraint was violated" }],
  [LangErrorCode.ConstraintExecutable, { name: "ConstraintExecutable", msg: "An executable constraint was violated" }],
  [LangErrorCode.ConstraintState, { name: "ConstraintState", msg: "A state constraint was violated" }],
  [LangErrorCode.ConstraintAssociated, { name: "ConstraintAssociated", msg: "An associated constraint was violated" }],
  [
    LangErrorCode.ConstraintAssociatedInit,
    { name: "ConstraintAssociatedInit", msg: "An associated init constraint was violated" },
  ],
  [LangErrorCode.ConstraintClose, { name: "ConstraintClose", msg: "A close constraint was violated" }],
  [LangErrorCode.ConstraintAddress, { name: "ConstraintAddress", msg: "An address constraint was violated" }],

  // Accounts.
  [
    LangErrorCode.AccountDiscriminatorAlreadySet,
    { name: "AccountDiscriminatorAlreadySet", msg: "The account discriminator was already set on this account" },
  ],
  [
    LangErrorCode.AccountDiscriminatorNotFound,
    { name: "AccountDiscriminatorNotFound", msg: "No 8 byte discriminator was found on the account" },
  ],
  [
    LangErrorCode.AccountDiscriminatorMismatch,
    { name: "AccountDiscriminatorMismatch", msg: "8 byte discriminator did not match what was expected" },
  ],
  [LangErrorCode.AccountDidNotDeserialize, { name: "AccountDidNotDeserialize", msg: "Failed to deserialize the account" }],
  [LangErrorCode.AccountDidNotSerialize, { name: "AccountDidNotSerialize", msg: "Failed to serialize the account" }],
  [
    LangErrorCode.AccountNotEnoughKeys,
    { name: "AccountNotEnoughKeys", msg: "Not enough account keys given to the instruction" },
  ],
  [LangErrorCode.AccountNotMutable, { name: "AccountNotMutable", msg: "The given account is not mutable" }],
  [
    LangErrorCode.AccountNotProgramOwned,
    { name: "AccountNotProgramOwned", msg: "The given account is not owned by the executing program" },
  ],
  [LangErrorCode.InvalidProgramId, { name: "InvalidProgramId", msg: "Program ID was not as expected" }],
  [
    LangErrorCode.InvalidProgramIdExecutable,
    { name: "InvalidProgramIdExecutable", msg: "Program account is not executable" },
  ],

  // State.
  [
    LangErrorCode.StateInvalidAddress,
    { name: "StateInvalidAddress", msg: "The given state account does not have the correct address" },
  ],

  // Misc.
  [
    LangErrorCode.Deprecated,
    { name: "Deprecated", msg: "The API being used is deprecated and should no longer be used" },
  ],
]);
