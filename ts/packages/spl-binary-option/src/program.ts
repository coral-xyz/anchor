import { PublicKey } from "@solana/web3.js";
import { Program, AnchorProvider } from "@project-serum/anchor";

import { SplBinaryOptionCoder } from "./coder";

export const SPL_BINARY_OPTION_PROGRAM_ID = new PublicKey(
  "betw959P4WToez4DkuXwNsJszqbpe3HuY56AcG5yevx"
);

interface GetProgramParams {
  programId?: PublicKey;
  provider?: AnchorProvider;
}

export function splBinaryOptionProgram(
  params?: GetProgramParams
): Program<SplBinaryOption> {
  return new Program<SplBinaryOption>(
    IDL,
    params?.programId ?? SPL_BINARY_OPTION_PROGRAM_ID,
    params?.provider,
    new SplBinaryOptionCoder(IDL)
  );
}

type SplBinaryOption = {
  version: "0.1.0";
  name: "spl_binary_option";
  instructions: [
    {
      name: "initializeBinaryOption";
      accounts: [
        {
          name: "poolAccount";
          isMut: true;
          isSigner: true;
        },
        {
          name: "escrowMint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "escrowAccount";
          isMut: true;
          isSigner: true;
        },
        {
          name: "longTokenMint";
          isMut: true;
          isSigner: true;
        },
        {
          name: "shortTokenMint";
          isMut: true;
          isSigner: true;
        },
        {
          name: "mintAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "updateAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "tokenProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "rent";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "decimals";
          type: "u8";
        }
      ];
    },
    {
      name: "trade";
      accounts: [
        {
          name: "poolAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "escrowAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "longTokenMint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "shortTokenMint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "buyer";
          isMut: false;
          isSigner: true;
        },
        {
          name: "seller";
          isMut: false;
          isSigner: true;
        },
        {
          name: "buyerAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "sellerAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "buyerLongTokenAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "buyerShortTokenAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "sellerLongTokenAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "sellerShortTokenAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "escrowAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "tokenProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "size";
          type: "u64";
        },
        {
          name: "buyPrice";
          type: "u64";
        },
        {
          name: "sellPrice";
          type: "u64";
        }
      ];
    },
    {
      name: "settle";
      accounts: [
        {
          name: "poolAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "winningMint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "poolAuthority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [];
    },
    {
      name: "collect";
      accounts: [
        {
          name: "poolAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "collectorAccount";
          isMut: false;
          isSigner: false;
        },
        {
          name: "collectorLongTokenAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "collectorShortTokenAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "collectorCollateralAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "longTokenMintAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "shortTokenMintAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "escrowAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "escrowAuthorityAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    }
  ];
  accounts: [
    {
      name: "binaryOption";
      type: {
        kind: "struct";
        fields: [
          {
            name: "decimals";
            type: "u8";
          },
          {
            name: "circulation";
            type: "u64";
          },
          {
            name: "settled";
            type: "bool";
          },
          {
            name: "escrowMintAccountPubkey";
            type: "publicKey";
          },
          {
            name: "escrowAccountPubkey";
            type: "publicKey";
          },
          {
            name: "longMintAccountPubkey";
            type: "publicKey";
          },
          {
            name: "shortMintAccountPubkey";
            type: "publicKey";
          },
          {
            name: "owner";
            type: "publicKey";
          },
          {
            name: "winningSidePubkey";
            type: "publicKey";
          }
        ];
      };
    }
  ];
  errors: [
    {
      code: 0;
      name: "PublicKeyMismatch";
      msg: "PublicKeyMismatch";
    },
    {
      code: 1;
      name: "InvalidMintAuthority";
      msg: "InvalidMintAuthority";
    },
    {
      code: 2;
      name: "NotMintAuthority";
      msg: "NotMintAuthority";
    },
    {
      code: 3;
      name: "InvalidSupply";
      msg: "InvalidSupply";
    },
    {
      code: 4;
      name: "InvalidWinner";
      msg: "InvalidWinner";
    },
    {
      code: 5;
      name: "UninitializedAccount";
      msg: "UninitializedAccount";
    },
    {
      code: 6;
      name: "IncorrectOwner";
      msg: "IncorrectOwner";
    },
    {
      code: 7;
      name: "AlreadySettled";
      msg: "AlreadySettled";
    },
    {
      code: 8;
      name: "BetNotSettled";
      msg: "BetNotSettled";
    },
    {
      code: 9;
      name: "TokenNotFoundInPool";
      msg: "TokenNotFoundInPool";
    },
    {
      code: 10;
      name: "PublicKeysShouldBeUnique";
      msg: "PublicKeysShouldBeUnique";
    },
    {
      code: 11;
      name: "TradePricesIncorrect";
      msg: "TradePricesIncorrect";
    }
  ];
};

const IDL: SplBinaryOption = {
  version: "0.1.0",
  name: "spl_binary_option",
  instructions: [
    {
      name: "initializeBinaryOption",
      accounts: [
        {
          name: "poolAccount",
          isMut: true,
          isSigner: true,
        },
        {
          name: "escrowMint",
          isMut: false,
          isSigner: false,
        },
        {
          name: "escrowAccount",
          isMut: true,
          isSigner: true,
        },
        {
          name: "longTokenMint",
          isMut: true,
          isSigner: true,
        },
        {
          name: "shortTokenMint",
          isMut: true,
          isSigner: true,
        },
        {
          name: "mintAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "updateAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "rent",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "decimals",
          type: "u8",
        },
      ],
    },
    {
      name: "trade",
      accounts: [
        {
          name: "poolAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "escrowAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "longTokenMint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "shortTokenMint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "buyer",
          isMut: false,
          isSigner: true,
        },
        {
          name: "seller",
          isMut: false,
          isSigner: true,
        },
        {
          name: "buyerAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "sellerAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "buyerLongTokenAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "buyerShortTokenAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "sellerLongTokenAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "sellerShortTokenAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "escrowAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "size",
          type: "u64",
        },
        {
          name: "buyPrice",
          type: "u64",
        },
        {
          name: "sellPrice",
          type: "u64",
        },
      ],
    },
    {
      name: "settle",
      accounts: [
        {
          name: "poolAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "winningMint",
          isMut: false,
          isSigner: false,
        },
        {
          name: "poolAuthority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [],
    },
    {
      name: "collect",
      accounts: [
        {
          name: "poolAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "collectorAccount",
          isMut: false,
          isSigner: false,
        },
        {
          name: "collectorLongTokenAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "collectorShortTokenAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "collectorCollateralAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "longTokenMintAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "shortTokenMintAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "escrowAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "escrowAuthorityAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
  ],
  accounts: [
    {
      name: "binaryOption",
      type: {
        kind: "struct",
        fields: [
          {
            name: "decimals",
            type: "u8",
          },
          {
            name: "circulation",
            type: "u64",
          },
          {
            name: "settled",
            type: "bool",
          },
          {
            name: "escrowMintAccountPubkey",
            type: "publicKey",
          },
          {
            name: "escrowAccountPubkey",
            type: "publicKey",
          },
          {
            name: "longMintAccountPubkey",
            type: "publicKey",
          },
          {
            name: "shortMintAccountPubkey",
            type: "publicKey",
          },
          {
            name: "owner",
            type: "publicKey",
          },
          {
            name: "winningSidePubkey",
            type: "publicKey",
          },
        ],
      },
    },
  ],
  errors: [
    {
      code: 0,
      name: "PublicKeyMismatch",
      msg: "PublicKeyMismatch",
    },
    {
      code: 1,
      name: "InvalidMintAuthority",
      msg: "InvalidMintAuthority",
    },
    {
      code: 2,
      name: "NotMintAuthority",
      msg: "NotMintAuthority",
    },
    {
      code: 3,
      name: "InvalidSupply",
      msg: "InvalidSupply",
    },
    {
      code: 4,
      name: "InvalidWinner",
      msg: "InvalidWinner",
    },
    {
      code: 5,
      name: "UninitializedAccount",
      msg: "UninitializedAccount",
    },
    {
      code: 6,
      name: "IncorrectOwner",
      msg: "IncorrectOwner",
    },
    {
      code: 7,
      name: "AlreadySettled",
      msg: "AlreadySettled",
    },
    {
      code: 8,
      name: "BetNotSettled",
      msg: "BetNotSettled",
    },
    {
      code: 9,
      name: "TokenNotFoundInPool",
      msg: "TokenNotFoundInPool",
    },
    {
      code: 10,
      name: "PublicKeysShouldBeUnique",
      msg: "PublicKeysShouldBeUnique",
    },
    {
      code: 11,
      name: "TradePricesIncorrect",
      msg: "TradePricesIncorrect",
    },
  ],
};
