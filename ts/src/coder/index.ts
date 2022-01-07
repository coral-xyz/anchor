import { Idl, IdlEvent } from "../idl.js";
import { BorshInstructionCoder } from "./instruction.js";
import { BorshAccountsCoder } from "./accounts.js";
import { BorshEventCoder } from "./event.js";
import { BorshStateCoder } from "./state.js";
import { sighash } from "./common.js";
import { Event } from "../program/event.js";

export { accountSize } from "./common.js";
export { BorshInstructionCoder } from "./instruction.js";
export { BorshAccountsCoder, ACCOUNT_DISCRIMINATOR_SIZE } from "./accounts.js";
export { BorshEventCoder, eventDiscriminator } from "./event.js";
export { BorshStateCoder, stateDiscriminator } from "./state.js";

/**
 * Coder provides a facade for encoding and decoding all IDL related objects.
 */
export interface Coder {
  /**
   * Instruction coder.
   */
  readonly instruction: InstructionCoder;

  /**
   * Account coder.
   */
  readonly accounts: AccountsCoder;

  /**
   * Coder for state structs.
   */
  readonly state: StateCoder;

  /**
   * Coder for events.
   */
  readonly events: EventCoder;
}

export interface StateCoder {
  encode<T = any>(name: string, account: T): Promise<Buffer>;
  decode<T = any>(ix: Buffer): T;
}

export interface AccountsCoder<A extends string = string> {
  encode<T = any>(accountName: A, account: T): Promise<Buffer>;
  decode<T = any>(accountName: A, ix: Buffer): T;
  decodeUnchecked<T = any>(accountName: A, ix: Buffer): T;
  memcmp(accountName: A, appendData?: Buffer): any;
}

export interface InstructionCoder {
  encode(ixName: string, ix: any): Buffer;
  encodeState(ixName: string, ix: any): Buffer;
}

export interface EventCoder {
  decode<E extends IdlEvent = IdlEvent, T = Record<string, string>>(
    log: string
  ): Event<E, T> | null;
}

/**
 * BorshCoder is the default Coder for Anchor programs.
 */
export class BorshCoder<A extends string = string> implements Coder {
  /**
   * Instruction coder.
   */
  readonly instruction: BorshInstructionCoder;

  /**
   * Account coder.
   */
  readonly accounts: BorshAccountsCoder<A>;

  /**
   * Coder for state structs.
   */
  readonly state: BorshStateCoder;

  /**
   * Coder for events.
   */
  readonly events: BorshEventCoder;

  constructor(idl: Idl) {
    this.instruction = new BorshInstructionCoder(idl);
    this.accounts = new BorshAccountsCoder(idl);
    this.events = new BorshEventCoder(idl);
    if (idl.state) {
      this.state = new BorshStateCoder(idl);
    }
  }

  public sighash(nameSpace: string, ixName: string): Buffer {
    return sighash(nameSpace, ixName);
  }
}
