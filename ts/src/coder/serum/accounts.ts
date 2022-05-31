import { AccountsCoder } from "../index.js";
import { Idl, IdlTypeDef } from "../../idl.js";
import BN from "bn.js";
import { PublicKey } from "@solana/web3.js";
import { deserialize, Schema, serialize } from "borsh";
import { accountSize } from "../common.js";

export class SerumAccountsCoder<A extends string = string>
  implements AccountsCoder
{
  constructor(private idl: Idl) {}

  public async encode(accountName: A, account: any): Promise<Buffer> {
    switch (accountName) {
      case "marketState": {
        return Buffer.from(new MarketState(account).serialize());
      }
      default: {
        throw new Error(`Invalid account name: ${accountName}`);
      }
    }
  }

  public decode<T = any>(accountName: A, ix: Buffer): T {
    return this.decodeUnchecked(accountName, ix);
  }

  public decodeUnchecked<T = any>(accountName: A, data: Buffer): T {
    switch (accountName) {
      case "marketState": {
        return MarketState.deserialize(data) as unknown as T;
      }
      default: {
        throw new Error(`Invalid account name: ${accountName}`);
      }
    }
  }

  // TODO: this won't use the appendData.
  public memcmp(accountName: A, _appendData?: Buffer): any {
    switch (accountName) {
      case "marketState": {
        return {
          dataSize: MarketState.SIZE,
        };
      }
      default: {
        throw new Error(`Invalid account name: ${accountName}`);
      }
    }
  }

  public size(idlAccount: IdlTypeDef): number {
    return accountSize(this.idl, idlAccount) ?? 0;
  }
}

enum AccountTag {
  Initialized = 0,
  MarketState = 1,
  UserAccount = 2,
}

export class MarketState {
  tag: AccountTag;
  baseMint: PublicKey;
  quoteMint: PublicKey;
  baseVault: PublicKey;
  quoteVault: PublicKey;
  orderbook: PublicKey;
  admin: PublicKey;
  creationTimestamp: BN;
  baseVolume: BN;
  quoteVolume: BN;
  accumulatedFees: BN;
  minBaseOrderSize: BN;
  signerNonce: number;
  feeType: number;

  static SIZE = 264;

  static schema: Schema = new Map([
    [
      MarketState,
      {
        kind: "struct",
        fields: [
          ["tag", "u64"],
          ["baseMint", [32]],
          ["quoteMint", [32]],
          ["baseVault", [32]],
          ["quoteVault", [32]],
          ["orderbook", [32]],
          ["admin", [32]],
          ["creationTimestamp", "u64"],
          ["baseVolume", "u64"],
          ["quoteVolume", "u64"],
          ["accumulatedFees", "u64"],
          ["minBaseOrderSize", "u64"],
          ["signerNonce", "u8"],
          ["feeType", "u8"],
          ["padding", [6]],
        ],
      },
    ],
  ]);

  constructor(obj: {
    tag: BN;
    signerNonce: number;
    baseMint: Uint8Array;
    quoteMint: Uint8Array;
    baseVault: Uint8Array;
    quoteVault: Uint8Array;
    orderbook: Uint8Array;
    admin: Uint8Array;
    creationTimestamp: BN;
    baseVolume: BN;
    quoteVolume: BN;
    accumulatedFees: BN;
    minBaseOrderSize: BN;
    feeType: number;
  }) {
    this.tag = obj.tag.toNumber() as AccountTag;
    this.signerNonce = obj.signerNonce;
    this.baseMint = new PublicKey(obj.baseMint);
    this.quoteMint = new PublicKey(obj.quoteMint);
    this.baseVault = new PublicKey(obj.baseVault);
    this.quoteVault = new PublicKey(obj.quoteVault);
    this.orderbook = new PublicKey(obj.orderbook);
    this.admin = new PublicKey(obj.admin);
    this.creationTimestamp = obj.creationTimestamp;
    this.baseVolume = obj.baseVolume;
    this.quoteVolume = obj.quoteVolume;
    this.accumulatedFees = obj.accumulatedFees;
    this.minBaseOrderSize = obj.minBaseOrderSize;
    this.feeType = obj.feeType;
  }

  serialize(): Uint8Array {
    return serialize(MarketState.schema, this);
  }

  public static deserialize(buffer: Buffer): MarketState {
    return deserialize(MarketState.schema, MarketState, buffer) as MarketState;
  }
}
