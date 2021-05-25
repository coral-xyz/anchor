import { PublicKey } from "@solana/web3.js";
import Provider from "../../provider";
import { IdlInstruction } from "../../idl";
import { translateError } from "../common";
import { splitArgsAndCtx } from "../context";
import { TransactionFn } from "./transaction";
import { EventParser } from "../event";
import Coder from "../../coder";
import { Idl } from "../../idl";

/**
 * Dynamically generated simualte namespace.
 */
export interface SimulateNamespace {
  [key: string]: SimulateFn;
}

/**
 * RpcFn is a single method generated from an IDL. It simulates a method
 * against a cluster configured by the provider, returning a list of all the
 * events and raw logs that were emitted during the execution of the
 * method.
 */
export type SimulateFn = (...args: any[]) => Promise<SimulateResponse>;

type SimulateResponse = {
  events: Event[];
  raw: string[];
};

export default class SimulateFactory {
  // Builds the rpc namespace.
  public static build(
    idlIx: IdlInstruction,
    txFn: TransactionFn,
    idlErrors: Map<number, string>,
    provider: Provider,
    coder: Coder,
    programId: PublicKey,
    idl: Idl
  ): SimulateFn {
    const simulate = async (...args: any[]): Promise<SimulateResponse> => {
      const tx = txFn(...args);
      const [, ctx] = splitArgsAndCtx(idlIx, [...args]);
      let resp = undefined;
      try {
        resp = await provider.simulate(tx, ctx.signers, ctx.options);
      } catch (err) {
        console.log("Translating error", err);
        let translatedErr = translateError(idlErrors, err);
        if (translatedErr === null) {
          throw err;
        }
        throw translatedErr;
      }
      if (resp === undefined) {
        throw new Error("Unable to simulate transaction");
      }
      if (resp.value.err) {
        throw new Error(`Simulate error: ${resp.value.err.toString()}`);
      }
      const logs = resp.value.logs;
      if (!logs) {
        throw new Error("Simulated logs not found");
      }

      const events = [];
      if (idl.events) {
        let parser = new EventParser(coder, programId);
        parser.parseLogs(logs, (event) => {
          events.push(event);
        });
      }
      return { events, raw: logs };
    };

    return simulate;
  }
}
