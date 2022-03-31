import {
  Connection,
  Signer,
  PublicKey,
  Transaction,
  TransactionSignature,
  ConfirmOptions,
  SimulatedTransactionResponse,
  Commitment,
  SendTransactionError,
  SendOptions,
  RpcResponseAndContext,
} from "@solana/web3.js";
import { bs58 } from "./utils/bytes/index.js";
import { isBrowser } from "./utils/common.js";
import {
  type as pick,
  number,
  string,
  array,
  boolean,
  literal,
  record,
  union,
  optional,
  nullable,
  coerce,
  instance,
  create,
  tuple,
  unknown,
  any,
  Struct,
} from "superstruct";

export interface Provider {
  readonly connection: Connection;

  send?(
    tx: Transaction,
    signers?: Signer[],
    opts?: SendOptions
  ): Promise<TransactionSignature>;
  sendAndConfirm?(
    tx: Transaction,
    signers?: Signer[],
    opts?: ConfirmOptions
  ): Promise<TransactionSignature>;
  sendAll?(
    txWithSigners: { tx: Transaction; signers?: Signer[] }[],
    opts?: ConfirmOptions
  ): Promise<Array<TransactionSignature>>;
  simulate?(
    tx: Transaction,
    signers?: Signer[],
    commitment?: Commitment,
    includeAccounts?: boolean | PublicKey[]
  ): Promise<SuccessfulTxSimulationResponse>;
}

/**
 * The network and wallet context used to send transactions paid for and signed
 * by the provider.
 */
export default class AnchorProvider implements Provider {
  /**
   * @param connection The cluster connection where the program is deployed.
   * @param wallet     The wallet used to pay for and sign all transactions.
   * @param opts       Transaction confirmation options to use by default.
   */
  constructor(
    readonly connection: Connection,
    readonly wallet: Wallet,
    readonly opts: ConfirmOptions
  ) {}

  static defaultOptions(): ConfirmOptions {
    return {
      preflightCommitment: "processed",
      commitment: "processed",
    };
  }

  /**
   * Returns a `Provider` with a wallet read from the local filesystem.
   *
   * @param url  The network cluster url.
   * @param opts The default transaction confirmation options.
   *
   * (This api is for Node only.)
   */
  static local(url?: string, opts?: ConfirmOptions): AnchorProvider {
    if (isBrowser) {
      throw new Error(`Provider local is not available on browser.`);
    }
    opts = opts ?? AnchorProvider.defaultOptions();
    const connection = new Connection(
      url ?? "http://localhost:8899",
      opts.preflightCommitment
    );
    const NodeWallet = require("./nodewallet.js").default;
    const wallet = NodeWallet.local();
    return new AnchorProvider(connection, wallet, opts);
  }

  /**
   * Returns a `Provider` read from the `ANCHOR_PROVIDER_URL` environment
   * variable
   *
   * (This api is for Node only.)
   */
  static env(): AnchorProvider {
    if (isBrowser) {
      throw new Error(`Provider env is not available on browser.`);
    }

    const process = require("process");
    const url = process.env.ANCHOR_PROVIDER_URL;
    if (url === undefined) {
      throw new Error("ANCHOR_PROVIDER_URL is not defined");
    }
    const options = AnchorProvider.defaultOptions();
    const connection = new Connection(url, options.commitment);
    const NodeWallet = require("./nodewallet.js").default;
    const wallet = NodeWallet.local();

    return new AnchorProvider(connection, wallet, options);
  }

  /**
   * Sends the given transaction, paid for and signed by the provider's wallet.
   *
   * @param tx      The transaction to send.
   * @param signers The signers of the transaction.
   * @param opts    Transaction confirmation options.
   */
  async sendAndConfirm(
    tx: Transaction,
    signers?: Signer[],
    opts?: ConfirmOptions
  ): Promise<TransactionSignature> {
    if (opts === undefined) {
      opts = this.opts;
    }

    tx.feePayer = this.wallet.publicKey;
    tx.recentBlockhash = (
      await this.connection.getRecentBlockhash(opts.preflightCommitment)
    ).blockhash;

    tx = await this.wallet.signTransaction(tx);
    (signers ?? []).forEach((kp) => {
      tx.partialSign(kp);
    });

    const rawTx = tx.serialize();

    try {
      return await sendAndConfirmRawTransaction(this.connection, rawTx, opts);
    } catch (err) {
      // thrown if the underlying 'confirmTransaction' encounters a failed tx
      // the 'confirmTransaction' error does not return logs so we make another rpc call to get them
      if (err instanceof ConfirmError) {
        // choose the shortest available commitment for 'getTransaction'
        // (the json RPC does not support any shorter than "confirmed" for 'getTransaction')
        // because that will see the tx sent with `sendAndConfirmRawTransaction` no matter which
        // commitment `sendAndConfirmRawTransaction` used
        const failedTx = await this.connection.getTransaction(
          bs58.encode(tx.signature!),
          { commitment: "confirmed" }
        );
        if (!failedTx) {
          throw err;
        } else {
          const logs = failedTx.meta?.logMessages;
          throw !logs ? err : new SendTransactionError(err.message, logs);
        }
      } else {
        throw err;
      }
    }
  }

  /**
   * Similar to `send`, but for an array of transactions and signers.
   */
  async sendAll(
    txWithSigners: { tx: Transaction; signers?: Signer[] }[],
    opts?: ConfirmOptions
  ): Promise<Array<TransactionSignature>> {
    if (opts === undefined) {
      opts = this.opts;
    }
    const blockhash = await this.connection.getRecentBlockhash(
      opts.preflightCommitment
    );

    let txs = txWithSigners.map((r) => {
      let tx = r.tx;
      let signers = r.signers ?? [];

      tx.feePayer = this.wallet.publicKey;
      tx.recentBlockhash = blockhash.blockhash;

      signers.forEach((kp) => {
        tx.partialSign(kp);
      });

      return tx;
    });

    const signedTxs = await this.wallet.signAllTransactions(txs);

    const sigs: TransactionSignature[] = [];

    for (let k = 0; k < txs.length; k += 1) {
      const tx = signedTxs[k];
      const rawTx = tx.serialize();
      sigs.push(
        await sendAndConfirmRawTransaction(this.connection, rawTx, opts)
      );
    }

    return sigs;
  }

  /**
   * Simulates the given transaction, returning emitted logs from execution.
   *
   * @param tx      The transaction to send.
   * @param signers The signers of the transaction.
   * @param opts    Transaction confirmation options.
   */
  async simulate(
    tx: Transaction,
    signers?: Signer[],
    commitment?: Commitment,
    includeAccounts?: boolean | PublicKey[]
  ): Promise<SuccessfulTxSimulationResponse> {
    tx.feePayer = this.wallet.publicKey;

    const result = await simulateTransaction(
      this.connection,
      tx,
      this.wallet,
      signers,
      commitment,
      includeAccounts
    );

    if (result.value.err) {
      throw new SimulateError(result.value);
    }

    return result.value;
  }
}

// copy from @solana/web3.js that allows a wallet to sign
// and has a commitment param
async function simulateTransaction(
  connection: Connection,
  transactionOrMessage: Transaction,
  wallet?: Wallet,
  signers?: Array<Signer>,
  commitment?: Commitment,
  includeAccounts?: boolean | Array<PublicKey>
): Promise<RpcResponseAndContext<SimulatedTransactionResponse>> {
  let transaction;
  if (transactionOrMessage instanceof Transaction) {
    transaction = transactionOrMessage;
  } else {
    transaction = Transaction.populate(transactionOrMessage);
  }

  if (transaction.nonceInfo && signers) {
    if (wallet) {
      transaction = await wallet.signTransaction(transaction);
    }
    transaction.sign(...signers);
  } else {
    //@ts-expect-error
    let disableCache = connection._disableBlockhashCaching;
    for (;;) {
      // @ts-expect-error
      transaction.recentBlockhash = await connection._recentBlockhash(
        disableCache
      );

      if (wallet) {
        transaction = await wallet.signTransaction(transaction);
      }
      if (!signers) break;

      transaction.sign(...signers);
      if (!transaction.signature) {
        throw new Error("!signature"); // should never happen
      }

      const signature = transaction.signature.toString("base64");
      if (
        // @ts-expect-error
        !connection._blockhashInfo.simulatedSignatures.includes(signature) &&
        // @ts-expect-error
        !connection._blockhashInfo.transactionSignatures.includes(signature)
      ) {
        // The signature of connection transaction has not been seen before with the
        // current recentBlockhash, all done. Let's break
        // @ts-expect-error
        connection._blockhashInfo.simulatedSignatures.push(signature);
        break;
      } else {
        // This transaction would be treated as duplicate (its derived signature
        // matched to one of already recorded signatures).
        // So, we must fetch a new blockhash for a different signature by disabling
        // our cache not to wait for the cache expiration (BLOCKHASH_CACHE_TIMEOUT_MS).
        disableCache = true;
      }
    }
  }

  const message = transaction._compile();
  const signData = message.serialize();
  const wireTransaction = transaction._serialize(signData);
  const encodedTransaction = wireTransaction.toString("base64");
  const config: any = {
    encoding: "base64",
    commitment: commitment ?? connection.commitment,
  };

  if (includeAccounts) {
    const addresses = (
      Array.isArray(includeAccounts) ? includeAccounts : message.nonProgramIds()
    ).map((key) => key.toBase58());

    config["accounts"] = {
      encoding: "base64",
      addresses,
    };
  }

  if (signers) {
    config.sigVerify = true;
  }

  const args = [encodedTransaction, config];
  // @ts-expect-error
  const unsafeRes = await connection._rpcRequest("simulateTransaction", args);
  const res = create(unsafeRes, SimulatedTransactionResponseStruct);
  if ("error" in res) {
    let logs;
    if ("data" in res.error) {
      logs = res.error.data.logs;
      if (logs && Array.isArray(logs)) {
        const traceIndent = "\n    ";
        const logTrace = traceIndent + logs.join(traceIndent);
        console.error(res.error.message, logTrace);
      }
    }
    throw new SendTransactionError(
      "failed to simulate transaction: " + res.error.message,
      logs
    );
  }
  return res.result;
}

// copy from @solana/web3.js
function jsonRpcResult<T, U>(schema: Struct<T, U>) {
  return coerce(createRpcResult(schema), UnknownRpcResult, (value) => {
    if ("error" in value) {
      return value;
    } else {
      return {
        ...value,
        result: create(value.result, schema),
      };
    }
  });
}

// copy from @solana/web3.js
const UnknownRpcResult = createRpcResult(unknown());

// copy from @solana/web3.js
function createRpcResult<T, U>(result: Struct<T, U>) {
  return union([
    pick({
      jsonrpc: literal("2.0"),
      id: string(),
      result,
    }),
    pick({
      jsonrpc: literal("2.0"),
      id: string(),
      error: pick({
        code: unknown(),
        message: string(),
        data: optional(any()),
      }),
    }),
  ]);
}

// copy from @solana/web3.js
function jsonRpcResultAndContext<T, U>(value: Struct<T, U>) {
  return jsonRpcResult(
    pick({
      context: pick({
        slot: number(),
      }),
      value,
    })
  );
}

// copy from @solana/web3.js
const SimulatedTransactionResponseStruct = jsonRpcResultAndContext(
  pick({
    err: nullable(union([pick({}), string()])),
    logs: nullable(array(string())),
    accounts: optional(
      nullable(
        array(
          nullable(
            pick({
              executable: boolean(),
              owner: string(),
              lamports: number(),
              data: array(string()),
              rentEpoch: optional(number()),
            })
          )
        )
      )
    ),
    unitsConsumed: optional(number()),
  })
);

export type SuccessfulTxSimulationResponse = Omit<
  SimulatedTransactionResponse,
  "err"
>;

class SimulateError extends Error {
  constructor(
    readonly simulationResponse: SimulatedTransactionResponse,
    message?: string
  ) {
    super(message);
  }
}

export type SendTxRequest = {
  tx: Transaction;
  signers: Array<Signer | undefined>;
};

/**
 * Wallet interface for objects that can be used to sign provider transactions.
 */
export interface Wallet {
  signTransaction(tx: Transaction): Promise<Transaction>;
  signAllTransactions(txs: Transaction[]): Promise<Transaction[]>;
  publicKey: PublicKey;
}

// Copy of Connection.sendAndConfirmRawTransaction that throws
// a better error if 'confirmTransaction` returns an error status
async function sendAndConfirmRawTransaction(
  connection: Connection,
  rawTransaction: Buffer,
  options?: ConfirmOptions
): Promise<TransactionSignature> {
  const sendOptions = options && {
    skipPreflight: options.skipPreflight,
    preflightCommitment: options.preflightCommitment || options.commitment,
  };

  const signature = await connection.sendRawTransaction(
    rawTransaction,
    sendOptions
  );

  const status = (
    await connection.confirmTransaction(
      signature,
      options && options.commitment
    )
  ).value;

  if (status.err) {
    throw new ConfirmError(
      `Raw transaction ${signature} failed (${JSON.stringify(status)})`
    );
  }

  return signature;
}

class ConfirmError extends Error {
  constructor(message?: string) {
    super(message);
  }
}

/**
 * Sets the default provider on the client.
 */
export function setProvider(provider: AnchorProvider) {
  _provider = provider;
}

/**
 * Returns the default provider being used by the client.
 */
export function getProvider(): AnchorProvider {
  if (_provider === null) {
    return AnchorProvider.local();
  }
  return _provider;
}

// Global provider used as the default when a provider is not given.
let _provider: AnchorProvider | null = null;
