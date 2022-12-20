import { Idl, TypesCoder } from "@coral-xyz/anchor";

export class SplTokenSwapTypesCoder implements TypesCoder {
  constructor(_idl: Idl) {}

  encode<T = any>(_name: string, _type: T): Buffer {
    throw new Error("SplTokenSwap does not have user-defined types");
  }
  decode<T = any>(_name: string, _typeData: Buffer): T {
    throw new Error("SplTokenSwap does not have user-defined types");
  }
}
