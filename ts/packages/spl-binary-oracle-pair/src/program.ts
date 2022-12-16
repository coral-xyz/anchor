import { PublicKey } from "@solana/web3.js";
import { Program, AnchorProvider } from "@coral-xyz/anchor";

import { SplBinaryOraclePairCoder } from "./coder";

export const SPL_BINARY_ORACLE_PAIR_PROGRAM_ID = new PublicKey(
  "Fd7btgySsrjuo25CJCj7oE7VPMyezDhnx7pZkj2v69Nk"
);

interface GetProgramParams {
  programId?: PublicKey;
  provider?: AnchorProvider;
}

export function splBinaryOraclePairProgram(
  params?: GetProgramParams
): Program<SplBinaryOraclePair> {
  return new Program<SplBinaryOraclePair>(
    IDL,
    params?.programId ?? SPL_BINARY_ORACLE_PAIR_PROGRAM_ID,
    params?.provider,
    new SplBinaryOraclePairCoder(IDL)
  );
}

type SplBinaryOraclePair = {
  version: "0.1.0";
  name: "spl_binary_oracle_pair";
  instructions: [
    {
      name: "initPool";
      accounts: [
        {
          name: "pool";
          isMut: true;
          isSigner: false;
        },
        {
          name: "authority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "decider";
          isMut: false;
          isSigner: false;
        },
        {
          name: "depositTokenMint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "depositAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenPassMint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenFailMint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "rent";
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
          name: "mintEndSlot";
          type: "u64";
        },
        {
          name: "decideEndSlot";
          type: "u64";
        },
        {
          name: "bumpSeed";
          type: "u8";
        }
      ];
    },
    {
      name: "deposit";
      accounts: [
        {
          name: "pool";
          isMut: false;
          isSigner: false;
        },
        {
          name: "authority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "userTransferAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "userTokenAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "poolDepositTokenAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenPassMint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenFailMint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenPassDestinationAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenFailDestinationAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "clock";
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
          name: "arg";
          type: "u64";
        }
      ];
    },
    {
      name: "withdraw";
      accounts: [
        {
          name: "pool";
          isMut: false;
          isSigner: false;
        },
        {
          name: "authority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "userTransferAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "poolDepositTokenAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenPassUserAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenFailUserAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenPassMint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenFailMint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "userTokenDestinationAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "clock";
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
          name: "arg";
          type: "u64";
        }
      ];
    },
    {
      name: "decide";
      accounts: [
        {
          name: "pool";
          isMut: true;
          isSigner: false;
        },
        {
          name: "decider";
          isMut: false;
          isSigner: true;
        },
        {
          name: "clock";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "arg";
          type: "bool";
        }
      ];
    }
  ];
  accounts: [
    {
      name: "pool";
      type: {
        kind: "struct";
        fields: [
          {
            name: "version";
            type: "u8";
          },
          {
            name: "bumpSeed";
            type: "u8";
          },
          {
            name: "tokenProgramId";
            type: "publicKey";
          },
          {
            name: "depositAccount";
            type: "publicKey";
          },
          {
            name: "tokenPassMint";
            type: "publicKey";
          },
          {
            name: "tokenFailMint";
            type: "publicKey";
          },
          {
            name: "decider";
            type: "publicKey";
          },
          {
            name: "mintEndSlot";
            type: "u64";
          },
          {
            name: "decideEndSlot";
            type: "u64";
          },
          {
            name: "decision";
            type: {
              defined: "Decision";
            };
          }
        ];
      };
    }
  ];
  types: [
    {
      name: "Decision";
      type: {
        kind: "enum";
        variants: [
          {
            name: "Undecided";
          },
          {
            name: "Pass";
          },
          {
            name: "Fail";
          }
        ];
      };
    }
  ];
  errors: [
    {
      code: 0;
      name: "AlreadyInUse";
      msg: "Pool account already in use";
    },
    {
      code: 1;
      name: "DepositAccountInUse";
      msg: "Deposit account already in use";
    },
    {
      code: 2;
      name: "TokenMintInUse";
      msg: "Token account already in use";
    },
    {
      code: 3;
      name: "InvalidAuthorityData";
      msg: "Failed to generate program account because of invalid data";
    },
    {
      code: 4;
      name: "InvalidAuthorityAccount";
      msg: "Invalid authority account provided";
    },
    {
      code: 5;
      name: "NotRentExempt";
      msg: "Lamport balance below rent-exempt threshold";
    },
    {
      code: 6;
      name: "InvalidTokenMint";
      msg: "Input token mint account is not valid";
    },
    {
      code: 7;
      name: "InvalidAmount";
      msg: "Amount should be more than zero";
    },
    {
      code: 8;
      name: "WrongDeciderAccount";
      msg: "Wrong decider account was sent";
    },
    {
      code: 9;
      name: "SignatureMissing";
      msg: "Signature missing in transaction";
    },
    {
      code: 10;
      name: "DecisionAlreadyMade";
      msg: "Decision was already made for this pool";
    },
    {
      code: 11;
      name: "InvalidSlotForDecision";
      msg: "Decision can't be made in current slot";
    },
    {
      code: 12;
      name: "InvalidSlotForDeposit";
      msg: "Deposit can't be made in current slot";
    },
    {
      code: 13;
      name: "NoDecisionMadeYet";
      msg: "No decision has been made yet";
    }
  ];
};

const IDL: SplBinaryOraclePair = {
  version: "0.1.0",
  name: "spl_binary_oracle_pair",
  instructions: [
    {
      name: "initPool",
      accounts: [
        {
          name: "pool",
          isMut: true,
          isSigner: false,
        },
        {
          name: "authority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "decider",
          isMut: false,
          isSigner: false,
        },
        {
          name: "depositTokenMint",
          isMut: false,
          isSigner: false,
        },
        {
          name: "depositAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenPassMint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenFailMint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "rent",
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
          name: "mintEndSlot",
          type: "u64",
        },
        {
          name: "decideEndSlot",
          type: "u64",
        },
        {
          name: "bumpSeed",
          type: "u8",
        },
      ],
    },
    {
      name: "deposit",
      accounts: [
        {
          name: "pool",
          isMut: false,
          isSigner: false,
        },
        {
          name: "authority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "userTransferAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "userTokenAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "poolDepositTokenAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenPassMint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenFailMint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenPassDestinationAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenFailDestinationAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "clock",
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
          name: "arg",
          type: "u64",
        },
      ],
    },
    {
      name: "withdraw",
      accounts: [
        {
          name: "pool",
          isMut: false,
          isSigner: false,
        },
        {
          name: "authority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "userTransferAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "poolDepositTokenAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenPassUserAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenFailUserAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenPassMint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenFailMint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "userTokenDestinationAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "clock",
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
          name: "arg",
          type: "u64",
        },
      ],
    },
    {
      name: "decide",
      accounts: [
        {
          name: "pool",
          isMut: true,
          isSigner: false,
        },
        {
          name: "decider",
          isMut: false,
          isSigner: true,
        },
        {
          name: "clock",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "arg",
          type: "bool",
        },
      ],
    },
  ],
  accounts: [
    {
      name: "pool",
      type: {
        kind: "struct",
        fields: [
          {
            name: "version",
            type: "u8",
          },
          {
            name: "bumpSeed",
            type: "u8",
          },
          {
            name: "tokenProgramId",
            type: "publicKey",
          },
          {
            name: "depositAccount",
            type: "publicKey",
          },
          {
            name: "tokenPassMint",
            type: "publicKey",
          },
          {
            name: "tokenFailMint",
            type: "publicKey",
          },
          {
            name: "decider",
            type: "publicKey",
          },
          {
            name: "mintEndSlot",
            type: "u64",
          },
          {
            name: "decideEndSlot",
            type: "u64",
          },
          {
            name: "decision",
            type: {
              defined: "Decision",
            },
          },
        ],
      },
    },
  ],
  types: [
    {
      name: "Decision",
      type: {
        kind: "enum",
        variants: [
          {
            name: "Undecided",
          },
          {
            name: "Pass",
          },
          {
            name: "Fail",
          },
        ],
      },
    },
  ],
  errors: [
    {
      code: 0,
      name: "AlreadyInUse",
      msg: "Pool account already in use",
    },
    {
      code: 1,
      name: "DepositAccountInUse",
      msg: "Deposit account already in use",
    },
    {
      code: 2,
      name: "TokenMintInUse",
      msg: "Token account already in use",
    },
    {
      code: 3,
      name: "InvalidAuthorityData",
      msg: "Failed to generate program account because of invalid data",
    },
    {
      code: 4,
      name: "InvalidAuthorityAccount",
      msg: "Invalid authority account provided",
    },
    {
      code: 5,
      name: "NotRentExempt",
      msg: "Lamport balance below rent-exempt threshold",
    },
    {
      code: 6,
      name: "InvalidTokenMint",
      msg: "Input token mint account is not valid",
    },
    {
      code: 7,
      name: "InvalidAmount",
      msg: "Amount should be more than zero",
    },
    {
      code: 8,
      name: "WrongDeciderAccount",
      msg: "Wrong decider account was sent",
    },
    {
      code: 9,
      name: "SignatureMissing",
      msg: "Signature missing in transaction",
    },
    {
      code: 10,
      name: "DecisionAlreadyMade",
      msg: "Decision was already made for this pool",
    },
    {
      code: 11,
      name: "InvalidSlotForDecision",
      msg: "Decision can't be made in current slot",
    },
    {
      code: 12,
      name: "InvalidSlotForDeposit",
      msg: "Deposit can't be made in current slot",
    },
    {
      code: 13,
      name: "NoDecisionMadeYet",
      msg: "No decision has been made yet",
    },
  ],
};
