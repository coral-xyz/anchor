import { PublicKey } from "@solana/web3.js";
import { Idl, IdlAccount } from "../../idl.js";
import { SimulateFn } from "./simulate.js";
import {
  AllInstructions,
  InstructionContextFn,
  MakeInstructionsNamespace,
} from "./types";
import { IdlCoder } from "../../coder/borsh/idl";
import { decode } from "../../utils/bytes/base64";

export default class ViewFactory {
  public static build<IDL extends Idl, I extends AllInstructions<IDL>>(
    programId: PublicKey,
    idlIx: AllInstructions<IDL>,
    simulateFn: SimulateFn<IDL>,
    idl: IDL
  ): ViewFn<IDL, I> | undefined {
    const isMut = idlIx.accounts.find((a: IdlAccount) => a.isMut);
    const hasReturn = !!idlIx.returns;
    if (isMut || !hasReturn) return;

    const view: ViewFn<IDL> = async (...args) => {
      let simulationResult = await simulateFn(...args);
      const returnPrefix = `Program return: ${programId} `;
      let returnLog = simulationResult.raw.find((l) =>
        l.startsWith(returnPrefix)
      );
      if (!returnLog) {
        throw new Error("View expected return log");
      }
      let returnData = decode(returnLog.slice(returnPrefix.length));
      let returnType = idlIx.returns;
      if (!returnType) {
        throw new Error("View expected return type");
      }
      const coder = IdlCoder.fieldLayout(
        { type: returnType },
        Array.from([...(idl.accounts ?? []), ...(idl.types ?? [])])
      );
      return coder.decode(returnData);
    };
    return view;
  }
}

export type ViewNamespace<
  IDL extends Idl = Idl,
  I extends AllInstructions<IDL> = AllInstructions<IDL>
> = MakeInstructionsNamespace<IDL, I, Promise<any>>;

/**
 * ViewFn is a single method generated from an IDL. It simulates a method
 * against a cluster configured by the provider, and then parses the events
 * and extracts return data from the raw logs emitted during the simulation.
 */
export type ViewFn<
  IDL extends Idl = Idl,
  I extends AllInstructions<IDL> = AllInstructions<IDL>
> = InstructionContextFn<IDL, I, Promise<any>>;
