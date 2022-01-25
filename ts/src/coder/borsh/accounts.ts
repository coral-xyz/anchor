import bs58 from "bs58";
import { Buffer } from "buffer";
import { Layout } from "buffer-layout";
import camelcase from "camelcase";
import { sha256 } from "js-sha256";
import { Idl, IdlTypeDef } from "../../idl.js";
import { IdlCoder } from "./idl.js";
import { AccountsCoder } from "../index.js";
import { accountSize } from "../common.js";
import * as features from "../../utils/features";

/**
 * Number of bytes of the account header.
 */
const ACCOUNT_HEADER_SIZE = 8;

/**
 * Number of bytes of the account discriminator.
 */
const ACCOUNT_DISCRIMINATOR_SIZE = 4;
const DEPRECATED_ACCOUNT_DISCRIMINATOR_SIZE = 4;

/**
 * Encodes and decodes account objects.
 */
export class BorshAccountsCoder<A extends string = string>
  implements AccountsCoder {
  /**
   * Maps account type identifier to a layout.
   */
  private accountLayouts: Map<A, Layout>;

  /**
   * IDL whose acconts will be coded.
   */
  private idl: Idl;

  public constructor(idl: Idl) {
    if (idl.accounts === undefined) {
      this.accountLayouts = new Map();
      return;
    }
    const layouts: [A, Layout][] = idl.accounts.map((acc) => {
      return [acc.name as A, IdlCoder.typeDefLayout(acc, idl.types)];
    });

    this.accountLayouts = new Map(layouts);
    this.idl = idl;
  }

  public async encode<T = any>(accountName: A, account: T): Promise<Buffer> {
    const buffer = Buffer.alloc(1000); // TODO: use a tighter buffer.
    const layout = this.accountLayouts.get(accountName);
    if (!layout) {
      throw new Error(`Unknown account: ${accountName}`);
    }
    const len = layout.encode(account, buffer);
    let accountData = buffer.slice(0, len);
    let header = BorshAccountHeader.encode(accountName);
    return Buffer.concat([header, accountData]);
  }

  public decode<T = any>(accountName: A, data: Buffer): T {
    const expectedDiscriminator = BorshAccountHeader.discriminator(accountName);
		const givenDisc = BorshAccountHeader.parseDiscriminator(data);
    if (expectedDiscriminator.compare(givenDisc)) {
      throw new Error("Invalid account discriminator");
    }
    return this.decodeUnchecked(accountName, data);
  }

  public decodeUnchecked<T = any>(accountName: A, ix: Buffer): T {
    const data = ix.slice(BorshAccountHeader.size());   // Chop off the header.
    const layout = this.accountLayouts.get(accountName);
    if (!layout) {
      throw new Error(`Unknown account: ${accountName}`);
    }
    return layout.decode(data);
  }

  public memcmp(accountName: A, appendData?: Buffer): any {
    const discriminator = BorshAccountHeader.discriminator(accountName);
    return {
      offset: BorshAccountHeader.discriminatorOffset(),
      bytes: bs58.encode(
        appendData ? Buffer.concat([discriminator, appendData]) : discriminator
      ),
    };
  }

  public size(idlAccount: IdlTypeDef): number {
    return (
      BorshAccountHeader.size() + (accountSize(this.idl, idlAccount) ?? 0)
    );
  }
}

export class BorshAccountHeader {
	/**
	 * Returns the default account header for an account with the given name.
	 */
	public static encode(accountName: string): Buffer {
		if (features.isSet('deprecated-layout')) {
			return BorshAccountHeader.discriminator(accountName);
		} else {
			return Buffer.concat([
				Buffer.from([0]), // Version.
				Buffer.from([0]), // Bump.
				BorshAccountHeader.discriminator(accountName), // Disc.
				Buffer.from([0, 0]), // Unused.
			]);
		}
	}

  /**
   * Calculates and returns a unique 8 byte discriminator prepended to all anchor accounts.
   *
   * @param name The name of the account to calculate the discriminator.
   */
  public static discriminator(name: string): Buffer {
		let size: number;
		if (features.isSet("deprecated-layout")) {
			size = DEPRECATED_ACCOUNT_DISCRIMINATOR_SIZE;
		} else {
			size = ACCOUNT_DISCRIMINATOR_SIZE;
		}
    return Buffer.from(
      sha256.digest(`account:${camelcase(name, { pascalCase: true })}`)
    ).slice(0, size);
  }

	/**
	 * Returns the account data index at which the discriminator starts.
	 */
	public static discriminatorOffset(): number {
		if (features.isSet("deprecated-layout")) {
			return 0;
		} else {
			return 2;
		}
	}

	/**
	 * Returns the byte size of the account header.
	 */
	public static size(): number {
		return ACCOUNT_HEADER_SIZE;
	}

	/**
	 * Returns the discriminator from the given account data.
	 */
	public static parseDiscriminator(data: Buffer): Buffer {
		if (features.isSet("deprecated-layout")) {
			return data.slice(0, 8);
		} else {
			return data.slice(2, 6);
		}
	}
}
