import { Idl } from "../../idl.js";
import { InstructionCoder } from "../index.js";

export class DexInstructionCoder implements InstructionCoder {
  constructor(idl: Idl) {}

  encode(ixName: string, ix: any): Buffer {
    throw new Error("TODO");
  }

  encodeState(ixName: string, ix: any): Buffer {
    throw new Error("TODO");
  }
}
