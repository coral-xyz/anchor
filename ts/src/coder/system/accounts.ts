import { AccountsCoder } from "../index.js";
import { Idl, IdlTypeDef } from "../../idl.js";

export class SystemAccountsCoder<A extends string = string>
  implements AccountsCoder
{
  constructor(private idl: Idl) {}

  public async encode<T = any>(accountName: A, account: T): Promise<Buffer> {
    throw new Error("System program does not have accounts");
  }

  public decode<T = any>(accountName: A, ix: Buffer): T {
    throw new Error("System program does not have accounts");
  }

  public decodeUnchecked<T = any>(accountName: A, ix: Buffer): T {
    throw new Error("System program does not have accounts");
  }

  // TODO: this won't use the appendData.
  public memcmp(accountName: A, _appendData?: Buffer): any {
    throw new Error("System program does not have accounts");
  }

  public size(idlAccount: IdlTypeDef): number {
    throw new Error("System program does not have accounts");
  }
}
