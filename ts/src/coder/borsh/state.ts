import { Buffer } from "buffer";
import { Layout } from "buffer-layout";
import { sha256 } from "js-sha256";
import { Idl } from "../../idl.js";
import { IdlCoder } from "./idl.js";
import * as features from "../../utils/features";
import { BorshAccountHeader } from "./accounts";

export class BorshStateCoder {
  private layout: Layout;
  readonly header: BorshAccountHeader;

  public constructor(idl: Idl) {
    if (idl.state === undefined) {
      throw new Error("Idl state not defined.");
    }
    this.layout = IdlCoder.typeDefLayout(idl.state.struct, idl.types);
    this.header = new BorshAccountHeader(idl);
  }

  public async encode<T = any>(name: string, account: T): Promise<Buffer> {
    const buffer = Buffer.alloc(1000); // TODO: use a tighter buffer.
    const len = this.layout.encode(account, buffer);

    let ns = features.isSet("anchor-deprecated-state") ? "account" : "state";
    const header = this.header.encode(name, ns);
    const accData = buffer.slice(0, len);

    return Buffer.concat([header, accData]);
  }

  public decode<T = any>(data: Buffer): T {
    // Chop off header.
    data = data.slice(BorshAccountHeader.size());
    return this.layout.decode(data);
  }
}

// Calculates unique 8 byte discriminator prepended to all anchor state accounts.
export async function stateDiscriminator(name: string): Promise<Buffer> {
  let ns = features.isSet("anchor-deprecated-state") ? "account" : "state";
  return Buffer.from(sha256.digest(`${ns}:${name}`)).slice(
    0,
    this.header.discriminatorSize()
  );
}
