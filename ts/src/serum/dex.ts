import { PublicKey } from "@solana/web3.js";
import { Program } from "../program/index.js";
import Provider from "../provider.js";
import { SerumCoder } from "../coder/serum";

const DEX_V4_PROGRAM_ID_DEVNET = new PublicKey(
  "GGKAzVAfJqtNPHhA8tGzz6RFnbinejaTdJZkVimSenM1"
);

export function program(provider?: Provider): Program<SerumDex> {
  return new Program<SerumDex>(
    IDL,
    DEX_V4_PROGRAM_ID_DEVNET,
    provider,
    coder()
  );
}

export function coder(): SerumCoder {
  return new SerumCoder(IDL);
}

export type SerumDex = {
  version: "0.1.0";
  name: "serum_dex";
  instructions: [
    {
      name: "createMarket";
      accounts: [
        {
          name: "market";
          isMut: true;
          isSigner: false;
        },
        {
          name: "orderbook";
          isMut: true;
          isSigner: false;
        },
        {
          name: "baseVault";
          isMut: false;
          isSigner: false;
        },
        {
          name: "quoteVault";
          isMut: false;
          isSigner: false;
        },
        {
          name: "marketAdmin";
          isMut: false;
          isSigner: false;
        },
        {
          name: "eventQueue";
          isMut: true;
          isSigner: false;
        },
        {
          name: "asks";
          isMut: true;
          isSigner: false;
        },
        {
          name: "bids";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenMetadata";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "signerNonce";
          type: "u64";
        },
        {
          name: "minBaseOrderSize";
          type: "u64";
        },
        {
          name: "tickSize";
          type: "u64";
        },
        {
          name: "crankerReward";
          type: "u64";
        }
      ];
    },
    {
      name: "closeMarket";
      accounts: [
        {
          name: "market";
          isMut: true;
          isSigner: false;
        },
        {
          name: "baseVault";
          isMut: true;
          isSigner: false;
        },
        {
          name: "quoteVault";
          isMut: true;
          isSigner: false;
        },
        {
          name: "orderbook";
          isMut: true;
          isSigner: false;
        },
        {
          name: "eventQueue";
          isMut: true;
          isSigner: false;
        },
        {
          name: "bids";
          isMut: true;
          isSigner: false;
        },
        {
          name: "asks";
          isMut: true;
          isSigner: false;
        },
        {
          name: "marketAdmin";
          isMut: false;
          isSigner: true;
        },
        {
          name: "targetLamportsAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "marketSigner";
          isMut: false;
          isSigner: false;
        },
        {
          name: "splTokenProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "closeAccount";
      accounts: [
        {
          name: "user";
          isMut: true;
          isSigner: false;
        },
        {
          name: "userOwner";
          isMut: false;
          isSigner: true;
        },
        {
          name: "targetLamportsAccount";
          isMut: true;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "sweepFees";
      accounts: [
        {
          name: "market";
          isMut: true;
          isSigner: false;
        },
        {
          name: "marketSigner";
          isMut: false;
          isSigner: false;
        },
        {
          name: "quoteVault";
          isMut: true;
          isSigner: false;
        },
        {
          name: "destinationTokenAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "splTokenProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "tokenMetadata";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "initializeAccount";
      accounts: [
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "user";
          isMut: true;
          isSigner: false;
        },
        {
          name: "userOwner";
          isMut: false;
          isSigner: true;
        },
        {
          name: "feePayer";
          isMut: true;
          isSigner: true;
        }
      ];
      args: [
        {
          name: "market";
          type: "publicKey";
        },
        {
          name: "maxOrders";
          type: "u64";
        }
      ];
    },
    {
      name: "settle";
      accounts: [
        {
          name: "splTokenProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "market";
          isMut: false;
          isSigner: false;
        },
        {
          name: "baseVault";
          isMut: true;
          isSigner: false;
        },
        {
          name: "quoteVault";
          isMut: true;
          isSigner: false;
        },
        {
          name: "marketSigner";
          isMut: false;
          isSigner: false;
        },
        {
          name: "user";
          isMut: true;
          isSigner: false;
        },
        {
          name: "userOwner";
          isMut: false;
          isSigner: true;
        },
        {
          name: "destinationBaseAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "destinationQuoteAccount";
          isMut: true;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "consumeEvents";
      accounts: [
        {
          name: "market";
          isMut: true;
          isSigner: false;
        },
        {
          name: "orderbook";
          isMut: true;
          isSigner: false;
        },
        {
          name: "eventQueue";
          isMut: true;
          isSigner: false;
        },
        {
          name: "rewardTarget";
          isMut: true;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "maxIterations";
          type: "u64";
        },
        {
          name: "noOpErr";
          type: "u64";
        }
      ];
    },
    {
      name: "cancelOrder";
      accounts: [
        {
          name: "market";
          isMut: false;
          isSigner: false;
        },
        {
          name: "orderbook";
          isMut: true;
          isSigner: false;
        },
        {
          name: "eventQueue";
          isMut: true;
          isSigner: false;
        },
        {
          name: "bids";
          isMut: true;
          isSigner: false;
        },
        {
          name: "asks";
          isMut: true;
          isSigner: false;
        },
        {
          name: "user";
          isMut: true;
          isSigner: false;
        },
        {
          name: "userOwner";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [
        {
          name: "orderIndex";
          type: "u64";
        },
        {
          name: "orderId";
          type: "u128";
        }
      ];
    },
    {
      name: "swap";
      accounts: [
        {
          name: "splTokenProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "market";
          isMut: true;
          isSigner: false;
        },
        {
          name: "orderbook";
          isMut: true;
          isSigner: false;
        },
        {
          name: "eventQueue";
          isMut: true;
          isSigner: false;
        },
        {
          name: "bids";
          isMut: true;
          isSigner: false;
        },
        {
          name: "asks";
          isMut: true;
          isSigner: false;
        },
        {
          name: "baseVault";
          isMut: true;
          isSigner: false;
        },
        {
          name: "quoteVault";
          isMut: true;
          isSigner: false;
        },
        {
          name: "marketSigner";
          isMut: false;
          isSigner: false;
        },
        {
          name: "userBaseAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "userQuoteAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "userOwner";
          isMut: true;
          isSigner: true;
        }
      ];
      args: [
        {
          name: "baseQty";
          type: "u64";
        },
        {
          name: "quoteQty";
          type: "u64";
        },
        {
          name: "matchLimit";
          type: "u64";
        },
        {
          name: "side";
          type: "u8";
        },
        {
          name: "hasDiscountTokenAccount";
          type: "u8";
        },
        {
          name: "padding";
          type: {
            array: ["u8", 6];
          };
        }
      ];
    },
    {
      name: "updateRoyalties";
      accounts: [
        {
          name: "market";
          isMut: true;
          isSigner: false;
        },
        {
          name: "eventQueue";
          isMut: false;
          isSigner: false;
        },
        {
          name: "orderbook";
          isMut: false;
          isSigner: false;
        },
        {
          name: "tokenMetadata";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    }
  ];
  accounts: [
    {
      name: "MarketState";
      type: {
        kind: "struct";
        fields: [
          {
            name: "tag";
            type: "u64";
          },
          {
            name: "baseMint";
            type: "publicKey";
          },
          {
            name: "quoteMint";
            type: "publicKey";
          },
          {
            name: "baseVault";
            type: "publicKey";
          },
          {
            name: "quoteVault";
            type: "publicKey";
          },
          {
            name: "orderbook";
            type: "publicKey";
          },
          {
            name: "admin";
            type: "publicKey";
          },
          {
            name: "creationTimestamp";
            type: "i64";
          },
          {
            name: "baseVolume";
            type: "u64";
          },
          {
            name: "quoteVolume";
            type: "u64";
          },
          {
            name: "accumulatedFees";
            type: "u64";
          },
          {
            name: "minBaseOrderSize";
            type: "u64";
          },
          {
            name: "royaltiesBps";
            type: "u64";
          },
          {
            name: "accumulatedRoyalties";
            type: "u64";
          },
          {
            name: "signerNonce";
            type: "u8";
          },
          {
            name: "feeType";
            type: "u8";
          },
          {
            name: "padding";
            type: {
              array: ["u8", 6];
            };
          }
        ];
      };
    }
  ];
};

export const IDL: SerumDex = {
  version: "0.1.0",
  name: "serum_dex",
  instructions: [
    {
      name: "createMarket",
      accounts: [
        {
          name: "market",
          isMut: true,
          isSigner: false,
        },
        {
          name: "orderbook",
          isMut: true,
          isSigner: false,
        },
        {
          name: "baseVault",
          isMut: false,
          isSigner: false,
        },
        {
          name: "quoteVault",
          isMut: false,
          isSigner: false,
        },
        {
          name: "marketAdmin",
          isMut: false,
          isSigner: false,
        },
        {
          name: "eventQueue",
          isMut: true,
          isSigner: false,
        },
        {
          name: "asks",
          isMut: true,
          isSigner: false,
        },
        {
          name: "bids",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenMetadata",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "signerNonce",
          type: "u64",
        },
        {
          name: "minBaseOrderSize",
          type: "u64",
        },
        {
          name: "tickSize",
          type: "u64",
        },
        {
          name: "crankerReward",
          type: "u64",
        },
      ],
    },
    {
      name: "closeMarket",
      accounts: [
        {
          name: "market",
          isMut: true,
          isSigner: false,
        },
        {
          name: "baseVault",
          isMut: true,
          isSigner: false,
        },
        {
          name: "quoteVault",
          isMut: true,
          isSigner: false,
        },
        {
          name: "orderbook",
          isMut: true,
          isSigner: false,
        },
        {
          name: "eventQueue",
          isMut: true,
          isSigner: false,
        },
        {
          name: "bids",
          isMut: true,
          isSigner: false,
        },
        {
          name: "asks",
          isMut: true,
          isSigner: false,
        },
        {
          name: "marketAdmin",
          isMut: false,
          isSigner: true,
        },
        {
          name: "targetLamportsAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "marketSigner",
          isMut: false,
          isSigner: false,
        },
        {
          name: "splTokenProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "closeAccount",
      accounts: [
        {
          name: "user",
          isMut: true,
          isSigner: false,
        },
        {
          name: "userOwner",
          isMut: false,
          isSigner: true,
        },
        {
          name: "targetLamportsAccount",
          isMut: true,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "sweepFees",
      accounts: [
        {
          name: "market",
          isMut: true,
          isSigner: false,
        },
        {
          name: "marketSigner",
          isMut: false,
          isSigner: false,
        },
        {
          name: "quoteVault",
          isMut: true,
          isSigner: false,
        },
        {
          name: "destinationTokenAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "splTokenProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "tokenMetadata",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "initializeAccount",
      accounts: [
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "user",
          isMut: true,
          isSigner: false,
        },
        {
          name: "userOwner",
          isMut: false,
          isSigner: true,
        },
        {
          name: "feePayer",
          isMut: true,
          isSigner: true,
        },
      ],
      args: [
        {
          name: "market",
          type: "publicKey",
        },
        {
          name: "maxOrders",
          type: "u64",
        },
      ],
    },
    {
      name: "settle",
      accounts: [
        {
          name: "splTokenProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "market",
          isMut: false,
          isSigner: false,
        },
        {
          name: "baseVault",
          isMut: true,
          isSigner: false,
        },
        {
          name: "quoteVault",
          isMut: true,
          isSigner: false,
        },
        {
          name: "marketSigner",
          isMut: false,
          isSigner: false,
        },
        {
          name: "user",
          isMut: true,
          isSigner: false,
        },
        {
          name: "userOwner",
          isMut: false,
          isSigner: true,
        },
        {
          name: "destinationBaseAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "destinationQuoteAccount",
          isMut: true,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "consumeEvents",
      accounts: [
        {
          name: "market",
          isMut: true,
          isSigner: false,
        },
        {
          name: "orderbook",
          isMut: true,
          isSigner: false,
        },
        {
          name: "eventQueue",
          isMut: true,
          isSigner: false,
        },
        {
          name: "rewardTarget",
          isMut: true,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "maxIterations",
          type: "u64",
        },
        {
          name: "noOpErr",
          type: "u64",
        },
      ],
    },
    {
      name: "cancelOrder",
      accounts: [
        {
          name: "market",
          isMut: false,
          isSigner: false,
        },
        {
          name: "orderbook",
          isMut: true,
          isSigner: false,
        },
        {
          name: "eventQueue",
          isMut: true,
          isSigner: false,
        },
        {
          name: "bids",
          isMut: true,
          isSigner: false,
        },
        {
          name: "asks",
          isMut: true,
          isSigner: false,
        },
        {
          name: "user",
          isMut: true,
          isSigner: false,
        },
        {
          name: "userOwner",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [
        {
          name: "orderIndex",
          type: "u64",
        },
        {
          name: "orderId",
          type: "u128",
        },
      ],
    },
    {
      name: "swap",
      accounts: [
        {
          name: "splTokenProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "market",
          isMut: true,
          isSigner: false,
        },
        {
          name: "orderbook",
          isMut: true,
          isSigner: false,
        },
        {
          name: "eventQueue",
          isMut: true,
          isSigner: false,
        },
        {
          name: "bids",
          isMut: true,
          isSigner: false,
        },
        {
          name: "asks",
          isMut: true,
          isSigner: false,
        },
        {
          name: "baseVault",
          isMut: true,
          isSigner: false,
        },
        {
          name: "quoteVault",
          isMut: true,
          isSigner: false,
        },
        {
          name: "marketSigner",
          isMut: false,
          isSigner: false,
        },
        {
          name: "userBaseAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "userQuoteAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "userOwner",
          isMut: true,
          isSigner: true,
        },
      ],
      args: [
        {
          name: "baseQty",
          type: "u64",
        },
        {
          name: "quoteQty",
          type: "u64",
        },
        {
          name: "matchLimit",
          type: "u64",
        },
        {
          name: "side",
          type: "u8",
        },
        {
          name: "hasDiscountTokenAccount",
          type: "u8",
        },
        {
          name: "padding",
          type: {
            array: ["u8", 6],
          },
        },
      ],
    },
    {
      name: "updateRoyalties",
      accounts: [
        {
          name: "market",
          isMut: true,
          isSigner: false,
        },
        {
          name: "eventQueue",
          isMut: false,
          isSigner: false,
        },
        {
          name: "orderbook",
          isMut: false,
          isSigner: false,
        },
        {
          name: "tokenMetadata",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
  ],
  accounts: [
    {
      name: "MarketState",
      type: {
        kind: "struct",
        fields: [
          {
            name: "tag",
            type: "u64",
          },
          {
            name: "baseMint",
            type: "publicKey",
          },
          {
            name: "quoteMint",
            type: "publicKey",
          },
          {
            name: "baseVault",
            type: "publicKey",
          },
          {
            name: "quoteVault",
            type: "publicKey",
          },
          {
            name: "orderbook",
            type: "publicKey",
          },
          {
            name: "admin",
            type: "publicKey",
          },
          {
            name: "creationTimestamp",
            type: "i64",
          },
          {
            name: "baseVolume",
            type: "u64",
          },
          {
            name: "quoteVolume",
            type: "u64",
          },
          {
            name: "accumulatedFees",
            type: "u64",
          },
          {
            name: "minBaseOrderSize",
            type: "u64",
          },
          {
            name: "royaltiesBps",
            type: "u64",
          },
          {
            name: "accumulatedRoyalties",
            type: "u64",
          },
          {
            name: "signerNonce",
            type: "u8",
          },
          {
            name: "feeType",
            type: "u8",
          },
          {
            name: "padding",
            type: {
              array: ["u8", 6],
            },
          },
        ],
      },
    },
  ],
};
