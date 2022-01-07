import { AccountsCoder } from "../index.js";
import { Idl } from "../../idl.js";

export class SplTokenAccountsCoder<A extends string = string>
  implements AccountsCoder {
  constructor(idl: Idl) {
    // todo
  }

  encode<T = any>(accountName: A, account: T): Promise<Buffer> {
    // todo
    // @ts-ignore
    return Buffer.from([]);
  }

  decode<T = any>(accountName: A, ix: Buffer): T {
    // todo
    // @ts-ignore
    return null;
  }

  decodeUnchecked<T = any>(accountName: A, ix: Buffer): T {
    // todo
    // @ts-ignore
    return null;
  }

  memcmp(accountName: A, appendData?: Buffer): any {
    // todo
    return null;
  }
}
