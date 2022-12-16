import { PublicKey } from "@solana/web3.js";
import { Program, AnchorProvider } from "@coral-xyz/anchor";

import { SplGovernanceCoder } from "./coder";

const SPL_GOVERNANCE_PROGRAM_ID = PublicKey.default;

interface GetProgramParams {
  programId?: PublicKey;
  provider?: AnchorProvider;
}

export function splGovernanceProgram(
  params?: GetProgramParams
): Program<SplGovernance> {
  return new Program<SplGovernance>(
    IDL,
    params?.programId ?? SPL_GOVERNANCE_PROGRAM_ID,
    params?.provider,
    new SplGovernanceCoder(IDL)
  );
}

type SplGovernance = {
  version: "3.0.0";
  name: "spl_governance";
  instructions: [
    {
      name: "createRealm";
      accounts: [
        {
          name: "realmAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "realmAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "communityTokenMint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "communityTokenHoldingAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "tokenProgram";
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
          name: "name";
          type: "string";
        },
        {
          name: "configArgs";
          type: {
            defined: "RealmConfigArgs";
          };
        }
      ];
    },
    {
      name: "depositGoverningTokens";
      accounts: [
        {
          name: "realm";
          isMut: false;
          isSigner: false;
        },
        {
          name: "governingTokenHoldingAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governingTokenSource";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governingTokenOwner";
          isMut: false;
          isSigner: true;
        },
        {
          name: "governingTokenTransferAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "tokenOwnerRecordAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "systemProgram";
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
          name: "amount";
          type: "u64";
        }
      ];
    },
    {
      name: "withdrawGoverningTokens";
      accounts: [
        {
          name: "realm";
          isMut: false;
          isSigner: false;
        },
        {
          name: "governingTokenHoldingAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governingTokenDestination";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governingTokenOwner";
          isMut: false;
          isSigner: true;
        },
        {
          name: "tokenOwnerRecordAddress";
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
    },
    {
      name: "setGovernanceDelegate";
      accounts: [
        {
          name: "governanceAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "voteRecordAddress";
          isMut: true;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "newGovernanceDelegate";
          type: {
            option: "publicKey";
          };
        }
      ];
    },
    {
      name: "createGovernance";
      accounts: [
        {
          name: "realm";
          isMut: false;
          isSigner: false;
        },
        {
          name: "governanceAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governedAccountAddress";
          isMut: false;
          isSigner: false;
        },
        {
          name: "tokenOwnerRecord";
          isMut: false;
          isSigner: false;
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "createAuthority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [
        {
          name: "config";
          type: {
            defined: "GovernanceConfig";
          };
        }
      ];
    },
    {
      name: "createProgramGovernance";
      accounts: [
        {
          name: "realm";
          isMut: false;
          isSigner: false;
        },
        {
          name: "programGovernanceAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governedProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "governedProgramDataAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governedProgramUpgradeAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "tokenOwnerRecord";
          isMut: false;
          isSigner: false;
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "bpfLoaderUpgradeable";
          isMut: false;
          isSigner: false;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "createAuthority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [
        {
          name: "config";
          type: {
            defined: "GovernanceConfig";
          };
        },
        {
          name: "transferUpgradeAuthority";
          type: "bool";
        }
      ];
    },
    {
      name: "createProposal";
      accounts: [
        {
          name: "realm";
          isMut: false;
          isSigner: false;
        },
        {
          name: "proposalAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governance";
          isMut: true;
          isSigner: false;
        },
        {
          name: "proposalOwnerRecord";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governingTokenMint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "governanceAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "name";
          type: "string";
        },
        {
          name: "descriptionLink";
          type: "string";
        },
        {
          name: "voteType";
          type: {
            defined: "VoteType";
          };
        },
        {
          name: "options";
          type: {
            vec: "string";
          };
        },
        {
          name: "useDenyOption";
          type: "bool";
        }
      ];
    },
    {
      name: "addSignatory";
      accounts: [
        {
          name: "proposal";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenOwnerRecord";
          isMut: false;
          isSigner: false;
        },
        {
          name: "governanceAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "signatoryRecordAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "signatory";
          type: "publicKey";
        }
      ];
    },
    {
      name: "removeSignatory";
      accounts: [
        {
          name: "proposal";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenOwnerRecord";
          isMut: false;
          isSigner: false;
        },
        {
          name: "governanceAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "signatoryRecordAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "beneficiary";
          isMut: true;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "signatory";
          type: "publicKey";
        }
      ];
    },
    {
      name: "insertTransaction";
      accounts: [
        {
          name: "governance";
          isMut: false;
          isSigner: false;
        },
        {
          name: "proposal";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenOwnerRecord";
          isMut: false;
          isSigner: false;
        },
        {
          name: "governanceAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "proposalTransactionAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
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
          name: "optionIndex";
          type: "u8";
        },
        {
          name: "index";
          type: "u16";
        },
        {
          name: "holdUpTime";
          type: "u32";
        },
        {
          name: "instructions";
          type: {
            vec: {
              defined: "InstructionData";
            };
          };
        }
      ];
    },
    {
      name: "removeTransaction";
      accounts: [
        {
          name: "proposal";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenOwnerRecord";
          isMut: false;
          isSigner: false;
        },
        {
          name: "governanceAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "proposalTransaction";
          isMut: true;
          isSigner: false;
        },
        {
          name: "beneficiary";
          isMut: true;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "cancelProposal";
      accounts: [
        {
          name: "realm";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governance";
          isMut: true;
          isSigner: false;
        },
        {
          name: "proposal";
          isMut: true;
          isSigner: false;
        },
        {
          name: "proposalOwnerRecord";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governanceAuthority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [];
    },
    {
      name: "signOffProposal";
      accounts: [
        {
          name: "realm";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governance";
          isMut: true;
          isSigner: false;
        },
        {
          name: "proposal";
          isMut: true;
          isSigner: false;
        },
        {
          name: "signatory";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [];
    },
    {
      name: "castVote";
      accounts: [
        {
          name: "realm";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governance";
          isMut: true;
          isSigner: false;
        },
        {
          name: "proposal";
          isMut: true;
          isSigner: false;
        },
        {
          name: "proposalOwnerRecord";
          isMut: true;
          isSigner: false;
        },
        {
          name: "voterTokenOwnerRecord";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governanceAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "voteRecordAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "voteGoverningTokenMint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "vote";
          type: {
            defined: "Vote";
          };
        }
      ];
    },
    {
      name: "finalizeVote";
      accounts: [
        {
          name: "realm";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governance";
          isMut: true;
          isSigner: false;
        },
        {
          name: "proposal";
          isMut: true;
          isSigner: false;
        },
        {
          name: "proposalOwnerRecord";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governingTokenMint";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "relinquishVote";
      accounts: [
        {
          name: "realm";
          isMut: false;
          isSigner: false;
        },
        {
          name: "governance";
          isMut: false;
          isSigner: false;
        },
        {
          name: "proposal";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenOwnerRecord";
          isMut: true;
          isSigner: false;
        },
        {
          name: "voteRecordAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "voteGoverningTokenMint";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "executeTransaction";
      accounts: [
        {
          name: "governance";
          isMut: false;
          isSigner: false;
        },
        {
          name: "proposal";
          isMut: true;
          isSigner: false;
        },
        {
          name: "proposalTransaction";
          isMut: true;
          isSigner: false;
        },
        {
          name: "instructionProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "createMintGovernance";
      accounts: [
        {
          name: "realm";
          isMut: false;
          isSigner: false;
        },
        {
          name: "mintGovernanceAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governedMint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governedMintAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "tokenOwnerRecord";
          isMut: false;
          isSigner: false;
        },
        {
          name: "payer";
          isMut: true;
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
          name: "createAuthority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [
        {
          name: "config";
          type: {
            defined: "GovernanceConfig";
          };
        },
        {
          name: "transferMintAuthorities";
          type: "bool";
        }
      ];
    },
    {
      name: "createTokenGovernance";
      accounts: [
        {
          name: "realm";
          isMut: false;
          isSigner: false;
        },
        {
          name: "tokenGovernanceAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governedToken";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governedTokenOwner";
          isMut: false;
          isSigner: true;
        },
        {
          name: "tokenOwnerRecord";
          isMut: false;
          isSigner: false;
        },
        {
          name: "payer";
          isMut: true;
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
          name: "createAuthority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [
        {
          name: "config";
          type: {
            defined: "GovernanceConfig";
          };
        },
        {
          name: "transferAccountAuthorities";
          type: "bool";
        }
      ];
    },
    {
      name: "setGovernanceConfig";
      accounts: [
        {
          name: "governance";
          isMut: true;
          isSigner: true;
        }
      ];
      args: [
        {
          name: "config";
          type: {
            defined: "GovernanceConfig";
          };
        }
      ];
    },
    {
      name: "flagTransactionError";
      accounts: [
        {
          name: "proposal";
          isMut: true;
          isSigner: false;
        },
        {
          name: "tokenOwnerRecord";
          isMut: false;
          isSigner: false;
        },
        {
          name: "governanceAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "proposalTransaction";
          isMut: true;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "setRealmAuthority";
      accounts: [
        {
          name: "realm";
          isMut: true;
          isSigner: false;
        },
        {
          name: "realmAuthority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [
        {
          name: "action";
          type: {
            defined: "SetRealmAuthorityAction";
          };
        }
      ];
    },
    {
      name: "setRealmConfig";
      accounts: [
        {
          name: "realm";
          isMut: true;
          isSigner: false;
        },
        {
          name: "realmAuthority";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [
        {
          name: "configArgs";
          type: {
            defined: "RealmConfigArgs";
          };
        }
      ];
    },
    {
      name: "createTokenOwnerRecord";
      accounts: [
        {
          name: "realm";
          isMut: false;
          isSigner: false;
        },
        {
          name: "governingTokenOwner";
          isMut: false;
          isSigner: false;
        },
        {
          name: "tokenOwnerRecordAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "governingTokenMint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "updateProgramMetadata";
      accounts: [
        {
          name: "programMetadataAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "createNativeTreasury";
      accounts: [
        {
          name: "governance";
          isMut: false;
          isSigner: false;
        },
        {
          name: "nativeTreasuryAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    }
  ];
  accounts: [
    {
      name: "realmV2";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "GovernanceAccountType";
            };
          },
          {
            name: "communityMint";
            type: "publicKey";
          },
          {
            name: "config";
            type: {
              defined: "RealmConfig";
            };
          },
          {
            name: "reserved";
            type: {
              array: ["u8", 6];
            };
          },
          {
            name: "votingProposalCount";
            type: "u16";
          },
          {
            name: "authority";
            type: {
              option: "publicKey";
            };
          },
          {
            name: "name";
            type: "string";
          },
          {
            name: "reservedV2";
            type: {
              array: ["u8", 128];
            };
          }
        ];
      };
    },
    {
      name: "proposalV2";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "GovernanceAccountType";
            };
          },
          {
            name: "governance";
            type: "publicKey";
          },
          {
            name: "governingTokenMint";
            type: "publicKey";
          },
          {
            name: "state";
            type: {
              defined: "ProposalState";
            };
          },
          {
            name: "tokenOwnerRecord";
            type: "publicKey";
          },
          {
            name: "signatoriesCount";
            type: "u8";
          },
          {
            name: "signatoriesSignedOffCount";
            type: "u8";
          },
          {
            name: "voteType";
            type: {
              defined: "VoteType";
            };
          },
          {
            name: "options";
            type: {
              vec: {
                defined: "ProposalOption";
              };
            };
          },
          {
            name: "denyVoteWeight";
            type: {
              option: "u64";
            };
          },
          {
            name: "reserved1";
            type: "u8";
          },
          {
            name: "abstainVoteWeight";
            type: {
              option: "u64";
            };
          },
          {
            name: "startVotingAt";
            type: {
              option: "i64";
            };
          },
          {
            name: "draftAt";
            type: "i64";
          },
          {
            name: "signingOffAt";
            type: {
              option: "i64";
            };
          },
          {
            name: "votingAt";
            type: {
              option: "i64";
            };
          },
          {
            name: "votingAtSlot";
            type: {
              option: "u64";
            };
          },
          {
            name: "votingCompletedAt";
            type: {
              option: "i64";
            };
          },
          {
            name: "executingAt";
            type: {
              option: "i64";
            };
          },
          {
            name: "closedAt";
            type: {
              option: "i64";
            };
          },
          {
            name: "executionFlags";
            type: {
              defined: "InstructionExecutionFlags";
            };
          },
          {
            name: "maxVoteWeight";
            type: {
              option: "u64";
            };
          },
          {
            name: "maxVotingTime";
            type: {
              option: "u32";
            };
          },
          {
            name: "voteThreshold";
            type: {
              option: {
                defined: "VoteThreshold";
              };
            };
          },
          {
            name: "reserved";
            type: {
              array: ["u8", 64];
            };
          },
          {
            name: "name";
            type: "string";
          },
          {
            name: "descriptionLink";
            type: "string";
          },
          {
            name: "vetoVoteWeight";
            type: "u64";
          }
        ];
      };
    },
    {
      name: "programMetadata";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "GovernanceAccountType";
            };
          },
          {
            name: "updatedAt";
            type: "u64";
          },
          {
            name: "version";
            type: "string";
          },
          {
            name: "reserved";
            type: {
              array: ["u8", 64];
            };
          }
        ];
      };
    },
    {
      name: "signatoryRecordV2";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "GovernanceAccountType";
            };
          },
          {
            name: "proposal";
            type: "publicKey";
          },
          {
            name: "signatory";
            type: "publicKey";
          },
          {
            name: "signedOff";
            type: "bool";
          },
          {
            name: "reservedV2";
            type: {
              array: ["u8", 8];
            };
          }
        ];
      };
    },
    {
      name: "realmV1";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "GovernanceAccountType";
            };
          },
          {
            name: "communityMint";
            type: "publicKey";
          },
          {
            name: "config";
            type: {
              defined: "RealmConfig";
            };
          },
          {
            name: "reserved";
            type: {
              array: ["u8", 6];
            };
          },
          {
            name: "votingProposalCount";
            type: "u16";
          },
          {
            name: "authority";
            type: {
              option: "publicKey";
            };
          },
          {
            name: "name";
            type: "string";
          }
        ];
      };
    },
    {
      name: "tokenOwnerRecordV1";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "GovernanceAccountType";
            };
          },
          {
            name: "realm";
            type: "publicKey";
          },
          {
            name: "governingTokenMint";
            type: "publicKey";
          },
          {
            name: "governingTokenOwner";
            type: "publicKey";
          },
          {
            name: "governingTokenDepositAmount";
            type: "u64";
          },
          {
            name: "unrelinquishedVotesCount";
            type: "u32";
          },
          {
            name: "totalVotesCount";
            type: "u32";
          },
          {
            name: "outstandingProposalCount";
            type: "u8";
          },
          {
            name: "reserved";
            type: {
              array: ["u8", 7];
            };
          },
          {
            name: "governanceDelegate";
            type: {
              option: "publicKey";
            };
          }
        ];
      };
    },
    {
      name: "governanceV1";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "GovernanceAccountType";
            };
          },
          {
            name: "realm";
            type: "publicKey";
          },
          {
            name: "governedAccount";
            type: "publicKey";
          },
          {
            name: "proposalsCount";
            type: "u32";
          },
          {
            name: "config";
            type: {
              defined: "GovernanceConfig";
            };
          },
          {
            name: "reserved";
            type: {
              array: ["u8", 6];
            };
          },
          {
            name: "votingProposalCount";
            type: "u16";
          }
        ];
      };
    },
    {
      name: "proposalV1";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "GovernanceAccountType";
            };
          },
          {
            name: "governance";
            type: "publicKey";
          },
          {
            name: "governingTokenMint";
            type: "publicKey";
          },
          {
            name: "state";
            type: {
              defined: "ProposalState";
            };
          },
          {
            name: "tokenOwnerRecord";
            type: "publicKey";
          },
          {
            name: "signatoriesCount";
            type: "u8";
          },
          {
            name: "signatoriesSignedOffCount";
            type: "u8";
          },
          {
            name: "yesVotesCount";
            type: "u64";
          },
          {
            name: "noVotesCount";
            type: "u64";
          },
          {
            name: "instructionsExecutedCount";
            type: "u16";
          },
          {
            name: "instructionsCount";
            type: "u16";
          },
          {
            name: "instructionsNextIndex";
            type: "u16";
          },
          {
            name: "draftAt";
            type: "i64";
          },
          {
            name: "signingOffAt";
            type: {
              option: "i64";
            };
          },
          {
            name: "votingAt";
            type: {
              option: "i64";
            };
          },
          {
            name: "votingAtSlot";
            type: {
              option: "u64";
            };
          },
          {
            name: "votingCompletedAt";
            type: {
              option: "i64";
            };
          },
          {
            name: "executingAt";
            type: {
              option: "i64";
            };
          },
          {
            name: "closedAt";
            type: {
              option: "i64";
            };
          },
          {
            name: "executionFlags";
            type: {
              defined: "InstructionExecutionFlags";
            };
          },
          {
            name: "maxVoteWeight";
            type: {
              option: "u64";
            };
          },
          {
            name: "voteThreshold";
            type: {
              option: {
                defined: "VoteThreshold";
              };
            };
          },
          {
            name: "name";
            type: "string";
          },
          {
            name: "descriptionLink";
            type: "string";
          }
        ];
      };
    },
    {
      name: "signatoryRecordV1";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "GovernanceAccountType";
            };
          },
          {
            name: "proposal";
            type: "publicKey";
          },
          {
            name: "signatory";
            type: "publicKey";
          },
          {
            name: "signedOff";
            type: "bool";
          }
        ];
      };
    },
    {
      name: "voteRecordV1";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "GovernanceAccountType";
            };
          },
          {
            name: "proposal";
            type: "publicKey";
          },
          {
            name: "governingTokenOwner";
            type: "publicKey";
          },
          {
            name: "isRelinquished";
            type: "bool";
          },
          {
            name: "voteWeight";
            type: {
              defined: "VoteWeightV1";
            };
          }
        ];
      };
    },
    {
      name: "governanceV2";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "GovernanceAccountType";
            };
          },
          {
            name: "realm";
            type: "publicKey";
          },
          {
            name: "governedAccount";
            type: "publicKey";
          },
          {
            name: "proposalsCount";
            type: "u32";
          },
          {
            name: "config";
            type: {
              defined: "GovernanceConfig";
            };
          },
          {
            name: "reserved";
            type: {
              array: ["u8", 6];
            };
          },
          {
            name: "votingProposalCount";
            type: "u16";
          },
          {
            name: "reservedV2";
            type: {
              array: ["u8", 128];
            };
          }
        ];
      };
    },
    {
      name: "voteRecordV2";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "GovernanceAccountType";
            };
          },
          {
            name: "proposal";
            type: "publicKey";
          },
          {
            name: "governingTokenOwner";
            type: "publicKey";
          },
          {
            name: "isRelinquished";
            type: "bool";
          },
          {
            name: "voterWeight";
            type: "u64";
          },
          {
            name: "vote";
            type: {
              defined: "Vote";
            };
          },
          {
            name: "reservedV2";
            type: {
              array: ["u8", 8];
            };
          }
        ];
      };
    },
    {
      name: "tokenOwnerRecordV2";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "GovernanceAccountType";
            };
          },
          {
            name: "realm";
            type: "publicKey";
          },
          {
            name: "governingTokenMint";
            type: "publicKey";
          },
          {
            name: "governingTokenOwner";
            type: "publicKey";
          },
          {
            name: "governingTokenDepositAmount";
            type: "u64";
          },
          {
            name: "unrelinquishedVotesCount";
            type: "u32";
          },
          {
            name: "totalVotesCount";
            type: "u32";
          },
          {
            name: "outstandingProposalCount";
            type: "u8";
          },
          {
            name: "reserved";
            type: {
              array: ["u8", 7];
            };
          },
          {
            name: "governanceDelegate";
            type: {
              option: "publicKey";
            };
          },
          {
            name: "reservedV2";
            type: {
              array: ["u8", 128];
            };
          }
        ];
      };
    },
    {
      name: "realmConfigAccount";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "GovernanceAccountType";
            };
          },
          {
            name: "realm";
            type: "publicKey";
          },
          {
            name: "communityVoterWeightAddin";
            type: {
              option: "publicKey";
            };
          },
          {
            name: "maxCommunityVoterWeightAddin";
            type: {
              option: "publicKey";
            };
          },
          {
            name: "councilVoterWeightAddin";
            type: {
              option: "publicKey";
            };
          },
          {
            name: "councilMaxVoteWeightAddin";
            type: {
              option: "publicKey";
            };
          },
          {
            name: "reserved";
            type: {
              array: ["u8", 128];
            };
          }
        ];
      };
    },
    {
      name: "proposalTransactionV2";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "GovernanceAccountType";
            };
          },
          {
            name: "proposal";
            type: "publicKey";
          },
          {
            name: "optionIndex";
            type: "u8";
          },
          {
            name: "transactionIndex";
            type: "u16";
          },
          {
            name: "holdUpTime";
            type: "u32";
          },
          {
            name: "instructions";
            type: {
              vec: {
                defined: "InstructionData";
              };
            };
          },
          {
            name: "executedAt";
            type: {
              option: "i64";
            };
          },
          {
            name: "executionStatus";
            type: {
              defined: "TransactionExecutionStatus";
            };
          },
          {
            name: "reservedV2";
            type: {
              array: ["u8", 8];
            };
          }
        ];
      };
    }
  ];
  types: [
    {
      name: "NativeTreasury";
      type: {
        kind: "struct";
        fields: [];
      };
    },
    {
      name: "RealmConfigArgs";
      type: {
        kind: "struct";
        fields: [
          {
            name: "useCouncilMint";
            type: "bool";
          },
          {
            name: "minCommunityWeightToCreateGovernance";
            type: "u64";
          },
          {
            name: "communityMintMaxVoteWeightSource";
            type: {
              defined: "MintMaxVoteWeightSource";
            };
          },
          {
            name: "useCommunityVoterWeightAddin";
            type: "bool";
          },
          {
            name: "useMaxCommunityVoterWeightAddin";
            type: "bool";
          }
        ];
      };
    },
    {
      name: "RealmConfig";
      type: {
        kind: "struct";
        fields: [
          {
            name: "useCommunityVoterWeightAddin";
            type: "bool";
          },
          {
            name: "useMaxCommunityVoterWeightAddin";
            type: "bool";
          },
          {
            name: "reserved";
            type: {
              array: ["u8", 6];
            };
          },
          {
            name: "minCommunityWeightToCreateGovernance";
            type: "u64";
          },
          {
            name: "communityMintMaxVoteWeightSource";
            type: {
              defined: "MintMaxVoteWeightSource";
            };
          },
          {
            name: "councilMint";
            type: {
              option: "publicKey";
            };
          }
        ];
      };
    },
    {
      name: "ProposalOption";
      type: {
        kind: "struct";
        fields: [
          {
            name: "label";
            type: "string";
          },
          {
            name: "voteWeight";
            type: "u64";
          },
          {
            name: "voteResult";
            type: {
              defined: "OptionVoteResult";
            };
          },
          {
            name: "transactionsExecutedCount";
            type: "u16";
          },
          {
            name: "transactionsCount";
            type: "u16";
          },
          {
            name: "transactionsNextIndex";
            type: "u16";
          }
        ];
      };
    },
    {
      name: "GovernanceConfig";
      type: {
        kind: "struct";
        fields: [
          {
            name: "communityVoteThreshold";
            type: {
              defined: "VoteThreshold";
            };
          },
          {
            name: "minCommunityWeightToCreateProposal";
            type: "u64";
          },
          {
            name: "minTransactionHoldUpTime";
            type: "u32";
          },
          {
            name: "maxVotingTime";
            type: "u32";
          },
          {
            name: "voteTipping";
            type: {
              defined: "VoteTipping";
            };
          },
          {
            name: "councilVoteThreshold";
            type: {
              defined: "VoteThreshold";
            };
          },
          {
            name: "councilVetoVoteThreshold";
            type: {
              defined: "VoteThreshold";
            };
          },
          {
            name: "minCouncilWeightToCreateProposal";
            type: "u64";
          }
        ];
      };
    },
    {
      name: "AccountMetaData";
      type: {
        kind: "struct";
        fields: [
          {
            name: "pubkey";
            type: "publicKey";
          },
          {
            name: "isSigner";
            type: "bool";
          },
          {
            name: "isWritable";
            type: "bool";
          }
        ];
      };
    },
    {
      name: "InstructionData";
      type: {
        kind: "struct";
        fields: [
          {
            name: "programId";
            type: "publicKey";
          },
          {
            name: "accounts";
            type: {
              vec: {
                defined: "AccountMetaData";
              };
            };
          },
          {
            name: "data";
            type: "bytes";
          }
        ];
      };
    },
    {
      name: "ProposalInstructionV1";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "GovernanceAccountType";
            };
          },
          {
            name: "proposal";
            type: "publicKey";
          },
          {
            name: "instructionIndex";
            type: "u16";
          },
          {
            name: "holdUpTime";
            type: "u32";
          },
          {
            name: "instruction";
            type: {
              defined: "InstructionData";
            };
          },
          {
            name: "executedAt";
            type: {
              option: "i64";
            };
          },
          {
            name: "executionStatus";
            type: {
              defined: "TransactionExecutionStatus";
            };
          }
        ];
      };
    },
    {
      name: "VoteChoice";
      type: {
        kind: "struct";
        fields: [
          {
            name: "rank";
            type: "u8";
          },
          {
            name: "weightPercentage";
            type: "u8";
          }
        ];
      };
    },
    {
      name: "MintMaxVoteWeightSource";
      type: {
        kind: "enum";
        variants: [
          {
            name: "SupplyFraction";
            fields: ["u64"];
          },
          {
            name: "Absolute";
            fields: ["u64"];
          }
        ];
      };
    },
    {
      name: "GovernanceAccountType";
      type: {
        kind: "enum";
        variants: [
          {
            name: "Uninitialized";
          },
          {
            name: "RealmV1";
          },
          {
            name: "TokenOwnerRecordV1";
          },
          {
            name: "GovernanceV1";
          },
          {
            name: "ProgramGovernanceV1";
          },
          {
            name: "ProposalV1";
          },
          {
            name: "SignatoryRecordV1";
          },
          {
            name: "VoteRecordV1";
          },
          {
            name: "ProposalInstructionV1";
          },
          {
            name: "MintGovernanceV1";
          },
          {
            name: "TokenGovernanceV1";
          },
          {
            name: "RealmConfig";
          },
          {
            name: "VoteRecordV2";
          },
          {
            name: "ProposalTransactionV2";
          },
          {
            name: "ProposalV2";
          },
          {
            name: "ProgramMetadata";
          },
          {
            name: "RealmV2";
          },
          {
            name: "TokenOwnerRecordV2";
          },
          {
            name: "GovernanceV2";
          },
          {
            name: "ProgramGovernanceV2";
          },
          {
            name: "MintGovernanceV2";
          },
          {
            name: "TokenGovernanceV2";
          },
          {
            name: "SignatoryRecordV2";
          }
        ];
      };
    },
    {
      name: "OptionVoteResult";
      type: {
        kind: "enum";
        variants: [
          {
            name: "None";
          },
          {
            name: "Succeeded";
          },
          {
            name: "Defeated";
          }
        ];
      };
    },
    {
      name: "ProposalState";
      type: {
        kind: "enum";
        variants: [
          {
            name: "Draft";
          },
          {
            name: "SigningOff";
          },
          {
            name: "Voting";
          },
          {
            name: "Succeeded";
          },
          {
            name: "Executing";
          },
          {
            name: "Completed";
          },
          {
            name: "Cancelled";
          },
          {
            name: "Defeated";
          },
          {
            name: "ExecutingWithErrors";
          },
          {
            name: "Vetoed";
          }
        ];
      };
    },
    {
      name: "VoteType";
      type: {
        kind: "enum";
        variants: [
          {
            name: "SingleChoice";
          },
          {
            name: "MultiChoice";
            fields: [
              {
                name: "max_voter_options";
                type: "u8";
              },
              {
                name: "max_winning_options";
                type: "u8";
              }
            ];
          }
        ];
      };
    },
    {
      name: "InstructionExecutionFlags";
      type: {
        kind: "enum";
        variants: [
          {
            name: "None";
          },
          {
            name: "Ordered";
          },
          {
            name: "UseTransaction";
          }
        ];
      };
    },
    {
      name: "VoteThreshold";
      type: {
        kind: "enum";
        variants: [
          {
            name: "YesVotePercentage";
            fields: ["u8"];
          },
          {
            name: "QuorumPercentage";
            fields: ["u8"];
          },
          {
            name: "Disabled";
          }
        ];
      };
    },
    {
      name: "VoteTipping";
      type: {
        kind: "enum";
        variants: [
          {
            name: "Strict";
          },
          {
            name: "Early";
          },
          {
            name: "Disabled";
          }
        ];
      };
    },
    {
      name: "TransactionExecutionStatus";
      type: {
        kind: "enum";
        variants: [
          {
            name: "None";
          },
          {
            name: "Success";
          },
          {
            name: "Error";
          }
        ];
      };
    },
    {
      name: "VoteWeightV1";
      type: {
        kind: "enum";
        variants: [
          {
            name: "Yes";
            fields: ["u64"];
          },
          {
            name: "No";
            fields: ["u64"];
          }
        ];
      };
    },
    {
      name: "Vote";
      type: {
        kind: "enum";
        variants: [
          {
            name: "Approve";
            fields: [
              {
                vec: {
                  defined: "VoteChoice";
                };
              }
            ];
          },
          {
            name: "Deny";
          },
          {
            name: "Abstain";
          },
          {
            name: "Veto";
          }
        ];
      };
    },
    {
      name: "SetRealmAuthorityAction";
      type: {
        kind: "enum";
        variants: [
          {
            name: "SetUnchecked";
          },
          {
            name: "SetChecked";
          },
          {
            name: "Remove";
          }
        ];
      };
    }
  ];
  errors: [
    {
      code: 500;
      name: "InvalidInstruction";
      msg: "Invalid instruction passed to program";
    },
    {
      code: 501;
      name: "RealmAlreadyExists";
      msg: "Realm with the given name and governing mints already exists";
    },
    {
      code: 502;
      name: "InvalidRealm";
      msg: "Invalid realm";
    },
    {
      code: 503;
      name: "InvalidGoverningTokenMint";
      msg: "Invalid Governing Token Mint";
    },
    {
      code: 504;
      name: "GoverningTokenOwnerMustSign";
      msg: "Governing Token Owner must sign transaction";
    },
    {
      code: 505;
      name: "GoverningTokenOwnerOrDelegateMustSign";
      msg: "Governing Token Owner or Delegate  must sign transaction";
    },
    {
      code: 506;
      name: "AllVotesMustBeRelinquishedToWithdrawGoverningTokens";
      msg: "All votes must be relinquished to withdraw governing tokens";
    },
    {
      code: 507;
      name: "InvalidTokenOwnerRecordAccountAddress";
      msg: "Invalid Token Owner Record account address";
    },
    {
      code: 508;
      name: "InvalidGoverningMintForTokenOwnerRecord";
      msg: "Invalid GoverningMint for TokenOwnerRecord";
    },
    {
      code: 509;
      name: "InvalidRealmForTokenOwnerRecord";
      msg: "Invalid Realm for TokenOwnerRecord";
    },
    {
      code: 510;
      name: "InvalidProposalForProposalTransaction";
      msg: "Invalid Proposal for ProposalTransaction,";
    },
    {
      code: 511;
      name: "InvalidSignatoryAddress";
      msg: "Invalid Signatory account address";
    },
    {
      code: 512;
      name: "SignatoryAlreadySignedOff";
      msg: "Signatory already signed off";
    },
    {
      code: 513;
      name: "SignatoryMustSign";
      msg: "Signatory must sign";
    },
    {
      code: 514;
      name: "InvalidProposalOwnerAccount";
      msg: "Invalid Proposal Owner";
    },
    {
      code: 515;
      name: "InvalidProposalForVoterRecord";
      msg: "Invalid Proposal for VoterRecord";
    },
    {
      code: 516;
      name: "InvalidGoverningTokenOwnerForVoteRecord";
      msg: "Invalid GoverningTokenOwner for VoteRecord";
    },
    {
      code: 517;
      name: "InvalidVoteThresholdPercentage";
      msg: "Invalid Governance config: Vote threshold percentage out of range";
    },
    {
      code: 518;
      name: "ProposalAlreadyExists";
      msg: "Proposal for the given Governance, Governing Token Mint and index already exists";
    },
    {
      code: 519;
      name: "VoteAlreadyExists";
      msg: "Token Owner already voted on the Proposal";
    },
    {
      code: 520;
      name: "NotEnoughTokensToCreateProposal";
      msg: "Owner doesn't have enough governing tokens to create Proposal";
    },
    {
      code: 521;
      name: "InvalidStateCannotEditSignatories";
      msg: "Invalid State: Can't edit Signatories";
    },
    {
      code: 522;
      name: "InvalidProposalState";
      msg: "Invalid Proposal state";
    },
    {
      code: 523;
      name: "InvalidStateCannotEditTransactions";
      msg: "Invalid State: Can't edit transactions";
    },
    {
      code: 524;
      name: "InvalidStateCannotExecuteTransaction";
      msg: "Invalid State: Can't execute transaction";
    },
    {
      code: 525;
      name: "CannotExecuteTransactionWithinHoldUpTime";
      msg: "Can't execute transaction within its hold up time";
    },
    {
      code: 526;
      name: "TransactionAlreadyExecuted";
      msg: "Transaction already executed";
    },
    {
      code: 527;
      name: "InvalidTransactionIndex";
      msg: "Invalid Transaction index";
    },
    {
      code: 528;
      name: "TransactionHoldUpTimeBelowRequiredMin";
      msg: "Transaction hold up time is below the min specified by Governance";
    },
    {
      code: 529;
      name: "TransactionAlreadyExists";
      msg: "Transaction at the given index for the Proposal already exists";
    },
    {
      code: 530;
      name: "InvalidStateCannotSignOff";
      msg: "Invalid State: Can't sign off";
    },
    {
      code: 531;
      name: "InvalidStateCannotVote";
      msg: "Invalid State: Can't vote";
    },
    {
      code: 532;
      name: "InvalidStateCannotFinalize";
      msg: "Invalid State: Can't finalize vote";
    },
    {
      code: 533;
      name: "InvalidStateCannotCancelProposal";
      msg: "Invalid State: Can't cancel Proposal";
    },
    {
      code: 534;
      name: "VoteAlreadyRelinquished";
      msg: "Vote already relinquished";
    },
    {
      code: 535;
      name: "CannotFinalizeVotingInProgress";
      msg: "Can't finalize vote. Voting still in progress";
    },
    {
      code: 536;
      name: "ProposalVotingTimeExpired";
      msg: "Proposal voting time expired";
    },
    {
      code: 537;
      name: "InvalidSignatoryMint";
      msg: "Invalid Signatory Mint";
    },
    {
      code: 538;
      name: "InvalidGovernanceForProposal";
      msg: "Proposal does not belong to the given Governance";
    },
    {
      code: 539;
      name: "InvalidGoverningMintForProposal";
      msg: "Proposal does not belong to given Governing Mint";
    },
    {
      code: 540;
      name: "MintAuthorityMustSign";
      msg: "Current mint authority must sign transaction";
    },
    {
      code: 541;
      name: "InvalidMintAuthority";
      msg: "Invalid mint authority";
    },
    {
      code: 542;
      name: "MintHasNoAuthority";
      msg: "Mint has no authority";
    },
    {
      code: 543;
      name: "SplTokenAccountWithInvalidOwner";
      msg: "Invalid Token account owner";
    },
    {
      code: 544;
      name: "SplTokenMintWithInvalidOwner";
      msg: "Invalid Mint account owner";
    },
    {
      code: 545;
      name: "SplTokenAccountNotInitialized";
      msg: "Token Account is not initialized";
    },
    {
      code: 546;
      name: "SplTokenAccountDoesNotExist";
      msg: "Token Account doesn't exist";
    },
    {
      code: 547;
      name: "SplTokenInvalidTokenAccountData";
      msg: "Token account data is invalid";
    },
    {
      code: 548;
      name: "SplTokenInvalidMintAccountData";
      msg: "Token mint account data is invalid";
    },
    {
      code: 549;
      name: "SplTokenMintNotInitialized";
      msg: "Token Mint account is not initialized";
    },
    {
      code: 550;
      name: "SplTokenMintDoesNotExist";
      msg: "Token Mint account doesn't exist";
    },
    {
      code: 551;
      name: "InvalidProgramDataAccountAddress";
      msg: "Invalid ProgramData account address";
    },
    {
      code: 552;
      name: "InvalidProgramDataAccountData";
      msg: "Invalid ProgramData account Data";
    },
    {
      code: 553;
      name: "InvalidUpgradeAuthority";
      msg: "Provided upgrade authority doesn't match current program upgrade authority";
    },
    {
      code: 554;
      name: "UpgradeAuthorityMustSign";
      msg: "Current program upgrade authority must sign transaction";
    },
    {
      code: 555;
      name: "ProgramNotUpgradable";
      msg: "Given program is not upgradable";
    },
    {
      code: 556;
      name: "InvalidTokenOwner";
      msg: "Invalid token owner";
    },
    {
      code: 557;
      name: "TokenOwnerMustSign";
      msg: "Current token owner must sign transaction";
    },
    {
      code: 558;
      name: "VoteThresholdTypeNotSupported";
      msg: "Given VoteThresholdType is not supported";
    },
    {
      code: 559;
      name: "VoteWeightSourceNotSupported";
      msg: "Given VoteWeightSource is not supported";
    },
    {
      code: 560;
      name: "GoverningTokenMintNotAllowedToVote";
      msg: "GoverningTokenMint not allowed to vote";
    },
    {
      code: 561;
      name: "GovernancePdaMustSign";
      msg: "Governance PDA must sign";
    },
    {
      code: 562;
      name: "TransactionAlreadyFlaggedWithError";
      msg: "Transaction already flagged with error";
    },
    {
      code: 563;
      name: "InvalidRealmForGovernance";
      msg: "Invalid Realm for Governance";
    },
    {
      code: 564;
      name: "InvalidAuthorityForRealm";
      msg: "Invalid Authority for Realm";
    },
    {
      code: 565;
      name: "RealmHasNoAuthority";
      msg: "Realm has no authority";
    },
    {
      code: 566;
      name: "RealmAuthorityMustSign";
      msg: "Realm authority must sign";
    },
    {
      code: 567;
      name: "InvalidGoverningTokenHoldingAccount";
      msg: "Invalid governing token holding account";
    },
    {
      code: 568;
      name: "RealmCouncilMintChangeIsNotSupported";
      msg: "Realm council mint change is not supported";
    },
    {
      code: 569;
      name: "MintMaxVoteWeightSourceNotSupported";
      msg: "Not supported mint max vote weight source";
    },
    {
      code: 570;
      name: "InvalidMaxVoteWeightSupplyFraction";
      msg: "Invalid max vote weight supply fraction";
    },
    {
      code: 571;
      name: "NotEnoughTokensToCreateGovernance";
      msg: "Owner doesn't have enough governing tokens to create Governance";
    },
    {
      code: 572;
      name: "TooManyOutstandingProposals";
      msg: "Too many outstanding proposals";
    },
    {
      code: 573;
      name: "AllProposalsMustBeFinalisedToWithdrawGoverningTokens";
      msg: "All proposals must be finalized to withdraw governing tokens";
    },
    {
      code: 574;
      name: "InvalidVoterWeightRecordForRealm";
      msg: "Invalid VoterWeightRecord for Realm";
    },
    {
      code: 575;
      name: "InvalidVoterWeightRecordForGoverningTokenMint";
      msg: "Invalid VoterWeightRecord for GoverningTokenMint";
    },
    {
      code: 576;
      name: "InvalidVoterWeightRecordForTokenOwner";
      msg: "Invalid VoterWeightRecord for TokenOwner";
    },
    {
      code: 577;
      name: "VoterWeightRecordExpired";
      msg: "VoterWeightRecord expired";
    },
    {
      code: 578;
      name: "InvalidRealmConfigForRealm";
      msg: "Invalid RealmConfig for Realm";
    },
    {
      code: 579;
      name: "TokenOwnerRecordAlreadyExists";
      msg: "TokenOwnerRecord already exists";
    },
    {
      code: 580;
      name: "GoverningTokenDepositsNotAllowed";
      msg: "Governing token deposits not allowed";
    },
    {
      code: 581;
      name: "InvalidVoteChoiceWeightPercentage";
      msg: "Invalid vote choice weight percentage";
    },
    {
      code: 582;
      name: "VoteTypeNotSupported";
      msg: "Vote type not supported";
    },
    {
      code: 583;
      name: "InvalidProposalOptions";
      msg: "Invalid proposal options";
    },
    {
      code: 584;
      name: "ProposalIsNotExecutable";
      msg: "Proposal is not not executable";
    },
    {
      code: 585;
      name: "InvalidVote";
      msg: "Invalid vote";
    },
    {
      code: 586;
      name: "CannotExecuteDefeatedOption";
      msg: "Cannot execute defeated option";
    },
    {
      code: 587;
      name: "VoterWeightRecordInvalidAction";
      msg: "VoterWeightRecord invalid action";
    },
    {
      code: 588;
      name: "VoterWeightRecordInvalidActionTarget";
      msg: "VoterWeightRecord invalid action target";
    },
    {
      code: 589;
      name: "InvalidMaxVoterWeightRecordForRealm";
      msg: "Invalid MaxVoterWeightRecord for Realm";
    },
    {
      code: 590;
      name: "InvalidMaxVoterWeightRecordForGoverningTokenMint";
      msg: "Invalid MaxVoterWeightRecord for GoverningTokenMint";
    },
    {
      code: 591;
      name: "MaxVoterWeightRecordExpired";
      msg: "MaxVoterWeightRecord expired";
    },
    {
      code: 592;
      name: "NotSupportedVoteType";
      msg: "Not supported VoteType";
    },
    {
      code: 593;
      name: "RealmConfigChangeNotAllowed";
      msg: "RealmConfig change not allowed";
    },
    {
      code: 594;
      name: "GovernanceConfigChangeNotAllowed";
      msg: "GovernanceConfig change not allowed";
    },
    {
      code: 595;
      name: "AtLeastOneVoteThresholdRequired";
      msg: "At least one VoteThreshold is required";
    },
    {
      code: 596;
      name: "ReservedBufferMustBeEmpty";
      msg: "Reserved buffer must be empty";
    },
    {
      code: 597;
      name: "CannotRelinquishInFinalizingState";
      msg: "Cannot Relinquish in Finalizing state";
    }
  ];
};

const IDL: SplGovernance = {
  version: "3.0.0",
  name: "spl_governance",
  instructions: [
    {
      name: "createRealm",
      accounts: [
        {
          name: "realmAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "realmAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "communityTokenMint",
          isMut: false,
          isSigner: false,
        },
        {
          name: "communityTokenHoldingAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "tokenProgram",
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
          name: "name",
          type: "string",
        },
        {
          name: "configArgs",
          type: {
            defined: "RealmConfigArgs",
          },
        },
      ],
    },
    {
      name: "depositGoverningTokens",
      accounts: [
        {
          name: "realm",
          isMut: false,
          isSigner: false,
        },
        {
          name: "governingTokenHoldingAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governingTokenSource",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governingTokenOwner",
          isMut: false,
          isSigner: true,
        },
        {
          name: "governingTokenTransferAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "tokenOwnerRecordAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "systemProgram",
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
          name: "amount",
          type: "u64",
        },
      ],
    },
    {
      name: "withdrawGoverningTokens",
      accounts: [
        {
          name: "realm",
          isMut: false,
          isSigner: false,
        },
        {
          name: "governingTokenHoldingAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governingTokenDestination",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governingTokenOwner",
          isMut: false,
          isSigner: true,
        },
        {
          name: "tokenOwnerRecordAddress",
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
    {
      name: "setGovernanceDelegate",
      accounts: [
        {
          name: "governanceAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "voteRecordAddress",
          isMut: true,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "newGovernanceDelegate",
          type: {
            option: "publicKey",
          },
        },
      ],
    },
    {
      name: "createGovernance",
      accounts: [
        {
          name: "realm",
          isMut: false,
          isSigner: false,
        },
        {
          name: "governanceAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governedAccountAddress",
          isMut: false,
          isSigner: false,
        },
        {
          name: "tokenOwnerRecord",
          isMut: false,
          isSigner: false,
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "createAuthority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [
        {
          name: "config",
          type: {
            defined: "GovernanceConfig",
          },
        },
      ],
    },
    {
      name: "createProgramGovernance",
      accounts: [
        {
          name: "realm",
          isMut: false,
          isSigner: false,
        },
        {
          name: "programGovernanceAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governedProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "governedProgramDataAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governedProgramUpgradeAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "tokenOwnerRecord",
          isMut: false,
          isSigner: false,
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "bpfLoaderUpgradeable",
          isMut: false,
          isSigner: false,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "createAuthority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [
        {
          name: "config",
          type: {
            defined: "GovernanceConfig",
          },
        },
        {
          name: "transferUpgradeAuthority",
          type: "bool",
        },
      ],
    },
    {
      name: "createProposal",
      accounts: [
        {
          name: "realm",
          isMut: false,
          isSigner: false,
        },
        {
          name: "proposalAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governance",
          isMut: true,
          isSigner: false,
        },
        {
          name: "proposalOwnerRecord",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governingTokenMint",
          isMut: false,
          isSigner: false,
        },
        {
          name: "governanceAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "name",
          type: "string",
        },
        {
          name: "descriptionLink",
          type: "string",
        },
        {
          name: "voteType",
          type: {
            defined: "VoteType",
          },
        },
        {
          name: "options",
          type: {
            vec: "string",
          },
        },
        {
          name: "useDenyOption",
          type: "bool",
        },
      ],
    },
    {
      name: "addSignatory",
      accounts: [
        {
          name: "proposal",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenOwnerRecord",
          isMut: false,
          isSigner: false,
        },
        {
          name: "governanceAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "signatoryRecordAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "signatory",
          type: "publicKey",
        },
      ],
    },
    {
      name: "removeSignatory",
      accounts: [
        {
          name: "proposal",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenOwnerRecord",
          isMut: false,
          isSigner: false,
        },
        {
          name: "governanceAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "signatoryRecordAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "beneficiary",
          isMut: true,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "signatory",
          type: "publicKey",
        },
      ],
    },
    {
      name: "insertTransaction",
      accounts: [
        {
          name: "governance",
          isMut: false,
          isSigner: false,
        },
        {
          name: "proposal",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenOwnerRecord",
          isMut: false,
          isSigner: false,
        },
        {
          name: "governanceAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "proposalTransactionAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
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
          name: "optionIndex",
          type: "u8",
        },
        {
          name: "index",
          type: "u16",
        },
        {
          name: "holdUpTime",
          type: "u32",
        },
        {
          name: "instructions",
          type: {
            vec: {
              defined: "InstructionData",
            },
          },
        },
      ],
    },
    {
      name: "removeTransaction",
      accounts: [
        {
          name: "proposal",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenOwnerRecord",
          isMut: false,
          isSigner: false,
        },
        {
          name: "governanceAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "proposalTransaction",
          isMut: true,
          isSigner: false,
        },
        {
          name: "beneficiary",
          isMut: true,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "cancelProposal",
      accounts: [
        {
          name: "realm",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governance",
          isMut: true,
          isSigner: false,
        },
        {
          name: "proposal",
          isMut: true,
          isSigner: false,
        },
        {
          name: "proposalOwnerRecord",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governanceAuthority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [],
    },
    {
      name: "signOffProposal",
      accounts: [
        {
          name: "realm",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governance",
          isMut: true,
          isSigner: false,
        },
        {
          name: "proposal",
          isMut: true,
          isSigner: false,
        },
        {
          name: "signatory",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [],
    },
    {
      name: "castVote",
      accounts: [
        {
          name: "realm",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governance",
          isMut: true,
          isSigner: false,
        },
        {
          name: "proposal",
          isMut: true,
          isSigner: false,
        },
        {
          name: "proposalOwnerRecord",
          isMut: true,
          isSigner: false,
        },
        {
          name: "voterTokenOwnerRecord",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governanceAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "voteRecordAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "voteGoverningTokenMint",
          isMut: false,
          isSigner: false,
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "vote",
          type: {
            defined: "Vote",
          },
        },
      ],
    },
    {
      name: "finalizeVote",
      accounts: [
        {
          name: "realm",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governance",
          isMut: true,
          isSigner: false,
        },
        {
          name: "proposal",
          isMut: true,
          isSigner: false,
        },
        {
          name: "proposalOwnerRecord",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governingTokenMint",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "relinquishVote",
      accounts: [
        {
          name: "realm",
          isMut: false,
          isSigner: false,
        },
        {
          name: "governance",
          isMut: false,
          isSigner: false,
        },
        {
          name: "proposal",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenOwnerRecord",
          isMut: true,
          isSigner: false,
        },
        {
          name: "voteRecordAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "voteGoverningTokenMint",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "executeTransaction",
      accounts: [
        {
          name: "governance",
          isMut: false,
          isSigner: false,
        },
        {
          name: "proposal",
          isMut: true,
          isSigner: false,
        },
        {
          name: "proposalTransaction",
          isMut: true,
          isSigner: false,
        },
        {
          name: "instructionProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "createMintGovernance",
      accounts: [
        {
          name: "realm",
          isMut: false,
          isSigner: false,
        },
        {
          name: "mintGovernanceAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governedMint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governedMintAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "tokenOwnerRecord",
          isMut: false,
          isSigner: false,
        },
        {
          name: "payer",
          isMut: true,
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
          name: "createAuthority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [
        {
          name: "config",
          type: {
            defined: "GovernanceConfig",
          },
        },
        {
          name: "transferMintAuthorities",
          type: "bool",
        },
      ],
    },
    {
      name: "createTokenGovernance",
      accounts: [
        {
          name: "realm",
          isMut: false,
          isSigner: false,
        },
        {
          name: "tokenGovernanceAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governedToken",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governedTokenOwner",
          isMut: false,
          isSigner: true,
        },
        {
          name: "tokenOwnerRecord",
          isMut: false,
          isSigner: false,
        },
        {
          name: "payer",
          isMut: true,
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
          name: "createAuthority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [
        {
          name: "config",
          type: {
            defined: "GovernanceConfig",
          },
        },
        {
          name: "transferAccountAuthorities",
          type: "bool",
        },
      ],
    },
    {
      name: "setGovernanceConfig",
      accounts: [
        {
          name: "governance",
          isMut: true,
          isSigner: true,
        },
      ],
      args: [
        {
          name: "config",
          type: {
            defined: "GovernanceConfig",
          },
        },
      ],
    },
    {
      name: "flagTransactionError",
      accounts: [
        {
          name: "proposal",
          isMut: true,
          isSigner: false,
        },
        {
          name: "tokenOwnerRecord",
          isMut: false,
          isSigner: false,
        },
        {
          name: "governanceAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "proposalTransaction",
          isMut: true,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "setRealmAuthority",
      accounts: [
        {
          name: "realm",
          isMut: true,
          isSigner: false,
        },
        {
          name: "realmAuthority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [
        {
          name: "action",
          type: {
            defined: "SetRealmAuthorityAction",
          },
        },
      ],
    },
    {
      name: "setRealmConfig",
      accounts: [
        {
          name: "realm",
          isMut: true,
          isSigner: false,
        },
        {
          name: "realmAuthority",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [
        {
          name: "configArgs",
          type: {
            defined: "RealmConfigArgs",
          },
        },
      ],
    },
    {
      name: "createTokenOwnerRecord",
      accounts: [
        {
          name: "realm",
          isMut: false,
          isSigner: false,
        },
        {
          name: "governingTokenOwner",
          isMut: false,
          isSigner: false,
        },
        {
          name: "tokenOwnerRecordAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "governingTokenMint",
          isMut: false,
          isSigner: false,
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "updateProgramMetadata",
      accounts: [
        {
          name: "programMetadataAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "createNativeTreasury",
      accounts: [
        {
          name: "governance",
          isMut: false,
          isSigner: false,
        },
        {
          name: "nativeTreasuryAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
  ],
  accounts: [
    {
      name: "realmV2",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "GovernanceAccountType",
            },
          },
          {
            name: "communityMint",
            type: "publicKey",
          },
          {
            name: "config",
            type: {
              defined: "RealmConfig",
            },
          },
          {
            name: "reserved",
            type: {
              array: ["u8", 6],
            },
          },
          {
            name: "votingProposalCount",
            type: "u16",
          },
          {
            name: "authority",
            type: {
              option: "publicKey",
            },
          },
          {
            name: "name",
            type: "string",
          },
          {
            name: "reservedV2",
            type: {
              array: ["u8", 128],
            },
          },
        ],
      },
    },
    {
      name: "proposalV2",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "GovernanceAccountType",
            },
          },
          {
            name: "governance",
            type: "publicKey",
          },
          {
            name: "governingTokenMint",
            type: "publicKey",
          },
          {
            name: "state",
            type: {
              defined: "ProposalState",
            },
          },
          {
            name: "tokenOwnerRecord",
            type: "publicKey",
          },
          {
            name: "signatoriesCount",
            type: "u8",
          },
          {
            name: "signatoriesSignedOffCount",
            type: "u8",
          },
          {
            name: "voteType",
            type: {
              defined: "VoteType",
            },
          },
          {
            name: "options",
            type: {
              vec: {
                defined: "ProposalOption",
              },
            },
          },
          {
            name: "denyVoteWeight",
            type: {
              option: "u64",
            },
          },
          {
            name: "reserved1",
            type: "u8",
          },
          {
            name: "abstainVoteWeight",
            type: {
              option: "u64",
            },
          },
          {
            name: "startVotingAt",
            type: {
              option: "i64",
            },
          },
          {
            name: "draftAt",
            type: "i64",
          },
          {
            name: "signingOffAt",
            type: {
              option: "i64",
            },
          },
          {
            name: "votingAt",
            type: {
              option: "i64",
            },
          },
          {
            name: "votingAtSlot",
            type: {
              option: "u64",
            },
          },
          {
            name: "votingCompletedAt",
            type: {
              option: "i64",
            },
          },
          {
            name: "executingAt",
            type: {
              option: "i64",
            },
          },
          {
            name: "closedAt",
            type: {
              option: "i64",
            },
          },
          {
            name: "executionFlags",
            type: {
              defined: "InstructionExecutionFlags",
            },
          },
          {
            name: "maxVoteWeight",
            type: {
              option: "u64",
            },
          },
          {
            name: "maxVotingTime",
            type: {
              option: "u32",
            },
          },
          {
            name: "voteThreshold",
            type: {
              option: {
                defined: "VoteThreshold",
              },
            },
          },
          {
            name: "reserved",
            type: {
              array: ["u8", 64],
            },
          },
          {
            name: "name",
            type: "string",
          },
          {
            name: "descriptionLink",
            type: "string",
          },
          {
            name: "vetoVoteWeight",
            type: "u64",
          },
        ],
      },
    },
    {
      name: "programMetadata",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "GovernanceAccountType",
            },
          },
          {
            name: "updatedAt",
            type: "u64",
          },
          {
            name: "version",
            type: "string",
          },
          {
            name: "reserved",
            type: {
              array: ["u8", 64],
            },
          },
        ],
      },
    },
    {
      name: "signatoryRecordV2",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "GovernanceAccountType",
            },
          },
          {
            name: "proposal",
            type: "publicKey",
          },
          {
            name: "signatory",
            type: "publicKey",
          },
          {
            name: "signedOff",
            type: "bool",
          },
          {
            name: "reservedV2",
            type: {
              array: ["u8", 8],
            },
          },
        ],
      },
    },
    {
      name: "realmV1",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "GovernanceAccountType",
            },
          },
          {
            name: "communityMint",
            type: "publicKey",
          },
          {
            name: "config",
            type: {
              defined: "RealmConfig",
            },
          },
          {
            name: "reserved",
            type: {
              array: ["u8", 6],
            },
          },
          {
            name: "votingProposalCount",
            type: "u16",
          },
          {
            name: "authority",
            type: {
              option: "publicKey",
            },
          },
          {
            name: "name",
            type: "string",
          },
        ],
      },
    },
    {
      name: "tokenOwnerRecordV1",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "GovernanceAccountType",
            },
          },
          {
            name: "realm",
            type: "publicKey",
          },
          {
            name: "governingTokenMint",
            type: "publicKey",
          },
          {
            name: "governingTokenOwner",
            type: "publicKey",
          },
          {
            name: "governingTokenDepositAmount",
            type: "u64",
          },
          {
            name: "unrelinquishedVotesCount",
            type: "u32",
          },
          {
            name: "totalVotesCount",
            type: "u32",
          },
          {
            name: "outstandingProposalCount",
            type: "u8",
          },
          {
            name: "reserved",
            type: {
              array: ["u8", 7],
            },
          },
          {
            name: "governanceDelegate",
            type: {
              option: "publicKey",
            },
          },
        ],
      },
    },
    {
      name: "governanceV1",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "GovernanceAccountType",
            },
          },
          {
            name: "realm",
            type: "publicKey",
          },
          {
            name: "governedAccount",
            type: "publicKey",
          },
          {
            name: "proposalsCount",
            type: "u32",
          },
          {
            name: "config",
            type: {
              defined: "GovernanceConfig",
            },
          },
          {
            name: "reserved",
            type: {
              array: ["u8", 6],
            },
          },
          {
            name: "votingProposalCount",
            type: "u16",
          },
        ],
      },
    },
    {
      name: "proposalV1",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "GovernanceAccountType",
            },
          },
          {
            name: "governance",
            type: "publicKey",
          },
          {
            name: "governingTokenMint",
            type: "publicKey",
          },
          {
            name: "state",
            type: {
              defined: "ProposalState",
            },
          },
          {
            name: "tokenOwnerRecord",
            type: "publicKey",
          },
          {
            name: "signatoriesCount",
            type: "u8",
          },
          {
            name: "signatoriesSignedOffCount",
            type: "u8",
          },
          {
            name: "yesVotesCount",
            type: "u64",
          },
          {
            name: "noVotesCount",
            type: "u64",
          },
          {
            name: "instructionsExecutedCount",
            type: "u16",
          },
          {
            name: "instructionsCount",
            type: "u16",
          },
          {
            name: "instructionsNextIndex",
            type: "u16",
          },
          {
            name: "draftAt",
            type: "i64",
          },
          {
            name: "signingOffAt",
            type: {
              option: "i64",
            },
          },
          {
            name: "votingAt",
            type: {
              option: "i64",
            },
          },
          {
            name: "votingAtSlot",
            type: {
              option: "u64",
            },
          },
          {
            name: "votingCompletedAt",
            type: {
              option: "i64",
            },
          },
          {
            name: "executingAt",
            type: {
              option: "i64",
            },
          },
          {
            name: "closedAt",
            type: {
              option: "i64",
            },
          },
          {
            name: "executionFlags",
            type: {
              defined: "InstructionExecutionFlags",
            },
          },
          {
            name: "maxVoteWeight",
            type: {
              option: "u64",
            },
          },
          {
            name: "voteThreshold",
            type: {
              option: {
                defined: "VoteThreshold",
              },
            },
          },
          {
            name: "name",
            type: "string",
          },
          {
            name: "descriptionLink",
            type: "string",
          },
        ],
      },
    },
    {
      name: "signatoryRecordV1",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "GovernanceAccountType",
            },
          },
          {
            name: "proposal",
            type: "publicKey",
          },
          {
            name: "signatory",
            type: "publicKey",
          },
          {
            name: "signedOff",
            type: "bool",
          },
        ],
      },
    },
    {
      name: "voteRecordV1",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "GovernanceAccountType",
            },
          },
          {
            name: "proposal",
            type: "publicKey",
          },
          {
            name: "governingTokenOwner",
            type: "publicKey",
          },
          {
            name: "isRelinquished",
            type: "bool",
          },
          {
            name: "voteWeight",
            type: {
              defined: "VoteWeightV1",
            },
          },
        ],
      },
    },
    {
      name: "governanceV2",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "GovernanceAccountType",
            },
          },
          {
            name: "realm",
            type: "publicKey",
          },
          {
            name: "governedAccount",
            type: "publicKey",
          },
          {
            name: "proposalsCount",
            type: "u32",
          },
          {
            name: "config",
            type: {
              defined: "GovernanceConfig",
            },
          },
          {
            name: "reserved",
            type: {
              array: ["u8", 6],
            },
          },
          {
            name: "votingProposalCount",
            type: "u16",
          },
          {
            name: "reservedV2",
            type: {
              array: ["u8", 128],
            },
          },
        ],
      },
    },
    {
      name: "voteRecordV2",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "GovernanceAccountType",
            },
          },
          {
            name: "proposal",
            type: "publicKey",
          },
          {
            name: "governingTokenOwner",
            type: "publicKey",
          },
          {
            name: "isRelinquished",
            type: "bool",
          },
          {
            name: "voterWeight",
            type: "u64",
          },
          {
            name: "vote",
            type: {
              defined: "Vote",
            },
          },
          {
            name: "reservedV2",
            type: {
              array: ["u8", 8],
            },
          },
        ],
      },
    },
    {
      name: "tokenOwnerRecordV2",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "GovernanceAccountType",
            },
          },
          {
            name: "realm",
            type: "publicKey",
          },
          {
            name: "governingTokenMint",
            type: "publicKey",
          },
          {
            name: "governingTokenOwner",
            type: "publicKey",
          },
          {
            name: "governingTokenDepositAmount",
            type: "u64",
          },
          {
            name: "unrelinquishedVotesCount",
            type: "u32",
          },
          {
            name: "totalVotesCount",
            type: "u32",
          },
          {
            name: "outstandingProposalCount",
            type: "u8",
          },
          {
            name: "reserved",
            type: {
              array: ["u8", 7],
            },
          },
          {
            name: "governanceDelegate",
            type: {
              option: "publicKey",
            },
          },
          {
            name: "reservedV2",
            type: {
              array: ["u8", 128],
            },
          },
        ],
      },
    },
    {
      name: "realmConfigAccount",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "GovernanceAccountType",
            },
          },
          {
            name: "realm",
            type: "publicKey",
          },
          {
            name: "communityVoterWeightAddin",
            type: {
              option: "publicKey",
            },
          },
          {
            name: "maxCommunityVoterWeightAddin",
            type: {
              option: "publicKey",
            },
          },
          {
            name: "councilVoterWeightAddin",
            type: {
              option: "publicKey",
            },
          },
          {
            name: "councilMaxVoteWeightAddin",
            type: {
              option: "publicKey",
            },
          },
          {
            name: "reserved",
            type: {
              array: ["u8", 128],
            },
          },
        ],
      },
    },
    {
      name: "proposalTransactionV2",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "GovernanceAccountType",
            },
          },
          {
            name: "proposal",
            type: "publicKey",
          },
          {
            name: "optionIndex",
            type: "u8",
          },
          {
            name: "transactionIndex",
            type: "u16",
          },
          {
            name: "holdUpTime",
            type: "u32",
          },
          {
            name: "instructions",
            type: {
              vec: {
                defined: "InstructionData",
              },
            },
          },
          {
            name: "executedAt",
            type: {
              option: "i64",
            },
          },
          {
            name: "executionStatus",
            type: {
              defined: "TransactionExecutionStatus",
            },
          },
          {
            name: "reservedV2",
            type: {
              array: ["u8", 8],
            },
          },
        ],
      },
    },
  ],
  types: [
    {
      name: "NativeTreasury",
      type: {
        kind: "struct",
        fields: [],
      },
    },
    {
      name: "RealmConfigArgs",
      type: {
        kind: "struct",
        fields: [
          {
            name: "useCouncilMint",
            type: "bool",
          },
          {
            name: "minCommunityWeightToCreateGovernance",
            type: "u64",
          },
          {
            name: "communityMintMaxVoteWeightSource",
            type: {
              defined: "MintMaxVoteWeightSource",
            },
          },
          {
            name: "useCommunityVoterWeightAddin",
            type: "bool",
          },
          {
            name: "useMaxCommunityVoterWeightAddin",
            type: "bool",
          },
        ],
      },
    },
    {
      name: "RealmConfig",
      type: {
        kind: "struct",
        fields: [
          {
            name: "useCommunityVoterWeightAddin",
            type: "bool",
          },
          {
            name: "useMaxCommunityVoterWeightAddin",
            type: "bool",
          },
          {
            name: "reserved",
            type: {
              array: ["u8", 6],
            },
          },
          {
            name: "minCommunityWeightToCreateGovernance",
            type: "u64",
          },
          {
            name: "communityMintMaxVoteWeightSource",
            type: {
              defined: "MintMaxVoteWeightSource",
            },
          },
          {
            name: "councilMint",
            type: {
              option: "publicKey",
            },
          },
        ],
      },
    },
    {
      name: "ProposalOption",
      type: {
        kind: "struct",
        fields: [
          {
            name: "label",
            type: "string",
          },
          {
            name: "voteWeight",
            type: "u64",
          },
          {
            name: "voteResult",
            type: {
              defined: "OptionVoteResult",
            },
          },
          {
            name: "transactionsExecutedCount",
            type: "u16",
          },
          {
            name: "transactionsCount",
            type: "u16",
          },
          {
            name: "transactionsNextIndex",
            type: "u16",
          },
        ],
      },
    },
    {
      name: "GovernanceConfig",
      type: {
        kind: "struct",
        fields: [
          {
            name: "communityVoteThreshold",
            type: {
              defined: "VoteThreshold",
            },
          },
          {
            name: "minCommunityWeightToCreateProposal",
            type: "u64",
          },
          {
            name: "minTransactionHoldUpTime",
            type: "u32",
          },
          {
            name: "maxVotingTime",
            type: "u32",
          },
          {
            name: "voteTipping",
            type: {
              defined: "VoteTipping",
            },
          },
          {
            name: "councilVoteThreshold",
            type: {
              defined: "VoteThreshold",
            },
          },
          {
            name: "councilVetoVoteThreshold",
            type: {
              defined: "VoteThreshold",
            },
          },
          {
            name: "minCouncilWeightToCreateProposal",
            type: "u64",
          },
        ],
      },
    },
    {
      name: "AccountMetaData",
      type: {
        kind: "struct",
        fields: [
          {
            name: "pubkey",
            type: "publicKey",
          },
          {
            name: "isSigner",
            type: "bool",
          },
          {
            name: "isWritable",
            type: "bool",
          },
        ],
      },
    },
    {
      name: "InstructionData",
      type: {
        kind: "struct",
        fields: [
          {
            name: "programId",
            type: "publicKey",
          },
          {
            name: "accounts",
            type: {
              vec: {
                defined: "AccountMetaData",
              },
            },
          },
          {
            name: "data",
            type: "bytes",
          },
        ],
      },
    },
    {
      name: "ProposalInstructionV1",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "GovernanceAccountType",
            },
          },
          {
            name: "proposal",
            type: "publicKey",
          },
          {
            name: "instructionIndex",
            type: "u16",
          },
          {
            name: "holdUpTime",
            type: "u32",
          },
          {
            name: "instruction",
            type: {
              defined: "InstructionData",
            },
          },
          {
            name: "executedAt",
            type: {
              option: "i64",
            },
          },
          {
            name: "executionStatus",
            type: {
              defined: "TransactionExecutionStatus",
            },
          },
        ],
      },
    },
    {
      name: "VoteChoice",
      type: {
        kind: "struct",
        fields: [
          {
            name: "rank",
            type: "u8",
          },
          {
            name: "weightPercentage",
            type: "u8",
          },
        ],
      },
    },
    {
      name: "MintMaxVoteWeightSource",
      type: {
        kind: "enum",
        variants: [
          {
            name: "SupplyFraction",
            fields: ["u64"],
          },
          {
            name: "Absolute",
            fields: ["u64"],
          },
        ],
      },
    },
    {
      name: "GovernanceAccountType",
      type: {
        kind: "enum",
        variants: [
          {
            name: "Uninitialized",
          },
          {
            name: "RealmV1",
          },
          {
            name: "TokenOwnerRecordV1",
          },
          {
            name: "GovernanceV1",
          },
          {
            name: "ProgramGovernanceV1",
          },
          {
            name: "ProposalV1",
          },
          {
            name: "SignatoryRecordV1",
          },
          {
            name: "VoteRecordV1",
          },
          {
            name: "ProposalInstructionV1",
          },
          {
            name: "MintGovernanceV1",
          },
          {
            name: "TokenGovernanceV1",
          },
          {
            name: "RealmConfig",
          },
          {
            name: "VoteRecordV2",
          },
          {
            name: "ProposalTransactionV2",
          },
          {
            name: "ProposalV2",
          },
          {
            name: "ProgramMetadata",
          },
          {
            name: "RealmV2",
          },
          {
            name: "TokenOwnerRecordV2",
          },
          {
            name: "GovernanceV2",
          },
          {
            name: "ProgramGovernanceV2",
          },
          {
            name: "MintGovernanceV2",
          },
          {
            name: "TokenGovernanceV2",
          },
          {
            name: "SignatoryRecordV2",
          },
        ],
      },
    },
    {
      name: "OptionVoteResult",
      type: {
        kind: "enum",
        variants: [
          {
            name: "None",
          },
          {
            name: "Succeeded",
          },
          {
            name: "Defeated",
          },
        ],
      },
    },
    {
      name: "ProposalState",
      type: {
        kind: "enum",
        variants: [
          {
            name: "Draft",
          },
          {
            name: "SigningOff",
          },
          {
            name: "Voting",
          },
          {
            name: "Succeeded",
          },
          {
            name: "Executing",
          },
          {
            name: "Completed",
          },
          {
            name: "Cancelled",
          },
          {
            name: "Defeated",
          },
          {
            name: "ExecutingWithErrors",
          },
          {
            name: "Vetoed",
          },
        ],
      },
    },
    {
      name: "VoteType",
      type: {
        kind: "enum",
        variants: [
          {
            name: "SingleChoice",
          },
          {
            name: "MultiChoice",
            fields: [
              {
                name: "max_voter_options",
                type: "u8",
              },
              {
                name: "max_winning_options",
                type: "u8",
              },
            ],
          },
        ],
      },
    },
    {
      name: "InstructionExecutionFlags",
      type: {
        kind: "enum",
        variants: [
          {
            name: "None",
          },
          {
            name: "Ordered",
          },
          {
            name: "UseTransaction",
          },
        ],
      },
    },
    {
      name: "VoteThreshold",
      type: {
        kind: "enum",
        variants: [
          {
            name: "YesVotePercentage",
            fields: ["u8"],
          },
          {
            name: "QuorumPercentage",
            fields: ["u8"],
          },
          {
            name: "Disabled",
          },
        ],
      },
    },
    {
      name: "VoteTipping",
      type: {
        kind: "enum",
        variants: [
          {
            name: "Strict",
          },
          {
            name: "Early",
          },
          {
            name: "Disabled",
          },
        ],
      },
    },
    {
      name: "TransactionExecutionStatus",
      type: {
        kind: "enum",
        variants: [
          {
            name: "None",
          },
          {
            name: "Success",
          },
          {
            name: "Error",
          },
        ],
      },
    },
    {
      name: "VoteWeightV1",
      type: {
        kind: "enum",
        variants: [
          {
            name: "Yes",
            fields: ["u64"],
          },
          {
            name: "No",
            fields: ["u64"],
          },
        ],
      },
    },
    {
      name: "Vote",
      type: {
        kind: "enum",
        variants: [
          {
            name: "Approve",
            fields: [
              {
                vec: {
                  defined: "VoteChoice",
                },
              },
            ],
          },
          {
            name: "Deny",
          },
          {
            name: "Abstain",
          },
          {
            name: "Veto",
          },
        ],
      },
    },
    {
      name: "SetRealmAuthorityAction",
      type: {
        kind: "enum",
        variants: [
          {
            name: "SetUnchecked",
          },
          {
            name: "SetChecked",
          },
          {
            name: "Remove",
          },
        ],
      },
    },
  ],
  errors: [
    {
      code: 500,
      name: "InvalidInstruction",
      msg: "Invalid instruction passed to program",
    },
    {
      code: 501,
      name: "RealmAlreadyExists",
      msg: "Realm with the given name and governing mints already exists",
    },
    {
      code: 502,
      name: "InvalidRealm",
      msg: "Invalid realm",
    },
    {
      code: 503,
      name: "InvalidGoverningTokenMint",
      msg: "Invalid Governing Token Mint",
    },
    {
      code: 504,
      name: "GoverningTokenOwnerMustSign",
      msg: "Governing Token Owner must sign transaction",
    },
    {
      code: 505,
      name: "GoverningTokenOwnerOrDelegateMustSign",
      msg: "Governing Token Owner or Delegate  must sign transaction",
    },
    {
      code: 506,
      name: "AllVotesMustBeRelinquishedToWithdrawGoverningTokens",
      msg: "All votes must be relinquished to withdraw governing tokens",
    },
    {
      code: 507,
      name: "InvalidTokenOwnerRecordAccountAddress",
      msg: "Invalid Token Owner Record account address",
    },
    {
      code: 508,
      name: "InvalidGoverningMintForTokenOwnerRecord",
      msg: "Invalid GoverningMint for TokenOwnerRecord",
    },
    {
      code: 509,
      name: "InvalidRealmForTokenOwnerRecord",
      msg: "Invalid Realm for TokenOwnerRecord",
    },
    {
      code: 510,
      name: "InvalidProposalForProposalTransaction",
      msg: "Invalid Proposal for ProposalTransaction,",
    },
    {
      code: 511,
      name: "InvalidSignatoryAddress",
      msg: "Invalid Signatory account address",
    },
    {
      code: 512,
      name: "SignatoryAlreadySignedOff",
      msg: "Signatory already signed off",
    },
    {
      code: 513,
      name: "SignatoryMustSign",
      msg: "Signatory must sign",
    },
    {
      code: 514,
      name: "InvalidProposalOwnerAccount",
      msg: "Invalid Proposal Owner",
    },
    {
      code: 515,
      name: "InvalidProposalForVoterRecord",
      msg: "Invalid Proposal for VoterRecord",
    },
    {
      code: 516,
      name: "InvalidGoverningTokenOwnerForVoteRecord",
      msg: "Invalid GoverningTokenOwner for VoteRecord",
    },
    {
      code: 517,
      name: "InvalidVoteThresholdPercentage",
      msg: "Invalid Governance config: Vote threshold percentage out of range",
    },
    {
      code: 518,
      name: "ProposalAlreadyExists",
      msg: "Proposal for the given Governance, Governing Token Mint and index already exists",
    },
    {
      code: 519,
      name: "VoteAlreadyExists",
      msg: "Token Owner already voted on the Proposal",
    },
    {
      code: 520,
      name: "NotEnoughTokensToCreateProposal",
      msg: "Owner doesn't have enough governing tokens to create Proposal",
    },
    {
      code: 521,
      name: "InvalidStateCannotEditSignatories",
      msg: "Invalid State: Can't edit Signatories",
    },
    {
      code: 522,
      name: "InvalidProposalState",
      msg: "Invalid Proposal state",
    },
    {
      code: 523,
      name: "InvalidStateCannotEditTransactions",
      msg: "Invalid State: Can't edit transactions",
    },
    {
      code: 524,
      name: "InvalidStateCannotExecuteTransaction",
      msg: "Invalid State: Can't execute transaction",
    },
    {
      code: 525,
      name: "CannotExecuteTransactionWithinHoldUpTime",
      msg: "Can't execute transaction within its hold up time",
    },
    {
      code: 526,
      name: "TransactionAlreadyExecuted",
      msg: "Transaction already executed",
    },
    {
      code: 527,
      name: "InvalidTransactionIndex",
      msg: "Invalid Transaction index",
    },
    {
      code: 528,
      name: "TransactionHoldUpTimeBelowRequiredMin",
      msg: "Transaction hold up time is below the min specified by Governance",
    },
    {
      code: 529,
      name: "TransactionAlreadyExists",
      msg: "Transaction at the given index for the Proposal already exists",
    },
    {
      code: 530,
      name: "InvalidStateCannotSignOff",
      msg: "Invalid State: Can't sign off",
    },
    {
      code: 531,
      name: "InvalidStateCannotVote",
      msg: "Invalid State: Can't vote",
    },
    {
      code: 532,
      name: "InvalidStateCannotFinalize",
      msg: "Invalid State: Can't finalize vote",
    },
    {
      code: 533,
      name: "InvalidStateCannotCancelProposal",
      msg: "Invalid State: Can't cancel Proposal",
    },
    {
      code: 534,
      name: "VoteAlreadyRelinquished",
      msg: "Vote already relinquished",
    },
    {
      code: 535,
      name: "CannotFinalizeVotingInProgress",
      msg: "Can't finalize vote. Voting still in progress",
    },
    {
      code: 536,
      name: "ProposalVotingTimeExpired",
      msg: "Proposal voting time expired",
    },
    {
      code: 537,
      name: "InvalidSignatoryMint",
      msg: "Invalid Signatory Mint",
    },
    {
      code: 538,
      name: "InvalidGovernanceForProposal",
      msg: "Proposal does not belong to the given Governance",
    },
    {
      code: 539,
      name: "InvalidGoverningMintForProposal",
      msg: "Proposal does not belong to given Governing Mint",
    },
    {
      code: 540,
      name: "MintAuthorityMustSign",
      msg: "Current mint authority must sign transaction",
    },
    {
      code: 541,
      name: "InvalidMintAuthority",
      msg: "Invalid mint authority",
    },
    {
      code: 542,
      name: "MintHasNoAuthority",
      msg: "Mint has no authority",
    },
    {
      code: 543,
      name: "SplTokenAccountWithInvalidOwner",
      msg: "Invalid Token account owner",
    },
    {
      code: 544,
      name: "SplTokenMintWithInvalidOwner",
      msg: "Invalid Mint account owner",
    },
    {
      code: 545,
      name: "SplTokenAccountNotInitialized",
      msg: "Token Account is not initialized",
    },
    {
      code: 546,
      name: "SplTokenAccountDoesNotExist",
      msg: "Token Account doesn't exist",
    },
    {
      code: 547,
      name: "SplTokenInvalidTokenAccountData",
      msg: "Token account data is invalid",
    },
    {
      code: 548,
      name: "SplTokenInvalidMintAccountData",
      msg: "Token mint account data is invalid",
    },
    {
      code: 549,
      name: "SplTokenMintNotInitialized",
      msg: "Token Mint account is not initialized",
    },
    {
      code: 550,
      name: "SplTokenMintDoesNotExist",
      msg: "Token Mint account doesn't exist",
    },
    {
      code: 551,
      name: "InvalidProgramDataAccountAddress",
      msg: "Invalid ProgramData account address",
    },
    {
      code: 552,
      name: "InvalidProgramDataAccountData",
      msg: "Invalid ProgramData account Data",
    },
    {
      code: 553,
      name: "InvalidUpgradeAuthority",
      msg: "Provided upgrade authority doesn't match current program upgrade authority",
    },
    {
      code: 554,
      name: "UpgradeAuthorityMustSign",
      msg: "Current program upgrade authority must sign transaction",
    },
    {
      code: 555,
      name: "ProgramNotUpgradable",
      msg: "Given program is not upgradable",
    },
    {
      code: 556,
      name: "InvalidTokenOwner",
      msg: "Invalid token owner",
    },
    {
      code: 557,
      name: "TokenOwnerMustSign",
      msg: "Current token owner must sign transaction",
    },
    {
      code: 558,
      name: "VoteThresholdTypeNotSupported",
      msg: "Given VoteThresholdType is not supported",
    },
    {
      code: 559,
      name: "VoteWeightSourceNotSupported",
      msg: "Given VoteWeightSource is not supported",
    },
    {
      code: 560,
      name: "GoverningTokenMintNotAllowedToVote",
      msg: "GoverningTokenMint not allowed to vote",
    },
    {
      code: 561,
      name: "GovernancePdaMustSign",
      msg: "Governance PDA must sign",
    },
    {
      code: 562,
      name: "TransactionAlreadyFlaggedWithError",
      msg: "Transaction already flagged with error",
    },
    {
      code: 563,
      name: "InvalidRealmForGovernance",
      msg: "Invalid Realm for Governance",
    },
    {
      code: 564,
      name: "InvalidAuthorityForRealm",
      msg: "Invalid Authority for Realm",
    },
    {
      code: 565,
      name: "RealmHasNoAuthority",
      msg: "Realm has no authority",
    },
    {
      code: 566,
      name: "RealmAuthorityMustSign",
      msg: "Realm authority must sign",
    },
    {
      code: 567,
      name: "InvalidGoverningTokenHoldingAccount",
      msg: "Invalid governing token holding account",
    },
    {
      code: 568,
      name: "RealmCouncilMintChangeIsNotSupported",
      msg: "Realm council mint change is not supported",
    },
    {
      code: 569,
      name: "MintMaxVoteWeightSourceNotSupported",
      msg: "Not supported mint max vote weight source",
    },
    {
      code: 570,
      name: "InvalidMaxVoteWeightSupplyFraction",
      msg: "Invalid max vote weight supply fraction",
    },
    {
      code: 571,
      name: "NotEnoughTokensToCreateGovernance",
      msg: "Owner doesn't have enough governing tokens to create Governance",
    },
    {
      code: 572,
      name: "TooManyOutstandingProposals",
      msg: "Too many outstanding proposals",
    },
    {
      code: 573,
      name: "AllProposalsMustBeFinalisedToWithdrawGoverningTokens",
      msg: "All proposals must be finalized to withdraw governing tokens",
    },
    {
      code: 574,
      name: "InvalidVoterWeightRecordForRealm",
      msg: "Invalid VoterWeightRecord for Realm",
    },
    {
      code: 575,
      name: "InvalidVoterWeightRecordForGoverningTokenMint",
      msg: "Invalid VoterWeightRecord for GoverningTokenMint",
    },
    {
      code: 576,
      name: "InvalidVoterWeightRecordForTokenOwner",
      msg: "Invalid VoterWeightRecord for TokenOwner",
    },
    {
      code: 577,
      name: "VoterWeightRecordExpired",
      msg: "VoterWeightRecord expired",
    },
    {
      code: 578,
      name: "InvalidRealmConfigForRealm",
      msg: "Invalid RealmConfig for Realm",
    },
    {
      code: 579,
      name: "TokenOwnerRecordAlreadyExists",
      msg: "TokenOwnerRecord already exists",
    },
    {
      code: 580,
      name: "GoverningTokenDepositsNotAllowed",
      msg: "Governing token deposits not allowed",
    },
    {
      code: 581,
      name: "InvalidVoteChoiceWeightPercentage",
      msg: "Invalid vote choice weight percentage",
    },
    {
      code: 582,
      name: "VoteTypeNotSupported",
      msg: "Vote type not supported",
    },
    {
      code: 583,
      name: "InvalidProposalOptions",
      msg: "Invalid proposal options",
    },
    {
      code: 584,
      name: "ProposalIsNotExecutable",
      msg: "Proposal is not not executable",
    },
    {
      code: 585,
      name: "InvalidVote",
      msg: "Invalid vote",
    },
    {
      code: 586,
      name: "CannotExecuteDefeatedOption",
      msg: "Cannot execute defeated option",
    },
    {
      code: 587,
      name: "VoterWeightRecordInvalidAction",
      msg: "VoterWeightRecord invalid action",
    },
    {
      code: 588,
      name: "VoterWeightRecordInvalidActionTarget",
      msg: "VoterWeightRecord invalid action target",
    },
    {
      code: 589,
      name: "InvalidMaxVoterWeightRecordForRealm",
      msg: "Invalid MaxVoterWeightRecord for Realm",
    },
    {
      code: 590,
      name: "InvalidMaxVoterWeightRecordForGoverningTokenMint",
      msg: "Invalid MaxVoterWeightRecord for GoverningTokenMint",
    },
    {
      code: 591,
      name: "MaxVoterWeightRecordExpired",
      msg: "MaxVoterWeightRecord expired",
    },
    {
      code: 592,
      name: "NotSupportedVoteType",
      msg: "Not supported VoteType",
    },
    {
      code: 593,
      name: "RealmConfigChangeNotAllowed",
      msg: "RealmConfig change not allowed",
    },
    {
      code: 594,
      name: "GovernanceConfigChangeNotAllowed",
      msg: "GovernanceConfig change not allowed",
    },
    {
      code: 595,
      name: "AtLeastOneVoteThresholdRequired",
      msg: "At least one VoteThreshold is required",
    },
    {
      code: 596,
      name: "ReservedBufferMustBeEmpty",
      msg: "Reserved buffer must be empty",
    },
    {
      code: 597,
      name: "CannotRelinquishInFinalizingState",
      msg: "Cannot Relinquish in Finalizing state",
    },
  ],
};
