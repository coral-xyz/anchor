import { Idl, TypesCoder } from "@coral-xyz/anchor";

export class SplGovernanceTypesCoder implements TypesCoder {
  constructor(_idl: Idl) {}

  encode<T = any>(_name: string, _type: T): Buffer {
    throw new Error("SplGovernance does not have user-defined types");
  }
  decode<T = any>(_name: string, _typeData: Buffer): T {
    throw new Error("SplGovernance does not have user-defined types");
  }
}
