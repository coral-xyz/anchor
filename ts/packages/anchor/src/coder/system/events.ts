import { EventCoder } from "../index.js";
import { Idl } from "../../idl.js";
import { Event } from "../../program/event";
import { IdlEvent } from "../../idl";
import { VersionedTransactionResponse } from "@solana/web3.js";

export class SystemEventsCoder implements EventCoder {
  constructor(_idl: Idl) {}

  decode<E extends IdlEvent = IdlEvent, T = Record<string, string>>(
    _log: string
  ): Event<E, T> | null {
    throw new Error("SystemProgram does not have events.");
  }
  parseCpiEvents<E extends IdlEvent = IdlEvent, T = Record<string, string>>(
    _transactionResponse: VersionedTransactionResponse
  ): Event<E, T>[] {
    throw new Error("SystemProgram does not have CPI events.");
  }
}
