import { Idl } from "../../idl.js";
import { Coder } from "..";
import { SerumAccountsCoder } from "./accounts";
import { SerumEventCoder } from "./events";
import { SerumInstructionCoder } from "./instruction";
import { SerumStateCoder } from "./state";
import { SerumTypesCoder } from "./types.js";

export class SerumCoder implements Coder {
  readonly instruction: SerumInstructionCoder;
  readonly accounts: SerumAccountsCoder;
  readonly state: SerumStateCoder;
  readonly events: SerumEventCoder;
  readonly types: SerumTypesCoder;

  constructor(idl: Idl) {
    this.instruction = new SerumInstructionCoder(idl);
    this.accounts = new SerumAccountsCoder(idl);
    this.events = new SerumEventCoder(idl);
    this.state = new SerumStateCoder(idl);
    this.types = new SerumTypesCoder(idl);
  }
}
