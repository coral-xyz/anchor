import { PublicKey } from "@solana/web3.js";
import { Program } from "../program/index.js";
import Provider from "../provider.js";
import { DexCoder } from "../coder/dex/index.js";

const DEX_PROGRAM_ID = new PublicKey(
  // TODO
  "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin"
);

export function program(provider?: Provider): Program<Dex> {
  return new Program<Dex>(IDL, DEX_PROGRAM_ID, provider, new DexCoder(IDL));
}

// TODO
export type Dex = {
  "version": "0.1.0",
  "name": "dex",
  "instructions": [
    {
      "name": "initializeMarket",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "pcMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "coinVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "requestQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pruneAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "consumeEventsAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "coinLotSize",
          "type": "u64"
        },
        {
          "name": "pcLotSize",
          "type": "u64"
        },
        {
          "name": "vaultSignerNonce",
          "type": "u64"
        },
        {
          "name": "pcDustThreshold",
          "type": "u64"
        },
        {
          "name": "feeRateBps",
          "type": "u16"
        }
      ]
    },
    {
      "name": "newOrder",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "requestQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "orderPayerTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "coinVault",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "pcVault",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "srmAccountReferral",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": {
            "defined": "Side"
          }
        },
        {
          "name": "limitPrice",
          "type": "u64"
        },
        {
          "name": "maxCoinQty",
          "type": "u64"
        },
        {
          "name": "orderType",
          "type": {
            "defined": "OrderType"
          }
        },
        {
          "name": "clientOrderId",
          "type": "u64"
        },
        {
          "name": "selfTradeBehaviour",
          "type": {
            "defined": "SelfTradeBehavior"
          }
        },
        {
          "name": "limit",
          "type": "u16"
        },
        {
          "name": "maxNativePcQtyIncludingFees",
          "type": "u64"
        }
      ]
    },
    {
      "name": "newOrderV3",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "requestQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "orderPayerTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "coinVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": {
            "defined": "Side"
          }
        },
        {
          "name": "limitPrice",
          "type": "u64"
        },
        {
          "name": "maxCoinQty",
          "type": "u64"
        },
        {
          "name": "orderType",
          "type": {
            "defined": "OrderType"
          }
        },
        {
          "name": "clientOrderId",
          "type": "u64"
        },
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    },
    {
      "name": "matchOrders",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "reqQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinFeeReceivableAccount",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "pcFeeReceivableAccount",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    },
    {
      "name": "consumeEvents",
      "accounts": [
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinFeeReceivableAccount",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "pcFeeReceivableAccount",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    },
    {
      "name": "cancelOrder",
      "accounts": [
        {
          "name": "market",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "reqQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": {
            "defined": "Side"
          }
        },
        {
          "name": "orderId",
          "type": "u128"
        }
      ]
    },
    {
      "name": "settleFunds",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "coinWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "vaultSigner",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "disableMarket",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "disableAuthorityKey",
          "isMut": false,
          "isSigner": true
        }
      ],
      "args": []
    },
    {
      "name": "sweepFees",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "sweepAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "sweepReceiverAmount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "vaultSigner",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "cancelOrderV2",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": {
            "defined": "Side"
          }
        },
        {
          "name": "orderId",
          "type": "u128"
        }
      ]
    },
    {
      "name": "cancelOrderByClientV2",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "clientId",
          "type": "u64"
        }
      ]
    },
    {
      "name": "sendTake",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "reqQueue",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": {
            "defined": "Side"
          }
        },
        {
          "name": "limitPrice",
          "type": "u64"
        },
        {
          "name": "maxCoinQty",
          "type": "u64"
        },
        {
          "name": "maxNativePcQtyIncludingFees",
          "type": "u64"
        },
        {
          "name": "minCoinQty",
          "type": "u64"
        },
        {
          "name": "minNatuvePcQty",
          "type": "u64"
        },
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    },
    {
      "name": "closeOpenOrders",
      "accounts": [
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "destination",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "initOpenOrders",
      "accounts": [
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "marketAuthority",
          "isMut": false,
          "isSigner": true
        }
      ],
      "args": []
    },
    {
      "name": "prune",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrders",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "pruneAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "openOrdersAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    },
    {
      "name": "consumeEventsPermissioned",
      "accounts": [
        {
          "name": "openOrders",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "consumeEventsAuthority",
          "isMut": false,
          "isSigner": true
        }
      ],
      "args": [
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    }
  ],
  "types": [
    {
      "name": "Side",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Bid"
          },
          {
            "name": "Ask"
          }
        ]
      }
    },
    {
      "name": "OrderType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Limit"
          },
          {
            "name": "ImmediateOrCancel"
          },
          {
            "name": "PostOnly"
          }
        ]
      }
    },
    {
      "name": "SelfTradeBehavior",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "DecrementTake"
          },
          {
            "name": "CancelProvide"
          },
          {
            "name": "AbortTransaction"
          }
        ]
      }
    }
  ]
}

// TODO
export const IDL: Dex = {
  "version": "0.1.0",
  "name": "dex",
  "instructions": [
    {
      "name": "initializeMarket",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "pcMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "coinVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "requestQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pruneAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "consumeEventsAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "coinLotSize",
          "type": "u64"
        },
        {
          "name": "pcLotSize",
          "type": "u64"
        },
        {
          "name": "vaultSignerNonce",
          "type": "u64"
        },
        {
          "name": "pcDustThreshold",
          "type": "u64"
        },
        {
          "name": "feeRateBps",
          "type": "u16"
        }
      ]
    },
    {
      "name": "newOrder",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "requestQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "orderPayerTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "coinVault",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "pcVault",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "srmAccountReferral",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": {
            "defined": "Side"
          }
        },
        {
          "name": "limitPrice",
          "type": "u64"
        },
        {
          "name": "maxCoinQty",
          "type": "u64"
        },
        {
          "name": "orderType",
          "type": {
            "defined": "OrderType"
          }
        },
        {
          "name": "clientOrderId",
          "type": "u64"
        },
        {
          "name": "selfTradeBehaviour",
          "type": {
            "defined": "SelfTradeBehavior"
          }
        },
        {
          "name": "limit",
          "type": "u16"
        },
        {
          "name": "maxNativePcQtyIncludingFees",
          "type": "u64"
        }
      ]
    },
    {
      "name": "newOrderV3",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "requestQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "orderPayerTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "coinVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": {
            "defined": "Side"
          }
        },
        {
          "name": "limitPrice",
          "type": "u64"
        },
        {
          "name": "maxCoinQty",
          "type": "u64"
        },
        {
          "name": "orderType",
          "type": {
            "defined": "OrderType"
          }
        },
        {
          "name": "clientOrderId",
          "type": "u64"
        },
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    },
    {
      "name": "matchOrders",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "reqQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinFeeReceivableAccount",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "pcFeeReceivableAccount",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    },
    {
      "name": "consumeEvents",
      "accounts": [
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinFeeReceivableAccount",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "pcFeeReceivableAccount",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    },
    {
      "name": "cancelOrder",
      "accounts": [
        {
          "name": "market",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "reqQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": {
            "defined": "Side"
          }
        },
        {
          "name": "orderId",
          "type": "u128"
        }
      ]
    },
    {
      "name": "settleFunds",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "coinWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "vaultSigner",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "disableMarket",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "disableAuthorityKey",
          "isMut": false,
          "isSigner": true
        }
      ],
      "args": []
    },
    {
      "name": "sweepFees",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "sweepAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "sweepReceiverAmount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "vaultSigner",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "cancelOrderV2",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": {
            "defined": "Side"
          }
        },
        {
          "name": "orderId",
          "type": "u128"
        }
      ]
    },
    {
      "name": "cancelOrderByClientV2",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "clientId",
          "type": "u64"
        }
      ]
    },
    {
      "name": "sendTake",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "reqQueue",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": {
            "defined": "Side"
          }
        },
        {
          "name": "limitPrice",
          "type": "u64"
        },
        {
          "name": "maxCoinQty",
          "type": "u64"
        },
        {
          "name": "maxNativePcQtyIncludingFees",
          "type": "u64"
        },
        {
          "name": "minCoinQty",
          "type": "u64"
        },
        {
          "name": "minNatuvePcQty",
          "type": "u64"
        },
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    },
    {
      "name": "closeOpenOrders",
      "accounts": [
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "destination",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "initOpenOrders",
      "accounts": [
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "marketAuthority",
          "isMut": false,
          "isSigner": true
        }
      ],
      "args": []
    },
    {
      "name": "prune",
      "accounts": [
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrders",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "pruneAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "openOrdersAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    },
    {
      "name": "consumeEventsPermissioned",
      "accounts": [
        {
          "name": "openOrders",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "consumeEventsAuthority",
          "isMut": false,
          "isSigner": true
        }
      ],
      "args": [
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    }
  ],
  "types": [
    {
      "name": "Side",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Bid"
          },
          {
            "name": "Ask"
          }
        ]
      }
    },
    {
      "name": "OrderType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Limit"
          },
          {
            "name": "ImmediateOrCancel"
          },
          {
            "name": "PostOnly"
          }
        ]
      }
    },
    {
      "name": "SelfTradeBehavior",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "DecrementTake"
          },
          {
            "name": "CancelProvide"
          },
          {
            "name": "AbortTransaction"
          }
        ]
      }
    }
  ]
}
