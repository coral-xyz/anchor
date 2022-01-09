import * as BufferLayout from "buffer-layout";
import { PublicKey } from "@solana/web3.js";
import { AccountsCoder } from "../index.js";
import { Idl } from "../../idl.js";

export class SplTokenAccountsCoder<A extends string = string>
  implements AccountsCoder {
  constructor(_: Idl) {}

  public async encode<T = any>(accountName: A, account: T): Promise<Buffer> {
    const buffer = Buffer.alloc(1000); // TODO: use a tighter buffer.
    switch (accountName) {
      case "Token": {
        const len = TOKEN_ACCOUNT_LAYOUT.encode(account, buffer);
        return buffer.slice(0, len);
      }
      case "Mint": {
        const len = MINT_ACCOUNT_LAYOUT.encode(account, buffer);
        return buffer.slice(0, len);
      }
      default: {
        throw new Error(`Invalid account name: ${accountName}`);
      }
    }
  }

  public decode<T = any>(accountName: A, ix: Buffer): T {
    return this.decodeUnchecked(accountName, ix);
  }

  public decodeUnchecked<T = any>(accountName: A, ix: Buffer): T {
    switch (accountName) {
      case "Token": {
        return decodeTokenAccount(ix);
      }
      case "Mint": {
        return decodeMintAccount(ix);
      }
      default: {
        throw new Error(`Invalid account name: ${accountName}`);
      }
    }
  }

  // TODO: this won't use the appendData.
  public memcmp(accountName: A, _appendData?: Buffer): any {
    switch (accountName) {
      case "Token": {
        return {
          dataSize: 165,
        };
      }
      case "Mint": {
        return {
          dataSize: 82,
        };
      }
      default: {
        throw new Error(`Invalid account name: ${accountName}`);
      }
    }
  }
}

// TODO: can probably clean this up by using a proper COption and PublicKey
//       struct layout decoder/subclass.
function decodeTokenAccount<T = any>(ix: Buffer): T {
  const account = TOKEN_ACCOUNT_LAYOUT.decode(ix) as T;
  // @ts-ignore
  account.authority = new PublicKey(account.authority);
  // @ts-ignore
  account.mint = new PublicKey(account.mint);
  // @ts-ignore
  account.delegate =
    // @ts-ignore
    account.delegateOption === 1 ? new PublicKey(account.delegate) : null;
  // @ts-ignore
  account.closeAuthority =
    // @ts-ignore
    account.closeAuthorityOption === 1
      ? // @ts-ignore
        new PublicKey(account.closeAuthority)
      : null;
  return account;
}

function decodeMintAccount<T = any>(ix: Buffer): T {
  const account = MINT_ACCOUNT_LAYOUT.decode(ix) as T;
  // @ts-ignore
  account.mintAuthority =
    // @ts-ignore
    account.mintAuthorityOption === 1
      ? // @ts-ignore
        new PublicKey(account.mintAuthority)
      : null;
  // @ts-ignore
  account.freezeAuthority =
    // @ts-ignore
    account.freezeAuthorityOption === 1
      ? // @ts-ignore
        new PublicKey(account.freezeAuthorityAuthority)
      : null;
  return account;
}

const TOKEN_ACCOUNT_LAYOUT = BufferLayout.struct([
  publicKey("mint"),
  publicKey("authority"),
  uint64("amount"),
  BufferLayout.u32("delegateOption"),
  publicKey("delegate"),
  BufferLayout.u8("state"),
  BufferLayout.u32("isNativeOption"),
  uint64("isNative"),
  uint64("delegatedAmount"),
  BufferLayout.u32("closeAuthorityOption"),
  publicKey("closeAuthority"),
]);

const MINT_ACCOUNT_LAYOUT = BufferLayout.struct([
  BufferLayout.u32("mintAuthorityOption"),
  publicKey("mintAuthority"),
  uint64("supply"),
  BufferLayout.u8("decimals"),
  BufferLayout.u8("isInitialized"),
  BufferLayout.u32("freezeAuthorityOption"),
  publicKey("freezeAuthority"),
]);

function publicKey(property: string): any {
  return BufferLayout.blob(32, property);
}

function uint64(property: string): any {
  return BufferLayout.blob(8, property);
}
