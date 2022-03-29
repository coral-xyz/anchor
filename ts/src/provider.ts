import {
  Connection,
  Signer,
  PublicKey,
  Transaction,
  TransactionSignature,
  ConfirmOptions,
  RpcResponseAndContext,
  SimulatedTransactionResponse,
  Commitment,
  SendTransactionError,
  SendOptions,
} from "@solana/web3.js";
import { bs58 } from "./utils/bytes/index.js";
import { isBrowser } from "./utils/common.js";

export interface Provider {
  readonly connection: Connection,

  send?(req: SendTxRequest, opts?: SendOptions): Promise<TransactionSignature>;
  sendAndConfirm?(req: SendTxRequest, opts?: ConfirmOptions): Promise<TransactionSignature>;
  sendAll?(reqs: Array<SendTxRequest>, opts?: ConfirmOptions): Promise<Array<TransactionSignature>>;
  simulate?(req: SendTxRequest, preflightCommitment?: Commitment, includeAccounts?: boolean | PublicKey[]): Promise<SuccessfulTxSimulationResponse>;
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
   * @param req     The transaction request to send.
   * @param opts    Transaction confirmation options.
   */
  async send(
    req: SendTxRequest,
    opts?: ConfirmOptions
  ): Promise<TransactionSignature> {
    if (opts === undefined) {
      opts = this.opts;
    }

    req.tx.feePayer = this.wallet.publicKey;
    req.tx.recentBlockhash = (
      await this.connection.getRecentBlockhash(opts.preflightCommitment)
    ).blockhash;

    req.tx = await this.wallet.signTransaction(req.tx);
    req.signers
      .filter((s): s is Signer => s !== undefined)
      .forEach((kp) => {
        req.tx.partialSign(kp);
      });

    const rawTx = req.tx.serialize();

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
          bs58.encode(req.tx.signature!),
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
    reqs: Array<SendTxRequest>,
    opts?: ConfirmOptions
  ): Promise<Array<TransactionSignature>> {
    if (opts === undefined) {
      opts = this.opts;
    }
    const blockhash = await this.connection.getRecentBlockhash(
      opts.preflightCommitment
    );

    let txs = reqs.map((r) => {
      let tx = r.tx;
      let signers = r.signers;

      if (signers === undefined) {
        signers = [];
      }

      tx.feePayer = this.wallet.publicKey;
      tx.recentBlockhash = blockhash.blockhash;

      signers
        .filter((s): s is Signer => s !== undefined)
        .forEach((kp) => {
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
   * @param req     The transaction request to send.
   * @param opts    Transaction confirmation options.
   */
  async simulate(
    req: SendTxRequest,
    commitmentMaybe?: Commitment,
    includeAccounts?: boolean | PublicKey[]
  ): Promise<SuccessfulTxSimulationResponse> {
    const commitment = commitmentMaybe ?? this.connection.commitment;
    req.tx.feePayer = this.wallet.publicKey;
    req.tx = await this.wallet.signTransaction(req.tx);

    // this allows us to pass a commitment arg into `simulateTransaction`
    // instead of having to copy the function here and adjust it
    const commitmentOverride = {
      get: function (_, prop) {
        if (prop === "commitment") {
          return commitment;
        } else {
          // this is the normal way to return all other props
          // without modifying them.
          // @ts-expect-error
          return Reflect.get(...arguments);
        }
      },
    };
    const simulateTransactionWithCommitment = this.connection.simulateTransaction.bind(new Proxy(this.connection, commitmentOverride));
    const result = await simulateTransactionWithCommitment.bind(
      req.tx,
      req.signers,
      includeAccounts
    );

    if (result.value.err) {
      throw new SimulateError(result.value);
    }

    return result.value;
  }
}

type SuccessfulTxSimulationResponse = Omit<SimulatedTransactionResponse, "err">;

class SimulateError extends Error {
  constructor(readonly simulationResponse: SimulatedTransactionResponse, message?: string) {
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
