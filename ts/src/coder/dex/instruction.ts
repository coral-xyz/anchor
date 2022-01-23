import * as BufferLayout from "buffer-layout";
import camelCase from "camelcase";
import { PublicKey } from "@solana/web3.js";
import { InstructionCoder } from "../index.js";
import { Idl } from "../../idl.js";
import { publicKey } from "@project-serum/anchor/dist/cjs/utils";

export class SplTokenInstructionCoder implements InstructionCoder {
  constructor(_: Idl) {}

  encode(ixName: string, ix: any): Buffer {
    switch (camelCase(ixName)) {
      case "initialize_market": {
        return encodeInitializeMarket(ix);
      }
      case "new_order_v3": {
        return encodeNewOrderv3(ix);
      }
      case "match_orders": {
        return encodeMatchOrders(ix);
      }
      case "consume_events": {
        return encodeConsumeEvents(ix);
      }
      case "cancel_order": {
        return encodeCancelOrder(ix);
      }
      case "cancel_order_v2": {
        return encodeCancelOrderV2(ix);
      }
      case "settle_funds": {
        return encodeSettleFunds(ix);
      }
      case "cancel_order_by_client_v2": {
        return encodeCancelOrdersByClientv2(ix);
      }
      case "disable_market": {
        return encodeDisableMarket(ix);
      }
      case "sweep_fees": {
        return encodeSweepFees(ix);
      }
      case "send_take": {
        return encodeSendTake(ix);
      }
      case "close_open_orders": {
        return encodeCloseOpenOrders(ix);
      }
      case "init_open_orders": {
        return encodeInitOpenOrders(ix);
      }
      case "prune": {
        return encodePrune(ix);
      }
      case "consume_events_permissioned": {
        return encodeConsumeEventsPermissioned(ix);
      }
      default: {
        throw new Error(`Invalid instruction: ${ixName}`);
      }
    }
  }

  encodeState(_ixName: string, _ix: any): Buffer {
    throw new Error("It does not have state");
  }
}

function encodeInitializeMarket({
    coin_lot_size,
    pc_lot_size,
    vault_signer_nonce,
    pc_dust_threshold,
    fee_rate_bps,
    prune_authority,
    consume_events_authority,
    authority, }: any): Buffer {
  return encodeData({
    InitializeMarket: {
        coin_lot_size,
        pc_lot_size,
        vault_signer_nonce,
        pc_dust_threshold, 
        fee_rate_bps,
        prune_authority,
        consume_events_authority,
        authority,

    },
  });
}
function encodeNewOrder({
  side, 
  limit_price, 
  max_coin_qty,  
  order_type,
  client_order_id,
  self_trade_behaviour,
  open_orders_authority, 
  limit,
  max_native_pc_qty_including_fees}: any): Buffer {
return encodeData({
  NewOrder: {
    side, 
    limit_price, 
    max_coin_qty,  
    order_type,
    client_order_id,
    self_trade_behaviour,
    open_orders_authority, 
    limit},
});
}


function encodeNewOrderv3({
    side, 
    limit_price, 
    max_coin_qty,
    self_trade_behaviour,  
    order_type,
    client_order_id,
    open_orders_authority, 
    limit }: any): Buffer {
  return encodeData({
    NewOrderv3: {
      side, 
      limit_price, 
      max_coin_qty,  
      self_trade_behaviour,
      order_type,
      client_order_id,
      open_orders_authority, 
      limit},
  });
}

function encodeMatchOrders({ limit }: any): Buffer {
  return encodeData({
    MatchOrders: {
      limit,
    },
  });
}

function encodeConsumeEvents({ limit }: any): Buffer {
  return encodeData({
    ConsumeEvents: { limit },
  });
}

function encodeCancelOrder({ side, order_id, open_orders_authority }: any): Buffer {
  return encodeData({
    CancelOrders: { side, order_id, open_orders_authority },
  });
}

function encodeSettleFunds({open_orders_authority}: any): Buffer {
  return encodeData({
    SettleFunds: {open_orders_authority},
  });
}

function encodeDisableMarket( {disable_authority_key}: any): Buffer {
  return encodeData({
    DisableMarket: {disable_authority_key},
  });
}

function encodeSweepFees({sweep_authority}: any): Buffer {
  return encodeData({
    SweepFees: {sweep_authority},
  });
}

function encodeCancelOrderV2({ side, order_id, open_orders_authority }: any): Buffer {
  return encodeData({
    CancelOrderv2: { side, order_id , open_orders_authority},
  });
}

function encodeCancelOrdersByClientv2({client_id, open_orders_authority}: any): Buffer {
  return encodeData({
    CancelOrderByClientv2: {client_id, open_orders_authority},
  });
}

function encodeSendTake({side, 
  limit_price, 
  max_coin_qty, 
  max_native_pc_qty_including_fees, 
  min_coin_qty, 
  min_natuve_pc_qty, 
  limit}: any): Buffer {
  return encodeData({
    SendTake: {side, 
      limit_price, 
      max_coin_qty, 
      max_native_pc_qty_including_fees, 
      min_coin_qty, 
      min_natuve_pc_qty, 
      limit},
  });
}

function encodeCloseOpenOrders({open_orders_authority}: any): Buffer {
  return encodeData({
    CloseOpenOrders: {open_orders_authority},
  });
}

function encodeInitOpenOrders({open_orders_authority, market_authority}: any): Buffer {
  return encodeData({
    InitOpenOrders: {open_orders_authority, market_authority},
  });
}

function encodePrune({ limit, prune_authority }: any): Buffer {
  return encodeData({
    Prune: { limit, prune_authority },
  });
}

function encodeConsumeEventsPermissioned({ limit, consume_events_authority }: any): Buffer {
  return encodeData({
    ConsumeEventsPermissioned: { limit, consume_events_authority },
  });
}


const LAYOUT = BufferLayout.union(BufferLayout.u8("instruction"));
LAYOUT.addVariant(
  0,
  BufferLayout.struct([
    BufferLayout.u64("coin_lot_size"),
    BufferLayout.u64("pc_lot_size"),
    BufferLayout.u64("vault_signer_nonce"),
    BufferLayout.u64("pc_dust_threshold"),
    BufferLayout.u16("fee_rate_bps"),
    publicKey("prune_authority"),
    publicKey("consume_events_authority"),
    publicKey("authority"),

  ]),
  "initialize_market"
);
LAYOUT.addVariant(1, BufferLayout.struct([
  BufferLayout.u32("side"),
  BufferLayout.u64("limit_price"),
  BufferLayout.u64("max_coin_qty"),
  BufferLayout.u32("order_type"),
  BufferLayout.u64("client_order_id"),
  BufferLayout.u32("self_trade_behavior"),
  publicKey("open_orders_authority"),
  BufferLayout.u16("limit"),
  BufferLayout.u64("max__native_pc_qty_including_fees"),
]), "new_order");

LAYOUT.addVariant(
  2,
  BufferLayout.struct([
    BufferLayout.u32("side"),
    BufferLayout.u64("limit_price"),
    BufferLayout.u64("max_coin_qty"),
    BufferLayout.u32("self_trade_behavior"),
    BufferLayout.u32("order_type"),
    BufferLayout.u64("client_order_id"),
    publicKey("open_orders_authority"),
    BufferLayout.u16("limit"),
  ]),
  "new_order_v3"
);
LAYOUT.addVariant(
  3,
  BufferLayout.struct([BufferLayout.u16("limit")]),
  "match_order"
);
LAYOUT.addVariant(
  4,
  BufferLayout.struct([
    BufferLayout.u16("limit"),
]),
  "consume_events"
);
LAYOUT.addVariant(5, BufferLayout.struct([
  BufferLayout.u32("side"),
  BufferLayout.u128("order_id"),
  publicKey("open_orders_authority"),
]), "cancel_order");

LAYOUT.addVariant(
  6,
  BufferLayout.struct([
    publicKey("open_orders_authority"),
  ]),
  "settle_funds"
);

LAYOUT.addVariant(
  7,
  BufferLayout.struct([
    publicKey("disable_authority_key"),
  ]),
  "disable_markets"
);

LAYOUT.addVariant(
  8,
  BufferLayout.struct([
    publicKey("sweep_authority"),
  ]),
  "sweep_fees"
);

LAYOUT.addVariant(9, BufferLayout.struct([
  BufferLayout.u32("side"),
  BufferLayout.u128("order_id"),
  publicKey("open_orders_authority"),
]), "cancel_order_v2");

LAYOUT.addVariant(10, BufferLayout.struct([
  BufferLayout.u64("client_id"),
  publicKey("open_orders_authority"),
]), "cancel_order_by_client_v2");

LAYOUT.addVariant(11, BufferLayout.struct([
  BufferLayout.u32("side"),
  BufferLayout.u64("limit_price"),
  BufferLayout.u64("max_coin_qty"),
  BufferLayout.u64("min_native_pc_qty"),
  BufferLayout.u16("limit"),
]), "send_take");

LAYOUT.addVariant(
  13,
  BufferLayout.struct([
    publicKey("open_orders_authority"),
  ]),
  "close_open_orders"
);
LAYOUT.addVariant(
  14,
  BufferLayout.struct([
    publicKey("open_orders_authority"),
    publicKey("market_authority"),
  ]),
  "init_open_orders"
);

LAYOUT.addVariant(
  15,
  BufferLayout.struct([
    BufferLayout.u16("limit"),
    publicKey("prune_authority"),
  ]),
  "prune"
);

LAYOUT.addVariant(
  16,
  BufferLayout.struct([BufferLayout.u16("limit")]),
  publicKey("consume_events_authority"),
  "consume_events_permissioned"
);



function encodeData(instruction: any): Buffer {
  let b = Buffer.alloc(instructionMaxSpan);
  let span = LAYOUT.encode(instruction, b);
  return b.slice(0, span);
}
function publicKey(property: string): any {
  return BufferLayout.blob(32, property);
}


const instructionMaxSpan = Math.max(
  // @ts-ignore
  ...Object.values(LAYOUT.registry).map((r) => r.span)
);
