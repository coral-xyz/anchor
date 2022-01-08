import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Program } from "../program/index.js";
import Provider from "../provider.js";
import { SplTokenCoder } from "../coder/spl-token/index.js";

export function program(provider?: Provider): Program<SplToken> {
  return new Program<SplToken>(
    IDL,
    TOKEN_PROGRAM_ID,
    provider,
    new SplTokenCoder(IDL)
  );
}

/**
 * SplToken IDL.
 */
export type SplToken = {
  version: "0.1.0";
  name: "custom_coder";
  instructions: [
    {
      name: "initializeMint";
      accounts: [
        {
          name: "mint";
          isMut: true;
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
        },
        {
          name: "mintAuthority";
          type: "publicKey";
        },
        {
          name: "freezeAuthority";
          type: {
            coption: "publicKey";
          };
        }
      ];
    },
    {
      name: "initializeMultisig";
      accounts: [
        {
          name: "account";
          isMut: true;
          isSigner: false;
        },
        {
          name: "rent";
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
          name: "account";
          isMut: true;
          isSigner: false;
        },
        {
          name: "mint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "authority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "rent";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "transfer";
      accounts: [
        {
          name: "source";
          isMut: true;
          isSigner: false;
        },
        {
          name: "destination";
          isMut: true;
          isSigner: false;
        },
        {
          name: "authority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [
        {
          name: "amount";
          type: "u64";
        }
      ];
    },
    {
      name: "approve";
      accounts: [
        {
          name: "source";
          isMut: true;
          isSigner: false;
        },
        {
          name: "delegate";
          isMut: false;
          isSigner: false;
        },
        {
          name: "authority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [];
    },
    {
      name: "revoke";
      accounts: [
        {
          name: "source";
          isMut: true;
          isSigner: false;
        },
        {
          name: "authority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [];
    },
    {
      name: "setAuthority";
      accounts: [
        {
          name: "mint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "authority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [];
    },
    {
      name: "mintTo";
      accounts: [
        {
          name: "mint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "to";
          isMut: true;
          isSigner: false;
        },
        {
          name: "authority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [];
    },
    {
      name: "burn";
      accounts: [
        {
          name: "source";
          isMut: true;
          isSigner: false;
        },
        {
          name: "mint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "authority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [];
    },
    {
      name: "closeAccount";
      accounts: [
        {
          name: "account";
          isMut: true;
          isSigner: false;
        },
        {
          name: "destination";
          isMut: true;
          isSigner: false;
        },
        {
          name: "authority";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "freezeAccount";
      accounts: [
        {
          name: "account";
          isMut: true;
          isSigner: false;
        },
        {
          name: "mint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "authority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [];
    },
    {
      name: "thawAccount";
      accounts: [
        {
          name: "account";
          isMut: true;
          isSigner: false;
        },
        {
          name: "mint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "authority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [];
    },
    {
      name: "transferChecked";
      accounts: [
        {
          name: "source";
          isMut: true;
          isSigner: false;
        },
        {
          name: "mint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "destination";
          isMut: true;
          isSigner: false;
        },
        {
          name: "authority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [];
    },
    {
      name: "approveChecked";
      accounts: [
        {
          name: "source";
          isMut: true;
          isSigner: false;
        },
        {
          name: "mint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "delegate";
          isMut: false;
          isSigner: false;
        },
        {
          name: "authority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [];
    },
    {
      name: "mintToChecked";
      accounts: [
        {
          name: "mint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "to";
          isMut: true;
          isSigner: false;
        },
        {
          name: "authority";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "burnChecked";
      accounts: [
        {
          name: "source";
          isMut: true;
          isSigner: false;
        },
        {
          name: "mint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "authority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [];
    },
    {
      name: "initializeAccount2";
      accounts: [
        {
          name: "account";
          isMut: true;
          isSigner: false;
        },
        {
          name: "mint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "rent";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "syncNative";
      accounts: [
        {
          name: "account";
          isMut: true;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "initializeAccount3";
      accounts: [
        {
          name: "account";
          isMut: true;
          isSigner: false;
        },
        {
          name: "mint";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "initializeMultisig2";
      accounts: [
        {
          name: "account";
          isMut: true;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "initializeMint2";
      accounts: [
        {
          name: "mint";
          isMut: true;
          isSigner: false;
        }
      ];
      args: [];
    }
  ];
};

export const IDL: SplToken = {
  version: "0.1.0",
  name: "custom_coder",
  instructions: [
    {
      name: "initializeMint",
      accounts: [
        {
          name: "mint",
          isMut: true,
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
        {
          name: "mintAuthority",
          type: "publicKey",
        },
        {
          name: "freezeAuthority",
          type: {
            coption: "publicKey",
          },
        },
      ],
    },
    {
      name: "initializeMultisig",
      accounts: [
        {
          name: "account",
          isMut: true,
          isSigner: false,
        },
        {
          name: "rent",
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
          name: "account",
          isMut: true,
          isSigner: false,
        },
        {
          name: "mint",
          isMut: false,
          isSigner: false,
        },
        {
          name: "authority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "rent",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "transfer",
      accounts: [
        {
          name: "source",
          isMut: true,
          isSigner: false,
        },
        {
          name: "destination",
          isMut: true,
          isSigner: false,
        },
        {
          name: "authority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [
        {
          name: "amount",
          type: "u64",
        },
      ],
    },
    {
      name: "approve",
      accounts: [
        {
          name: "source",
          isMut: true,
          isSigner: false,
        },
        {
          name: "delegate",
          isMut: false,
          isSigner: false,
        },
        {
          name: "authority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [],
    },
    {
      name: "revoke",
      accounts: [
        {
          name: "source",
          isMut: true,
          isSigner: false,
        },
        {
          name: "authority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [],
    },
    {
      name: "setAuthority",
      accounts: [
        {
          name: "mint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "authority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [],
    },
    {
      name: "mintTo",
      accounts: [
        {
          name: "mint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "to",
          isMut: true,
          isSigner: false,
        },
        {
          name: "authority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [],
    },
    {
      name: "burn",
      accounts: [
        {
          name: "source",
          isMut: true,
          isSigner: false,
        },
        {
          name: "mint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "authority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [],
    },
    {
      name: "closeAccount",
      accounts: [
        {
          name: "account",
          isMut: true,
          isSigner: false,
        },
        {
          name: "destination",
          isMut: true,
          isSigner: false,
        },
        {
          name: "authority",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "freezeAccount",
      accounts: [
        {
          name: "account",
          isMut: true,
          isSigner: false,
        },
        {
          name: "mint",
          isMut: false,
          isSigner: false,
        },
        {
          name: "authority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [],
    },
    {
      name: "thawAccount",
      accounts: [
        {
          name: "account",
          isMut: true,
          isSigner: false,
        },
        {
          name: "mint",
          isMut: false,
          isSigner: false,
        },
        {
          name: "authority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [],
    },
    {
      name: "transferChecked",
      accounts: [
        {
          name: "source",
          isMut: true,
          isSigner: false,
        },
        {
          name: "mint",
          isMut: false,
          isSigner: false,
        },
        {
          name: "destination",
          isMut: true,
          isSigner: false,
        },
        {
          name: "authority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [],
    },
    {
      name: "approveChecked",
      accounts: [
        {
          name: "source",
          isMut: true,
          isSigner: false,
        },
        {
          name: "mint",
          isMut: false,
          isSigner: false,
        },
        {
          name: "delegate",
          isMut: false,
          isSigner: false,
        },
        {
          name: "authority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [],
    },
    {
      name: "mintToChecked",
      accounts: [
        {
          name: "mint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "to",
          isMut: true,
          isSigner: false,
        },
        {
          name: "authority",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "burnChecked",
      accounts: [
        {
          name: "source",
          isMut: true,
          isSigner: false,
        },
        {
          name: "mint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "authority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [],
    },
    {
      name: "initializeAccount2",
      accounts: [
        {
          name: "account",
          isMut: true,
          isSigner: false,
        },
        {
          name: "mint",
          isMut: false,
          isSigner: false,
        },
        {
          name: "rent",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "syncNative",
      accounts: [
        {
          name: "account",
          isMut: true,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "initializeAccount3",
      accounts: [
        {
          name: "account",
          isMut: true,
          isSigner: false,
        },
        {
          name: "mint",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "initializeMultisig2",
      accounts: [
        {
          name: "account",
          isMut: true,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "initializeMint2",
      accounts: [
        {
          name: "mint",
          isMut: true,
          isSigner: false,
        },
      ],
      args: [],
    },
  ],
};
