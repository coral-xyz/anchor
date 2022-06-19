import camelCase from "camelcase";
import { Idl } from "src/idl";
import { InstructionCoder } from "..";
import { Schema, serialize } from "borsh";
import BN from "bn.js";

export class SerumInstructionCoder implements InstructionCoder {
  constructor(_: Idl) {}

  encode(ixName: string, ix: any): Buffer {
    switch (camelCase(ixName)) {
      case "createMarket": {
        return Buffer.from(new createMarketInstruction(ix).serialize());
      }
      case "newOrder": {
        return Buffer.from(new newOrderInstruction(ix).serialize());
      }
      case "swap": {
        return Buffer.from(new swapInstruction(ix).serialize());
      }
      case "cancelOrder": {
        return Buffer.from(new cancelOrderInstruction(ix).serialize());
      }
      case "consumeEvents": {
        return Buffer.from(new consumeEventsInstruction(ix).serialize());
      }
      case "settle": {
        return Buffer.from(new settleInstruction().serialize());
      }
      case "initializeAccount": {
        return Buffer.from(new initializeAccountInstruction(ix).serialize());
      }
      case "sweepFees": {
        return Buffer.from(new sweepFeesInstruction().serialize());
      }
      case "closeAccount": {
        return Buffer.from(new closeAccountInstruction().serialize());
      }
      case "closeMarket": {
        return Buffer.from(new closeMarketInstruction().serialize());
      }
      case "updateRoyalties": {
        return Buffer.from(new updateRoyaltiesInstruction().serialize());
      }
      default: {
        throw new Error(`Invalid instruction: ${ixName}`);
      }
    }
  }

  encodeState(_ixName: string, _ix: any): Buffer {
    throw new Error("SPL token does not have state");
  }
}

export class createMarketInstruction {
  tag: BN;
  signerNonce: BN;
  minBaseOrderSize: BN;
  tickSize: BN;
  crankerReward: BN;
  static schema: Schema = new Map([
    [
      createMarketInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u64"],
          ["signerNonce", "u64"],
          ["minBaseOrderSize", "u64"],
          ["tickSize", "u64"],
          ["crankerReward", "u64"],
        ],
      },
    ],
  ]);
  constructor(obj: {
    signerNonce: BN;
    minBaseOrderSize: BN;
    tickSize: BN;
    crankerReward: BN;
  }) {
    this.tag = new BN(0);
    this.signerNonce = obj.signerNonce;
    this.minBaseOrderSize = obj.minBaseOrderSize;
    this.tickSize = obj.tickSize;
    this.crankerReward = obj.crankerReward;
  }
  serialize(): Uint8Array {
    return serialize(createMarketInstruction.schema, this);
  }
}

export class newOrderInstruction {
  tag: BN;
  clientOrderId: BN;
  limitPrice: BN;
  maxBaseQty: BN;
  maxQuoteQty: BN;
  matchLimit: BN;
  side: number;
  orderType: number;
  selfTradeBehavior: number;
  hasDiscountTokenAccount: number;
  padding: Uint8Array;
  static schema: Schema = new Map([
    [
      newOrderInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u64"],
          ["clientOrderId", "u128"],
          ["limitPrice", "u64"],
          ["maxBaseQty", "u64"],
          ["maxQuoteQty", "u64"],
          ["matchLimit", "u64"],
          ["side", "u8"],
          ["orderType", "u8"],
          ["selfTradeBehavior", "u8"],
          ["hasDiscountTokenAccount", "u8"],
          ["padding", "u32"],
        ],
      },
    ],
  ]);
  constructor(obj: {
    clientOrderId: BN;
    limitPrice: BN;
    maxBaseQty: BN;
    maxQuoteQty: BN;
    matchLimit: BN;
    side: number;
    orderType: number;
    selfTradeBehavior: number;
    hasDiscountTokenAccount: number;
  }) {
    this.tag = new BN(1);
    this.clientOrderId = obj.clientOrderId;
    this.limitPrice = obj.limitPrice;
    this.maxBaseQty = obj.maxBaseQty;
    this.maxQuoteQty = obj.maxQuoteQty;
    this.matchLimit = obj.matchLimit;
    this.side = obj.side;
    this.orderType = obj.orderType;
    this.selfTradeBehavior = obj.selfTradeBehavior;
    this.hasDiscountTokenAccount = obj.hasDiscountTokenAccount;
    this.padding = new Uint8Array(4).fill(0);
  }
  serialize(): Uint8Array {
    return serialize(newOrderInstruction.schema, this);
  }
}

export class swapInstruction {
  tag: BN;
  baseQty: BN;
  quoteQty: BN;
  matchLimit: BN;
  side: number;
  hasDiscountTokenAccount: number;
  padding: Uint8Array;
  static schema: Schema = new Map([
    [
      swapInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u64"],
          ["baseQty", "u64"],
          ["quoteQty", "u64"],
          ["matchLimit", "u64"],
          ["side", "u8"],
          ["hasDiscountTokenAccount", "u8"],
          ["padding", [6]],
        ],
      },
    ],
  ]);
  constructor(obj: {
    baseQty: BN;
    quoteQty: BN;
    matchLimit: BN;
    side: number;
    hasDiscountTokenAccount: number;
  }) {
    this.tag = new BN(2);
    this.baseQty = obj.baseQty;
    this.quoteQty = obj.quoteQty;
    this.matchLimit = obj.matchLimit;
    this.side = obj.side;
    this.hasDiscountTokenAccount = obj.hasDiscountTokenAccount;
    this.padding = new Uint8Array(6).fill(0);
  }
  serialize(): Uint8Array {
    return serialize(swapInstruction.schema, this);
  }
}

export class cancelOrderInstruction {
  tag: BN;
  orderId: BN;
  orderIndex: BN;
  isClientId: number;
  padding: Uint8Array;
  static schema: Schema = new Map([
    [
      cancelOrderInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u64"],
          ["orderId", "u128"],
          ["orderIndex", "u64"],
          ["isClientId", "u8"],
          ["padding", [7]],
        ],
      },
    ],
  ]);
  constructor(obj: { orderId: BN; orderIndex: BN; isClientId: number }) {
    this.tag = new BN(3);
    this.orderId = obj.orderId;
    this.orderIndex = obj.orderIndex;
    this.isClientId = obj.isClientId;
    this.padding = new Uint8Array(7).fill(0);
  }
  serialize(): Uint8Array {
    return serialize(cancelOrderInstruction.schema, this);
  }
}

export class consumeEventsInstruction {
  tag: BN;
  maxIterations: BN;
  noOpErr: BN;
  static schema: Schema = new Map([
    [
      consumeEventsInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u64"],
          ["maxIterations", "u64"],
          ["noOpErr", "u64"],
        ],
      },
    ],
  ]);
  constructor(obj: { maxIterations: BN; noOpErr: BN }) {
    this.tag = new BN(4);
    this.maxIterations = obj.maxIterations;
    this.noOpErr = obj.noOpErr;
  }
  serialize(): Uint8Array {
    return serialize(consumeEventsInstruction.schema, this);
  }
}

export class settleInstruction {
  tag: BN;
  static schema: Schema = new Map([
    [
      settleInstruction,
      {
        kind: "struct",
        fields: [["tag", "u64"]],
      },
    ],
  ]);
  constructor() {
    this.tag = new BN(5);
  }
  serialize(): Uint8Array {
    return serialize(settleInstruction.schema, this);
  }
}

export class initializeAccountInstruction {
  tag: BN;
  market: Uint8Array;
  maxOrders: BN;
  static schema: Schema = new Map([
    [
      initializeAccountInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u64"],
          ["market", [32]],
          ["maxOrders", "u64"],
        ],
      },
    ],
  ]);
  constructor(obj: { market: Uint8Array; maxOrders: BN }) {
    this.tag = new BN(6);
    this.market = obj.market;
    this.maxOrders = obj.maxOrders;
  }
  serialize(): Uint8Array {
    return serialize(initializeAccountInstruction.schema, this);
  }
}

export class sweepFeesInstruction {
  tag: BN;
  static schema: Schema = new Map([
    [
      sweepFeesInstruction,
      {
        kind: "struct",
        fields: [["tag", "u64"]],
      },
    ],
  ]);
  constructor() {
    this.tag = new BN(7);
  }
  serialize(): Uint8Array {
    return serialize(sweepFeesInstruction.schema, this);
  }
}

export class closeAccountInstruction {
  tag: BN;
  static schema: Schema = new Map([
    [
      closeAccountInstruction,
      {
        kind: "struct",
        fields: [["tag", "u64"]],
      },
    ],
  ]);
  constructor() {
    this.tag = new BN(8);
  }
  serialize(): Uint8Array {
    return serialize(closeAccountInstruction.schema, this);
  }
}

export class closeMarketInstruction {
  tag: BN;
  static schema: Schema = new Map([
    [
      closeMarketInstruction,
      {
        kind: "struct",
        fields: [["tag", "u64"]],
      },
    ],
  ]);
  constructor() {
    this.tag = new BN(9);
  }
  serialize(): Uint8Array {
    return serialize(closeMarketInstruction.schema, this);
  }
}

export class updateRoyaltiesInstruction {
  tag: BN;
  static schema: Schema = new Map([
    [
      updateRoyaltiesInstruction,
      {
        kind: "struct",
        fields: [["tag", "u64"]],
      },
    ],
  ]);
  constructor() {
    this.tag = new BN(10);
  }
  serialize(): Uint8Array {
    return serialize(updateRoyaltiesInstruction.schema, this);
  }
}
