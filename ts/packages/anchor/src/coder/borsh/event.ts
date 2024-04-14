import { Buffer } from "buffer";
import { Layout } from "buffer-layout";
import * as base64 from "../../utils/bytes/base64.js";
import * as bs58 from "../../utils/bytes/bs58.js";
import { Idl } from "../../idl.js";
import { IdlCoder } from "./idl.js";
import { EventCoder } from "../index.js";
import BN from "bn.js";
import {
  CompiledInnerInstruction,
  PublicKey,
  Transaction,
  TransactionResponse,
  VersionedTransactionResponse,
} from "@solana/web3.js";

export class BorshEventCoder implements EventCoder {
  /**
   * CPI event discriminator.
   * https://github.com/coral-xyz/anchor/blob/v0.29.0/lang/src/event.rs
   */
  private static eventIxTag: BN = new BN("1d9acb512ea545e4", "hex");

  public address: PublicKey;

  /**
   * Maps account type identifier to a layout.
   */
  private layouts: Map<string, Layout>;

  /**
   * Maps base64 encoded event discriminator to event name.
   */
  private discriminators: Map<string, string>;

  public constructor(idl: Idl) {
    if (!idl.events) {
      this.layouts = new Map();
      return;
    }

    const types = idl.types;
    if (!types) {
      throw new Error("Events require `idl.types`");
    }

    const layouts: [string, Layout<any>][] = idl.events.map((ev) => {
      const typeDef = types.find((ty) => ty.name === ev.name);
      if (!typeDef) {
        throw new Error(`Event not found: ${ev.name}`);
      }
      return [ev.name, IdlCoder.typeDefLayout({ typeDef, types })];
    });
    this.layouts = new Map(layouts);

    this.discriminators = new Map<string, string>(
      (idl.events ?? []).map((ev) => [
        base64.encode(Buffer.from(ev.discriminator)),
        ev.name,
      ])
    );

    this.address = new PublicKey(idl.address);
  }

  /**
   *
   * @param log base 64 encoded log data from transaction message
   *            or base58 encoded transaction message or CPI encoded event data
   * @returns decoded event object or null
   */
  public decode(log: string): {
    name: string;
    data: any;
  } | null {
    const transactionCpiData = this.parseAsTransactionCpiData(log);
    if (transactionCpiData !== null) {
      // log parsed to be CPI data, recursive call stripped event data
      return this.decode(transactionCpiData);
    }

    let logArr: Buffer;
    // This will throw if log length is not a multiple of 4.
    try {
      logArr = base64.decode(log);
    } catch (e) {
      return null;
    }

    // Only deserialize if the discriminator implies a proper event.
    const disc = base64.encode(logArr.slice(0, 8));
    const eventName = this.discriminators.get(disc);
    if (!eventName) {
      return null;
    }

    const layout = this.layouts.get(eventName);
    if (!layout) {
      throw new Error(`Unknown event: ${eventName}`);
    }
    const data = layout.decode(logArr.slice(8));
    return { data, name: eventName };
  }

  /**
   * Check the log data to be transaction CPI event:
   * Expected data format:
   *  < cpi event discriminator | event name discriminator | event data >
   * If matches cpi event discriminator
   * < event name | event data> base64 formatted is returned
   * otherwise null is returned.
   */
  parseAsTransactionCpiData(log: string): string | null {
    let encodedLog: Buffer;
    try {
      // verification if log is transaction cpi data encoded with base58
      encodedLog = bs58.decode(log);
    } catch (e) {
      return null;
    }
    const disc = encodedLog.slice(0, 8);
    if (disc.equals(BorshEventCoder.eventIxTag.toBuffer("le"))) {
      // after CPI tag data follows in format of standard event
      return base64.encode(encodedLog.slice(8));
    } else {
      return null;
    }
  }

  public parseCpiEvents(
    transactionResponse: VersionedTransactionResponse | TransactionResponse
  ): { name: string; data: any }[] {
    const events: { name: string; data: any }[] = [];
    const inner: CompiledInnerInstruction[] =
      transactionResponse?.meta?.innerInstructions ?? [];
    const idlProgramId = this.address;
    for (let i = 0; i < inner.length; i++) {
      for (let j = 0; j < inner[i].instructions.length; j++) {
        const ix = inner[i].instructions[j];
        const programPubkey =
          transactionResponse?.transaction.message.staticAccountKeys[
            ix.programIdIndex
          ];
        if (
          programPubkey === undefined ||
          !programPubkey.equals(idlProgramId)
        ) {
          // we are at instructions that does not match the linked program
          continue;
        }
        const event = this.decode(ix.data);
        if (event) {
          events.push(event);
        }
      }
    }
    return events;
  }
}
