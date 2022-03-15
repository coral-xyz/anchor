import {
  PublicKey,
  RpcResponseAndContext,
  SimulatedTransactionResponse,
} from "@solana/web3.js";
import Provider from "../../provider.js";
import { Idl, IdlEvent } from "../../idl.js";
import { SimulateFn } from "./simulate.js";
import {
  AllInstructions,
  IdlTypes,
  InstructionContextFn,
  MakeInstructionsNamespace,
} from "./types";
import { IdlCoder } from "../../coder/borsh/idl";
import { decode } from "../../utils/bytes/base64";

export default class ViewsFactory {
  public static build<IDL extends Idl, I extends AllInstructions<IDL>>(
    programId: PublicKey,
    idlIx: AllInstructions<IDL>,
    simulateFn: SimulateFn<IDL>
  ): ViewsFn<IDL, I> | undefined {
    const isMut = idlIx.accounts.find((a) => a.isMut);
    const hasReturn = !!idlIx.returns;
    if (isMut || !hasReturn) return;

    const view: ViewsFn<IDL> = async (...args) => {
      let simulationResult = await simulateFn(...args);
      const returnPrefix = `Program return: ${programId} `;
      let returnLog = simulationResult.raw.find((l) =>
        l.startsWith(returnPrefix)
      );
      if (!returnLog) {
        throw new Error("Expected return log");
      }
      let returnData = decode(returnLog.slice(returnPrefix.length));
      let returnType = idlIx.returns;

      if (!returnType) {
        throw new Error("Expected return type");
      }
      const coder = IdlCoder.fieldLayout({ type: returnType });
      return coder.decode(returnData);
    };
    return view;
  }
}

export type ViewsNamespace<
  IDL extends Idl = Idl,
  I extends AllInstructions<IDL> = AllInstructions<IDL>
> = MakeInstructionsNamespace<IDL, I, Promise<ViewsResponse>>;

/**
 * ViewsFn is a single method generated from an IDL. It simulates a method
 * against a cluster configured by the provider, and then parses the events
 * and extracts return data from the raw logs emitted during the simulation.
 */
export type ViewsFn<
  IDL extends Idl = Idl,
  I extends AllInstructions<IDL> = AllInstructions<IDL>
> = InstructionContextFn<IDL, I, Promise<ViewsResponse>>;

export type ViewsResponse = any;
