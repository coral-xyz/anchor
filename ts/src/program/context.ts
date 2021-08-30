import {
  AccountMeta,
  Signer,
  ConfirmOptions,
  TransactionInstruction,
} from "@solana/web3.js";
import { Address } from "./common";
import { IdlInstruction } from "../idl";

/**
 * Context provides all non-argument inputs for generating Anchor transactions.
 */
export type Context = {
  /**
   * Accounts used in the instruction context.
   */
  accounts?: Accounts;

  /**
   * All accounts to pass into an instruction *after* the main `accounts`.
   * This can be used for optional or otherwise unknown accounts.
   */
  remainingAccounts?: AccountMeta[];

  /**
   * Accounts that must sign a given transaction.
   */
  signers?: Array<Signer>;

  /**
   * Instructions to run *before* a given method. Often this is used, for
   * example to create accounts prior to executing a method.
   */
  instructions?: TransactionInstruction[];

  /**
   * Commitment parameters to use for a transaction.
   */
  options?: ConfirmOptions;

  /**
   * @hidden
   *
   * Private namespace for development.
   */
  __private?: { logAccounts: boolean };
};

/**
 * The type which is passed in for an array of accounts. Thus, whether the account is a signer of is mutable
 * can be determined from the frontend
 */
export type AccountsArray = {
  address: Address;
  isSigner?: boolean;
  isWriteable?: boolean;
}[];

/**
 * A set of accounts mapping one-to-one to the program's accounts struct, i.e.,
 * the type deriving `#[derive(Accounts)]`.
 *
 * The name of each field should match the name for that account in the IDL.
 *
 * If multiple accounts are nested in the rust program, then they should be
 * nested here.
 */
export type Accounts = {
  [key: string]: Address | Accounts | AccountsArray;
};

export function splitArgsAndCtx(
  idlIx: IdlInstruction,
  args: any[]
): [any[], Context] {
  let options = {};

  const inputLen = idlIx.args ? idlIx.args.length : 0;
  if (args.length > inputLen) {
    if (args.length !== inputLen + 1) {
      throw new Error("provided too many arguments ${args}");
    }
    options = args.pop();
  }

  return [args, options];
}
