import { Idl, IdlTypeDef } from "../../idl.js";
import { AccountsCoder } from "../index.js";

export class DexAccountsCoder<A extends string = string>
  implements AccountsCoder {
  constructor(idl: Idl) {}

  encode<T = any>(accountName: A, account: T): Promise<Buffer> {
    throw new Error("TODO");
  }

  decode<T = any>(accountName: A, ix: Buffer): T {
    throw new Error("TODO");
  }

  decodeUnchecked<T = any>(accountName: A, ix: Buffer): T {
    throw new Error("TODO");
  }

  memcmp(accountName: A, appendData?: Buffer): any {
    throw new Error("TODO");
  }

  size(idlAccount: IdlTypeDef): number {
    throw new Error("TODO");
  }
}
