import { Buffer } from "buffer";
import {
  Keypair,
  PublicKey,
  Transaction,
  Version,
  VersionedTransaction,
} from "@solana/web3.js";
import { Wallet } from "./provider";

/**
 * Node only wallet.
 */
export default class NodeWallet implements Wallet {
  constructor(readonly payer: Keypair) {}

  static local(): NodeWallet | never {
    const process = require("process");

    if (!process.env.ANCHOR_WALLET || process.env.ANCHOR_WALLET === "") {
      throw new Error(
        "expected environment variable `ANCHOR_WALLET` is not set."
      );
    }

    const payer = Keypair.fromSecretKey(
      Buffer.from(
        JSON.parse(
          require("fs").readFileSync(process.env.ANCHOR_WALLET, {
            encoding: "utf-8",
          })
        )
      )
    );

    return new NodeWallet(payer);
  }

  async signTransaction<T extends Transaction | VersionedTransaction>(
    tx: T
  ): Promise<T> {
    // Type checking tx using instanceof failed to detect VersionedTransaction
    // or Transaction types sometimes. Can't figure out why. Using this
    // approach instead.
    // if (typeof tx["partialSign"] === "function") {
    //   (tx as Transaction).partialSign(this.payer);
    if (typeof tx["sign"] === "function") {
      (tx as VersionedTransaction).sign([this.payer]);
    } else {
      console.log("Failed Transaction Object: ", tx);
      throw new Error(`Object of type ${typeof tx} cannot be signed`);
    }

    // if (tx instanceof VersionedTransaction) {
    //   tx.sign([this.payer]);
    // } else if (tx instanceof Transaction) {
    //   tx.partialSign(this.payer);
    // } else {
    //   console.log("Failed Transaction Type: ", tx);
    //   throw new Error(`Object of type ${typeof tx} cannot be signed`);
    // }

    return tx;
  }

  async signAllTransactions<T extends Transaction | VersionedTransaction>(
    txs: T[]
  ): Promise<T[]> {
    return txs.map((t) => {
      if (t instanceof VersionedTransaction) {
        t.sign([this.payer]);
      } else {
        t.partialSign(this.payer);
      }
      return t;
    });
  }

  get publicKey(): PublicKey {
    return this.payer.publicKey;
  }
}
