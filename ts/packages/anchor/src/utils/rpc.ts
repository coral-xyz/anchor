import { Buffer } from "buffer";
import {
  AccountInfo,
  AccountMeta,
  Connection,
  PublicKey,
  TransactionSignature,
  TransactionInstruction,
  Commitment,
  Signer,
  RpcResponseAndContext,
  SimulatedTransactionResponse,
  SendTransactionError,
  Context,
  VersionedTransaction,
} from "@solana/web3.js";
import { chunks } from "../utils/common.js";
import { Address, translateAddress } from "../program/common.js";
import Provider, { getProvider } from "../provider.js";
import {
  type as pick,
  number,
  string,
  array,
  boolean,
  literal,
  union,
  optional,
  nullable,
  coerce,
  create,
  unknown,
  any,
  Struct,
} from "superstruct";

/**
 * Sends a transaction to a program with the given accounts and instruction
 * data.
 */
export async function invoke(
  programId: Address,
  accounts?: Array<AccountMeta>,
  data?: Buffer,
  provider?: Provider
): Promise<TransactionSignature> {
  programId = translateAddress(programId);
  if (!provider) {
    provider = getProvider();
  }

  const txInstructions: TransactionInstruction[] = [];
  txInstructions.push(
    new TransactionInstruction({
      programId,
      keys: accounts ?? [],
      data,
    })
  );

  if (provider.sendAndConfirm === undefined) {
    throw new Error(
      "This function requires 'Provider.sendAndConfirm' to be implemented."
    );
  }

  return await provider.sendAndConfirm(txInstructions, []);
}

const GET_MULTIPLE_ACCOUNTS_LIMIT: number = 99;

export async function getMultipleAccounts(
  connection: Connection,
  publicKeys: PublicKey[],
  commitment?: Commitment
): Promise<
  Array<null | { publicKey: PublicKey; account: AccountInfo<Buffer> }>
> {
  const results = await getMultipleAccountsAndContext(
    connection,
    publicKeys,
    commitment
  );
  return results.map((result) => {
    return result
      ? { publicKey: result.publicKey, account: result.account }
      : null;
  });
}

export async function getMultipleAccountsAndContext(
  connection: Connection,
  publicKeys: PublicKey[],
  commitment?: Commitment
): Promise<
  Array<null | {
    context: Context;
    publicKey: PublicKey;
    account: AccountInfo<Buffer>;
  }>
> {
  if (publicKeys.length <= GET_MULTIPLE_ACCOUNTS_LIMIT) {
    return await getMultipleAccountsAndContextCore(
      connection,
      publicKeys,
      commitment
    );
  } else {
    const batches = chunks(publicKeys, GET_MULTIPLE_ACCOUNTS_LIMIT);
    const results = await Promise.all<
      Array<null | {
        publicKey: PublicKey;
        account: AccountInfo<Buffer>;
        context: Context;
      }>
    >(
      batches.map((batch) =>
        getMultipleAccountsAndContextCore(connection, batch, commitment)
      )
    );
    return results.flat();
  }
}

async function getMultipleAccountsAndContextCore(
  connection: Connection,
  publicKeys: PublicKey[],
  commitmentOverride?: Commitment
): Promise<
  Array<null | {
    publicKey: PublicKey;
    account: AccountInfo<Buffer>;
    context: Context;
  }>
> {
  const commitment = commitmentOverride ?? connection.commitment;
  const { value: accountInfos, context } =
    await connection.getMultipleAccountsInfoAndContext(publicKeys, commitment);
  const accounts = accountInfos.map((account, idx) => {
    if (account === null) {
      return null;
    }
    return {
      publicKey: publicKeys[idx],
      account,
      context,
    };
  });

  return accounts;
}

export type SuccessfulTxSimulationResponse = Omit<
  SimulatedTransactionResponse,
  "err"
>;
