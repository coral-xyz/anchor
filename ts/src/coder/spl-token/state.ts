import { StateCoder } from "../index.js";

export class SplTokenStateCoder implements StateCoder {
  encode<T = any>(_name: string, _account: T): Promise<Buffer> {
    throw new Error("SPL token does not have state");
  }
  decode<T = any>(_ix: Buffer): T {
    throw new Error("SPL token does not have state");
  }
}
