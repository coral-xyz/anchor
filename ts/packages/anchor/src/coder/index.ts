import { VersionedTransactionResponse } from "@solana/web3.js";
import { IdlEvent } from "../idl.js";
import { Event } from "../program/event.js";

export * from "./borsh/index.js";
export * from "./system/index.js";

/**
 * Coder provides a facade for encoding and decoding all IDL related objects.
 */
export interface Coder<A extends string = string, T extends string = string> {
  /**
   * Instruction coder.
   */
  readonly instruction: InstructionCoder;

  /**
   * Account coder.
   */
  readonly accounts: AccountsCoder<A>;

  /**
   * Coder for events.
   */
  readonly events: EventCoder;

  /**
   * Coder for user-defined types.
   */
  readonly types: TypesCoder<T>;
}

export interface StateCoder {
  encode<T = any>(name: string, account: T): Promise<Buffer>;
  decode<T = any>(ix: Buffer): T;
}

export interface AccountsCoder<A extends string = string> {
  encode<T = any>(accountName: A, account: T): Promise<Buffer>;
  decode<T = any>(accountName: A, acc: Buffer): T;
  decodeUnchecked<T = any>(accountName: A, acc: Buffer): T;
  memcmp(accountName: A, appendData?: Buffer): any;
  size(accountName: A): number;
}

export interface InstructionCoder {
  encode(ixName: string, ix: any): Buffer;
}

export interface EventCoder {
  decode<E extends IdlEvent = IdlEvent, T = Record<string, string>>(
    log: string
  ): Event<E, T> | null;

  parseCpiEvents<E extends IdlEvent = IdlEvent, T = Record<string, string>>(
    transactionResponse: VersionedTransactionResponse
  ): Event<E, T>[];
}

export abstract class NoEventCoder implements EventCoder {
  decode<E extends IdlEvent = IdlEvent, T = Record<string, string>>(
    _log: string
  ): Event<E, T> | null {
    throw new Error(`${this.constructor} program does not have events`);
  }

  parseCpiEvents<E extends IdlEvent = IdlEvent, T = Record<string, string>>(
    _transactionResponse: VersionedTransactionResponse
  ): Event<E, T>[] {
    throw new Error(`${this.constructor} program does not have CPI events`);
  }
}

export interface TypesCoder<N extends string = string> {
  encode<T = any>(typeName: N, type: T): Buffer;
  decode<T = any>(typeName: N, typeData: Buffer): T;
}
