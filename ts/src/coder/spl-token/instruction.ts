import { InstructionCoder } from "../index.js";
import { Idl } from "../../idl.js";

export class SplTokenInstructionCoder implements InstructionCoder {
  constructor(idl: Idl) {
    // todo
  }

  encode(ixName: string, ix: any): Buffer {
    // todo
    return Buffer.from([]);
  }

  encodeState(_ixName: string, _ix: any): Buffer {
    throw new Error("SPL token does not have state");
  }
}
