import { Buffer } from "buffer";
import * as base64 from "base64-js";
import { Layout } from "buffer-layout";
import { sha256 } from "js-sha256";
import { Idl, IdlEvent, IdlTypeDef } from "../../idl.js";
import { Event, EventData } from "../../program/event.js";
import { IdlCoder } from "./idl.js";
import { EventCoder } from "../index.js";

export class BorshEventCoder implements EventCoder {
  /**
   * Maps account type identifier to a layout.
   */
  private layouts: Map<string, Layout>;

  /**
   * Maps base64 encoded event discriminator to event name.
   */
  private discriminators: Map<string, string>;

  /**
   * Header configuration.
   */
  private header: EventHeader;

  public constructor(idl: Idl) {
    if (idl.events === undefined) {
      this.layouts = new Map();
      return;
    }
    this.header = new EventHeader(idl);
    const layouts: [string, Layout<any>][] = idl.events.map((event) => {
      let eventTypeDef: IdlTypeDef = {
        name: event.name,
        type: {
          kind: "struct",
          fields: event.fields.map((f) => {
            return { name: f.name, type: f.type };
          }),
        },
      };
      return [event.name, IdlCoder.typeDefLayout(eventTypeDef, idl.types)];
    });
    this.layouts = new Map(layouts);

    this.discriminators = new Map<string, string>(
      idl.events === undefined
        ? []
        : idl.events.map((e) => [
            base64.fromByteArray(this.header.discriminator(e.name)),
            e.name,
          ])
    );
  }

  public decode<E extends IdlEvent = IdlEvent, T = Record<string, never>>(
    log: string
  ): Event<E, T> | null {
    let logArr: Buffer;
    // This will throw if log length is not a multiple of 4.
    try {
      logArr = Buffer.from(base64.toByteArray(log));
    } catch (e) {
      return null;
    }
    const disc = base64.fromByteArray(this.header.parseDiscriminator(logArr));

    // Only deserialize if the discriminator implies a proper event.
    const eventName = this.discriminators.get(disc);
    if (eventName === undefined) {
      return null;
    }

    const layout = this.layouts.get(eventName);
    if (!layout) {
      throw new Error(`Unknown event: ${eventName}`);
    }
    const data = layout.decode(logArr.slice(this.header.size())) as EventData<
      E["fields"][number],
      T
    >;
    return { data, name: eventName };
  }
}

class EventHeader {
  constructor(private _idl: Idl) {}

  public parseDiscriminator(data: Buffer): Buffer {
    if (this._idl.layoutVersion === undefined) {
      return data.slice(0, 8);
    } else {
      return data.slice(0, 4);
    }
  }

  public size(): number {
    if (this._idl.layoutVersion === undefined) {
      return 8;
    } else {
      return 4;
    }
  }

  public discriminator(name: string): Buffer {
    if (this._idl.layoutVersion === undefined) {
      return Buffer.from(sha256.digest(`event:${name}`)).slice(0, 8);
    } else {
      return Buffer.from(sha256.digest(`event:${name}`)).slice(0, 4);
    }
  }
}
