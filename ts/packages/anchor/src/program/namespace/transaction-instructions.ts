import { TransactionInstruction } from "@solana/web3.js";
import { Idl, IdlInstruction } from "../../idl.js";
import { splitArgsAndCtx } from "../context.js";
import { InstructionFn } from "./instruction.js";
import {
  AllInstructions,
  InstructionContextFn,
  MakeInstructionsNamespace,
} from "./types.js";

export default class TransactionInstructionsFactory {
  public static build<IDL extends Idl, I extends AllInstructions<IDL>>(
    idlIx: I,
    ixFn: InstructionFn<IDL, I>
  ): TransactionInstructionsFn<IDL, I> {
    const txIxsFn: TransactionInstructionsFn<IDL, I> = (
      ...args
    ): TransactionInstruction[] => {
      const [, ctx] = splitArgsAndCtx(idlIx, [...args]);
      const tx: TransactionInstruction[] = [];
      if (ctx.preInstructions && ctx.instructions) {
        throw new Error("instructions is deprecated, use preInstructions");
      }
      ctx.preInstructions?.forEach((ix) => tx.push(ix));
      ctx.instructions?.forEach((ix) => tx.push(ix));
      tx.push(ixFn(...args));
      ctx.postInstructions?.forEach((ix) => tx.push(ix));
      return tx;
    };

    return txIxsFn;
  }
}

/**
 * The namespace provides functions to build [[TransactionInstruction\[\]]] objects for each
 * method of a program.
 *
 * ## Usage
 *
 * ```javascript
 * program.transactionInstructions.<method>(...args, ctx);
 * ```
 *
 * ## Parameters
 *
 * 1. `args` - The positional arguments for the program. The type and number
 *    of these arguments depend on the program being used.
 * 2. `ctx`  - [[Context]] non-argument parameters to pass to the method.
 *    Always the last parameter in the method call.
 *
 * ## Example
 *
 * To create an instruction for the `increment` method above,
 *
 * ```javascript
 * const tx = await program.transactionInstructions.increment({
 *   accounts: {
 *     counter,
 *   },
 * });
 * ```
 */
export type TransactionInstructionsNamespace<
  IDL extends Idl = Idl,
  I extends AllInstructions<IDL> = AllInstructions<IDL>
> = MakeInstructionsNamespace<IDL, I, TransactionInstruction[]>;

/**
 * TxIxs is a function to create a `TransactionInstruction[]` for a given program instruction.
 */
export type TransactionInstructionsFn<
  IDL extends Idl = Idl,
  I extends AllInstructions<IDL> = AllInstructions<IDL>
> = InstructionContextFn<IDL, I, TransactionInstruction[]>;
