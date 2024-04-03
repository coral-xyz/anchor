import { Buffer } from "buffer";
import { Layout } from "buffer-layout";
import * as base64 from "../../utils/bytes/base64.js";
import { Idl } from "../../idl.js";
import { IdlCoder } from "./idl.js";
import { EventCoder } from "../index.js";
import BN from "bn.js";
import { PublicKey } from "@solana/web3.js";
import { decode as bs58Decode } from "../../utils/bytes/bs58.js";

export class BorshEventCoder implements EventCoder {
  /**
   * CPI event discriminator.
   * https://github.com/coral-xyz/anchor/blob/v0.29.0/lang/src/event.rs
   */
  private static eventIxTag: BN = new BN("1d9acb512ea545e4", "hex");

  private address: string;

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

    this.address = idl.address;
  }

  public decode(log: string): {
    name: string;
    data: any;
  } | null {
    let logArr: Buffer;
    // This will throw if log length is not a multiple of 4.
    try {
      logArr = base64.decode(log);
    } catch (e) {
      return null;
    }
    const disc = base64.encode(logArr.slice(0, 8));

    // Only deserialize if the discriminator implies a proper event.
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

  get idlAddress(): PublicKey {
    return new PublicKey(this.address);
  }

  public static isCPIEventData(buffer: Buffer): boolean {
    return buffer
      .subarray(0, 8)
      .equals(BorshEventCoder.eventIxTag.toBuffer("le"));
  }

  public decodeCpi(ixInputData: string): {
    name: string;
    data: any;
  } | null {
    const ixInputBufferData = bs58Decode(ixInputData);
    if (BorshEventCoder.isCPIEventData(ixInputBufferData)) {
      const eventData = base64.encode(ixInputBufferData.subarray(8));
      return this.decode(eventData);
    }
    return null;
  }
}
