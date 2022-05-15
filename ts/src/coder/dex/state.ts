import { Idl } from "../../idl.js";

export class DexStateCoder {
  constructor(_idl: Idl) {}

  encode<T = any>(_name: string, _account: T): Promise<Buffer> {
    throw new Error("Dex has no state");
  }

  decode<T = any>(_ix: Buffer): T {
    throw new Error("Dex has no state");
  }
}
