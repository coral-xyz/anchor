import { Transaction } from "@solana/web3.js";
import { IdlInstruction } from "../../idl";
import { splitArgsAndCtx } from "../context";
import { InstructionFn } from "./instruction";

/**
 * Dynamically generated transaction namespace.
 */
export interface TransactionNamespace {
  [key: string]: TransactionFn;
}

/**
 * Tx is a function to create a `Transaction` for a given program instruction.
 */
export type TransactionFn = (...args: any[]) => Transaction;

export default class TransactionFactory {
  // Builds the transaction namespace.
  public static build(
    idlIx: IdlInstruction,
    ixFn: InstructionFn
  ): TransactionFn {
    const txFn = (...args: any[]): Transaction => {
      const [, ctx] = splitArgsAndCtx(idlIx, [...args]);
      const tx = new Transaction();
      if (ctx.instructions !== undefined) {
        tx.add(...ctx.instructions);
      }
      tx.add(ixFn(...args));
      return tx;
    };

    return txFn;
  }
}
