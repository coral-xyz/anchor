import { Idl } from "../../idl.js";
import { Coder } from "../index.js";
import { DexInstructionCoder } from "./instruction.js";
import { DexStateCoder } from "./state.js";
import { DexAccountsCoder } from "./accounts.js";
import { DexEventsCoder } from "./events.js";

export class DexCoder implements Coder {
  readonly instruction: DexInstructionCoder;
  readonly accounts: DexAccountsCoder;
  readonly state: DexStateCoder;
  readonly events: DexEventsCoder;

  constructor(idl: Idl) {
    this.instruction = new DexInstructionCoder(idl);
    this.accounts = new DexAccountsCoder(idl);
    this.events = new DexEventsCoder(idl);
    this.state = new DexStateCoder(idl);
  }
}
