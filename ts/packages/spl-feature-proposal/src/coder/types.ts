import { Idl, TypesCoder } from "@project-serum/anchor";

export class SplFeatureProposalTypesCoder implements TypesCoder {
  constructor(_idl: Idl) {}

  encode<T = any>(_name: string, _type: T): Buffer {
    throw new Error("SplFeatureProposal does not have user-defined types");
  }
  decode<T = any>(_name: string, _typeData: Buffer): T {
    throw new Error("SplFeatureProposal does not have user-defined types");
  }
}
