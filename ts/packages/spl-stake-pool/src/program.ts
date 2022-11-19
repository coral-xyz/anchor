import { PublicKey } from "@solana/web3.js";
import { Program, AnchorProvider } from "@project-serum/anchor";

import { SplStakePoolCoder } from "./coder";

export const SPL_STAKE_POOL_PROGRAM_ID = new PublicKey(
  "SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy"
);

interface GetProgramParams {
  programId?: PublicKey;
  provider?: AnchorProvider;
}

export function splStakePoolProgram(
  params?: GetProgramParams
): Program<SplStakePool> {
  return new Program<SplStakePool>(
    IDL,
    params?.programId ?? SPL_STAKE_POOL_PROGRAM_ID,
    params?.provider,
    new SplStakePoolCoder(IDL)
  );
}

type SplStakePool = {
  version: "0.7.0";
  name: "spl_stake_pool";
  instructions: [
    {
      name: "initialize";
      accounts: [
        {
          name: "stakePool";
          isMut: true;
          isSigner: false;
        },
        {
          name: "manager";
          isMut: false;
          isSigner: true;
        },
        {
          name: "staker";
          isMut: false;
          isSigner: false;
        },
        {
          name: "stakePoolWithdrawAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "validatorList";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserveStake";
          isMut: false;
          isSigner: false;
        },
        {
          name: "poolMint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "managerPoolAccount";
          isMut: true;
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
          name: "fee";
          type: {
            defined: "Fee";
          };
        },
        {
          name: "withdrawalFee";
          type: {
            defined: "Fee";
          };
        },
        {
          name: "depositFee";
          type: {
            defined: "Fee";
          };
        },
        {
          name: "referralFee";
          type: "u8";
        },
        {
          name: "maxValidators";
          type: "u32";
        }
      ];
    },
    {
      name: "addValidatorToPool";
      accounts: [
        {
          name: "stakePool";
          isMut: true;
          isSigner: false;
        },
        {
          name: "staker";
          isMut: false;
          isSigner: true;
        },
        {
          name: "funder";
          isMut: true;
          isSigner: true;
        },
        {
          name: "stakePoolWithdraw";
          isMut: false;
          isSigner: false;
        },
        {
          name: "validatorList";
          isMut: true;
          isSigner: false;
        },
        {
          name: "stake";
          isMut: true;
          isSigner: false;
        },
        {
          name: "validator";
          isMut: false;
          isSigner: false;
        },
        {
          name: "rent";
          isMut: false;
          isSigner: false;
        },
        {
          name: "clock";
          isMut: false;
          isSigner: false;
        },
        {
          name: "sysvarStakeHistory";
          isMut: false;
          isSigner: false;
        },
        {
          name: "stakeConfig";
          isMut: false;
          isSigner: false;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "stakeProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "removeValidatorFromPool";
      accounts: [
        {
          name: "stakePool";
          isMut: true;
          isSigner: false;
        },
        {
          name: "staker";
          isMut: false;
          isSigner: true;
        },
        {
          name: "stakePoolWithdraw";
          isMut: false;
          isSigner: false;
        },
        {
          name: "newStakeAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "validatorList";
          isMut: true;
          isSigner: false;
        },
        {
          name: "stakeAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "transientStakeAccount";
          isMut: false;
          isSigner: false;
        },
        {
          name: "destinationStakeAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "clock";
          isMut: false;
          isSigner: false;
        },
        {
          name: "stakeProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "decreaseValidatorStake";
      accounts: [
        {
          name: "stakePool";
          isMut: false;
          isSigner: false;
        },
        {
          name: "staker";
          isMut: false;
          isSigner: true;
        },
        {
          name: "stakePoolWithdrawAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "validatorList";
          isMut: true;
          isSigner: false;
        },
        {
          name: "validatorStake";
          isMut: true;
          isSigner: false;
        },
        {
          name: "transientStake";
          isMut: true;
          isSigner: false;
        },
        {
          name: "clock";
          isMut: false;
          isSigner: false;
        },
        {
          name: "rent";
          isMut: false;
          isSigner: false;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "stakeProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "lamports";
          type: "u64";
        },
        {
          name: "transientStakeSeed";
          type: "u64";
        }
      ];
    },
    {
      name: "increaseValidatorStake";
      accounts: [
        {
          name: "stakePool";
          isMut: false;
          isSigner: false;
        },
        {
          name: "staker";
          isMut: false;
          isSigner: true;
        },
        {
          name: "stakePoolWithdrawAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "validatorList";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserveStake";
          isMut: true;
          isSigner: false;
        },
        {
          name: "transientStake";
          isMut: true;
          isSigner: false;
        },
        {
          name: "validatorStake";
          isMut: false;
          isSigner: false;
        },
        {
          name: "validator";
          isMut: false;
          isSigner: false;
        },
        {
          name: "clock";
          isMut: false;
          isSigner: false;
        },
        {
          name: "rent";
          isMut: false;
          isSigner: false;
        },
        {
          name: "sysvarStakeHistory";
          isMut: false;
          isSigner: false;
        },
        {
          name: "stakeConfig";
          isMut: false;
          isSigner: false;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "stakeProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "lamports";
          type: "u64";
        },
        {
          name: "transientStakeSeed";
          type: "u64";
        }
      ];
    },
    {
      name: "setPreferredValidator";
      accounts: [
        {
          name: "stakePoolAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "staker";
          isMut: false;
          isSigner: true;
        },
        {
          name: "validatorListAddress";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "validatorType";
          type: {
            defined: "PreferredValidatorType";
          };
        },
        {
          name: "validatorVoteAddress";
          type: {
            option: "publicKey";
          };
        }
      ];
    },
    {
      name: "updateValidatorListBalance";
      accounts: [
        {
          name: "stakePool";
          isMut: false;
          isSigner: false;
        },
        {
          name: "stakePoolWithdrawAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "validatorListAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserveStake";
          isMut: true;
          isSigner: false;
        },
        {
          name: "clock";
          isMut: false;
          isSigner: false;
        },
        {
          name: "sysvarStakeHistory";
          isMut: false;
          isSigner: false;
        },
        {
          name: "stakeProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "startIndex";
          type: "u32";
        },
        {
          name: "noMerge";
          type: "bool";
        }
      ];
    },
    {
      name: "updateStakePoolBalance";
      accounts: [
        {
          name: "stakePool";
          isMut: true;
          isSigner: false;
        },
        {
          name: "withdrawAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "validatorListStorage";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserveStake";
          isMut: false;
          isSigner: false;
        },
        {
          name: "managerFeeAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "stakePoolMint";
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
      name: "cleanupRemovedValidatorEntries";
      accounts: [
        {
          name: "stakePool";
          isMut: false;
          isSigner: false;
        },
        {
          name: "validatorListStorage";
          isMut: true;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "depositStake";
      accounts: [
        {
          name: "stakePool";
          isMut: true;
          isSigner: false;
        },
        {
          name: "validatorListStorage";
          isMut: true;
          isSigner: false;
        },
        {
          name: "stakePoolDepositAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "stakePoolWithdrawAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "depositStakeAddress";
          isMut: true;
          isSigner: false;
        },
        {
          name: "validatorStakeAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserveStakeAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "poolTokensTo";
          isMut: true;
          isSigner: false;
        },
        {
          name: "managerFeeAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "referrerPoolTokensAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "poolMint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "clock";
          isMut: false;
          isSigner: false;
        },
        {
          name: "sysvarStakeHistory";
          isMut: false;
          isSigner: false;
        },
        {
          name: "tokenProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "stakeProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "withdrawStake";
      accounts: [
        {
          name: "stakePool";
          isMut: true;
          isSigner: false;
        },
        {
          name: "validatorListStorage";
          isMut: true;
          isSigner: false;
        },
        {
          name: "stakePoolWithdraw";
          isMut: false;
          isSigner: false;
        },
        {
          name: "stakeToSplit";
          isMut: true;
          isSigner: false;
        },
        {
          name: "stakeToReceive";
          isMut: true;
          isSigner: false;
        },
        {
          name: "userStakeAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "userTransferAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "userPoolTokenAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "managerFeeAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "poolMint";
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
        },
        {
          name: "stakeProgram";
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
      name: "setManager";
      accounts: [
        {
          name: "stakePool";
          isMut: true;
          isSigner: false;
        },
        {
          name: "manager";
          isMut: false;
          isSigner: true;
        },
        {
          name: "newManager";
          isMut: false;
          isSigner: true;
        },
        {
          name: "newFeeReceiver";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "setFee";
      accounts: [
        {
          name: "stakePool";
          isMut: true;
          isSigner: false;
        },
        {
          name: "manager";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [
        {
          name: "fee";
          type: {
            defined: "FeeType";
          };
        }
      ];
    },
    {
      name: "setStaker";
      accounts: [
        {
          name: "stakePool";
          isMut: true;
          isSigner: false;
        },
        {
          name: "setStakerAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "newStaker";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "depositSol";
      accounts: [
        {
          name: "stakePool";
          isMut: true;
          isSigner: false;
        },
        {
          name: "stakePoolWithdrawAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "reserveStakeAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "lamportsFrom";
          isMut: true;
          isSigner: true;
        },
        {
          name: "poolTokensTo";
          isMut: true;
          isSigner: false;
        },
        {
          name: "managerFeeAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "referrerPoolTokensAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "poolMint";
          isMut: true;
          isSigner: false;
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
          name: "arg";
          type: "u64";
        }
      ];
    },
    {
      name: "setFundingAuthority";
      accounts: [
        {
          name: "stakePool";
          isMut: true;
          isSigner: false;
        },
        {
          name: "manager";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [
        {
          name: "arg";
          type: {
            defined: "FundingType";
          };
        }
      ];
    },
    {
      name: "withdrawSol";
      accounts: [
        {
          name: "stakePool";
          isMut: true;
          isSigner: false;
        },
        {
          name: "stakePoolWithdrawAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "userTransferAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "poolTokensFrom";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserveStakeAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "lamportsTo";
          isMut: true;
          isSigner: false;
        },
        {
          name: "managerFeeAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "poolMint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "clock";
          isMut: false;
          isSigner: false;
        },
        {
          name: "sysvarStakeHistory";
          isMut: false;
          isSigner: false;
        },
        {
          name: "stakeProgram";
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
      name: "createTokenMetadata";
      accounts: [
        {
          name: "stakePool";
          isMut: false;
          isSigner: false;
        },
        {
          name: "manager";
          isMut: false;
          isSigner: true;
        },
        {
          name: "stakePoolWithdrawAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "poolMint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "tokenMetadata";
          isMut: true;
          isSigner: false;
        },
        {
          name: "mplTokenMetadata";
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
          name: "name";
          type: "string";
        },
        {
          name: "symbol";
          type: "string";
        },
        {
          name: "uri";
          type: "string";
        }
      ];
    },
    {
      name: "updateTokenMetadata";
      accounts: [
        {
          name: "stakePool";
          isMut: false;
          isSigner: false;
        },
        {
          name: "manager";
          isMut: false;
          isSigner: true;
        },
        {
          name: "stakePoolWithdrawAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "tokenMetadata";
          isMut: true;
          isSigner: false;
        },
        {
          name: "mplTokenMetadata";
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
          name: "symbol";
          type: "string";
        },
        {
          name: "uri";
          type: "string";
        }
      ];
    }
  ];
  accounts: [
    {
      name: "stakePool";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "AccountType";
            };
          },
          {
            name: "manager";
            type: "publicKey";
          },
          {
            name: "staker";
            type: "publicKey";
          },
          {
            name: "stakeDepositAuthority";
            type: "publicKey";
          },
          {
            name: "stakeWithdrawBumpSeed";
            type: "u8";
          },
          {
            name: "validatorList";
            type: "publicKey";
          },
          {
            name: "reserveStake";
            type: "publicKey";
          },
          {
            name: "poolMint";
            type: "publicKey";
          },
          {
            name: "managerFeeAccount";
            type: "publicKey";
          },
          {
            name: "tokenProgramId";
            type: "publicKey";
          },
          {
            name: "totalLamports";
            type: "u64";
          },
          {
            name: "poolTokenSupply";
            type: "u64";
          },
          {
            name: "lastUpdateEpoch";
            type: "u64";
          },
          {
            name: "lockup";
            type: {
              defined: "Lockup";
            };
          },
          {
            name: "epochFee";
            type: {
              defined: "Fee";
            };
          },
          {
            name: "nextEpochFee";
            type: {
              option: {
                defined: "Fee";
              };
            };
          },
          {
            name: "preferredDepositValidatorVoteAddress";
            type: {
              option: "publicKey";
            };
          },
          {
            name: "preferredWithdrawValidatorVoteAddress";
            type: {
              option: "publicKey";
            };
          },
          {
            name: "stakeDepositFee";
            type: {
              defined: "Fee";
            };
          },
          {
            name: "stakeWithdrawalFee";
            type: {
              defined: "Fee";
            };
          },
          {
            name: "nextStakeWithdrawalFee";
            type: {
              option: {
                defined: "Fee";
              };
            };
          },
          {
            name: "stakeReferralFee";
            type: "u8";
          },
          {
            name: "solDepositAuthority";
            type: {
              option: "publicKey";
            };
          },
          {
            name: "solDepositFee";
            type: {
              defined: "Fee";
            };
          },
          {
            name: "solReferralFee";
            type: "u8";
          },
          {
            name: "solWithdrawAuthority";
            type: {
              option: "publicKey";
            };
          },
          {
            name: "solWithdrawalFee";
            type: {
              defined: "Fee";
            };
          },
          {
            name: "nextSolWithdrawalFee";
            type: {
              option: {
                defined: "Fee";
              };
            };
          },
          {
            name: "lastEpochPoolTokenSupply";
            type: "u64";
          },
          {
            name: "lastEpochTotalLamports";
            type: "u64";
          }
        ];
      };
    },
    {
      name: "validatorStakeInfo";
      type: {
        kind: "struct";
        fields: [
          {
            name: "activeStakeLamports";
            type: "u64";
          },
          {
            name: "transientStakeLamports";
            type: "u64";
          },
          {
            name: "lastUpdateEpoch";
            type: "u64";
          },
          {
            name: "transientSeedSuffixStart";
            type: "u64";
          },
          {
            name: "transientSeedSuffixEnd";
            type: "u64";
          },
          {
            name: "status";
            type: {
              defined: "StakeStatus";
            };
          },
          {
            name: "voteAccountAddress";
            type: "publicKey";
          }
        ];
      };
    },
    {
      name: "validatorList";
      type: {
        kind: "struct";
        fields: [
          {
            name: "header";
            type: {
              defined: "ValidatorListHeader";
            };
          },
          {
            name: "validators";
            type: {
              vec: {
                defined: "ValidatorStakeInfo";
              };
            };
          }
        ];
      };
    }
  ];
  types: [
    {
      name: "Fee";
      type: {
        kind: "struct";
        fields: [
          {
            name: "denominator";
            type: "u64";
          },
          {
            name: "numerator";
            type: "u64";
          }
        ];
      };
    },
    {
      name: "ValidatorListHeader";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountType";
            type: {
              defined: "AccountType";
            };
          },
          {
            name: "maxValidators";
            type: "u32";
          }
        ];
      };
    },
    {
      name: "AccountType";
      type: {
        kind: "enum";
        variants: [
          {
            name: "Uninitialized";
          },
          {
            name: "StakePool";
          },
          {
            name: "ValidatorList";
          }
        ];
      };
    },
    {
      name: "StakeStatus";
      type: {
        kind: "enum";
        variants: [
          {
            name: "Active";
          },
          {
            name: "DeactivatingTransient";
          },
          {
            name: "ReadyForRemoval";
          }
        ];
      };
    },
    {
      name: "PreferredValidatorType";
      type: {
        kind: "enum";
        variants: [
          {
            name: "Deposit";
          },
          {
            name: "Withdraw";
          }
        ];
      };
    },
    {
      name: "FeeType";
      type: {
        kind: "enum";
        variants: [
          {
            name: "SolReferral";
            fields: ["u8"];
          },
          {
            name: "StakeReferral";
            fields: ["u8"];
          },
          {
            name: "Epoch";
            fields: [
              {
                defined: "Fee";
              }
            ];
          },
          {
            name: "StakeWithdrawal";
            fields: [
              {
                defined: "Fee";
              }
            ];
          },
          {
            name: "SolDeposit";
            fields: [
              {
                defined: "Fee";
              }
            ];
          },
          {
            name: "StakeDeposit";
            fields: [
              {
                defined: "Fee";
              }
            ];
          },
          {
            name: "SolWithdrawal";
            fields: [
              {
                defined: "Fee";
              }
            ];
          }
        ];
      };
    },
    {
      name: "FundingType";
      type: {
        kind: "enum";
        variants: [
          {
            name: "StakeDeposit";
          },
          {
            name: "SolDeposit";
          },
          {
            name: "SolWithdraw";
          }
        ];
      };
    }
  ];
  errors: [
    {
      code: 0;
      name: "AlreadyInUse";
      msg: "AlreadyInUse";
    },
    {
      code: 1;
      name: "InvalidProgramAddress";
      msg: "InvalidProgramAddress";
    },
    {
      code: 2;
      name: "InvalidState";
      msg: "InvalidState";
    },
    {
      code: 3;
      name: "CalculationFailure";
      msg: "CalculationFailure";
    },
    {
      code: 4;
      name: "FeeTooHigh";
      msg: "FeeTooHigh";
    },
    {
      code: 5;
      name: "WrongAccountMint";
      msg: "WrongAccountMint";
    },
    {
      code: 6;
      name: "WrongManager";
      msg: "WrongManager";
    },
    {
      code: 7;
      name: "SignatureMissing";
      msg: "SignatureMissing";
    },
    {
      code: 8;
      name: "InvalidValidatorStakeList";
      msg: "InvalidValidatorStakeList";
    },
    {
      code: 9;
      name: "InvalidFeeAccount";
      msg: "InvalidFeeAccount";
    },
    {
      code: 10;
      name: "WrongPoolMint";
      msg: "WrongPoolMint";
    },
    {
      code: 11;
      name: "WrongStakeState";
      msg: "WrongStakeState";
    },
    {
      code: 12;
      name: "UserStakeNotActive";
      msg: "UserStakeNotActive";
    },
    {
      code: 13;
      name: "ValidatorAlreadyAdded";
      msg: "ValidatorAlreadyAdded";
    },
    {
      code: 14;
      name: "ValidatorNotFound";
      msg: "ValidatorNotFound";
    },
    {
      code: 15;
      name: "InvalidStakeAccountAddress";
      msg: "InvalidStakeAccountAddress";
    },
    {
      code: 16;
      name: "StakeListOutOfDate";
      msg: "StakeListOutOfDate";
    },
    {
      code: 17;
      name: "StakeListAndPoolOutOfDate";
      msg: "StakeListAndPoolOutOfDate";
    },
    {
      code: 18;
      name: "UnknownValidatorStakeAccount";
      msg: "UnknownValidatorStakeAccount";
    },
    {
      code: 19;
      name: "WrongMintingAuthority";
      msg: "WrongMintingAuthority";
    },
    {
      code: 20;
      name: "UnexpectedValidatorListAccountSize";
      msg: "UnexpectedValidatorListAccountSize";
    },
    {
      code: 21;
      name: "WrongStaker";
      msg: "WrongStaker";
    },
    {
      code: 22;
      name: "NonZeroPoolTokenSupply";
      msg: "NonZeroPoolTokenSupply";
    },
    {
      code: 23;
      name: "StakeLamportsNotEqualToMinimum";
      msg: "StakeLamportsNotEqualToMinimum";
    },
    {
      code: 24;
      name: "IncorrectDepositVoteAddress";
      msg: "IncorrectDepositVoteAddress";
    },
    {
      code: 25;
      name: "IncorrectWithdrawVoteAddress";
      msg: "IncorrectWithdrawVoteAddress";
    },
    {
      code: 26;
      name: "InvalidMintFreezeAuthority";
      msg: "InvalidMintFreezeAuthority";
    },
    {
      code: 27;
      name: "FeeIncreaseTooHigh";
      msg: "FeeIncreaseTooHigh";
    },
    {
      code: 28;
      name: "WithdrawalTooSmall";
      msg: "WithdrawalTooSmall";
    },
    {
      code: 29;
      name: "DepositTooSmall";
      msg: "DepositTooSmall";
    },
    {
      code: 30;
      name: "InvalidStakeDepositAuthority";
      msg: "InvalidStakeDepositAuthority";
    },
    {
      code: 31;
      name: "InvalidSolDepositAuthority";
      msg: "InvalidSolDepositAuthority";
    },
    {
      code: 32;
      name: "InvalidPreferredValidator";
      msg: "InvalidPreferredValidator";
    },
    {
      code: 33;
      name: "TransientAccountInUse";
      msg: "TransientAccountInUse";
    },
    {
      code: 34;
      name: "InvalidSolWithdrawAuthority";
      msg: "InvalidSolWithdrawAuthority";
    },
    {
      code: 35;
      name: "SolWithdrawalTooLarge";
      msg: "SolWithdrawalTooLarge";
    },
    {
      code: 36;
      name: "InvalidMetadataAccount";
      msg: "InvalidMetadataAccount";
    }
  ];
};

const IDL: SplStakePool = {
  version: "0.7.0",
  name: "spl_stake_pool",
  instructions: [
    {
      name: "initialize",
      accounts: [
        {
          name: "stakePool",
          isMut: true,
          isSigner: false,
        },
        {
          name: "manager",
          isMut: false,
          isSigner: true,
        },
        {
          name: "staker",
          isMut: false,
          isSigner: false,
        },
        {
          name: "stakePoolWithdrawAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "validatorList",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserveStake",
          isMut: false,
          isSigner: false,
        },
        {
          name: "poolMint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "managerPoolAccount",
          isMut: true,
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
          name: "fee",
          type: {
            defined: "Fee",
          },
        },
        {
          name: "withdrawalFee",
          type: {
            defined: "Fee",
          },
        },
        {
          name: "depositFee",
          type: {
            defined: "Fee",
          },
        },
        {
          name: "referralFee",
          type: "u8",
        },
        {
          name: "maxValidators",
          type: "u32",
        },
      ],
    },
    {
      name: "addValidatorToPool",
      accounts: [
        {
          name: "stakePool",
          isMut: true,
          isSigner: false,
        },
        {
          name: "staker",
          isMut: false,
          isSigner: true,
        },
        {
          name: "funder",
          isMut: true,
          isSigner: true,
        },
        {
          name: "stakePoolWithdraw",
          isMut: false,
          isSigner: false,
        },
        {
          name: "validatorList",
          isMut: true,
          isSigner: false,
        },
        {
          name: "stake",
          isMut: true,
          isSigner: false,
        },
        {
          name: "validator",
          isMut: false,
          isSigner: false,
        },
        {
          name: "rent",
          isMut: false,
          isSigner: false,
        },
        {
          name: "clock",
          isMut: false,
          isSigner: false,
        },
        {
          name: "sysvarStakeHistory",
          isMut: false,
          isSigner: false,
        },
        {
          name: "stakeConfig",
          isMut: false,
          isSigner: false,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "stakeProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "removeValidatorFromPool",
      accounts: [
        {
          name: "stakePool",
          isMut: true,
          isSigner: false,
        },
        {
          name: "staker",
          isMut: false,
          isSigner: true,
        },
        {
          name: "stakePoolWithdraw",
          isMut: false,
          isSigner: false,
        },
        {
          name: "newStakeAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "validatorList",
          isMut: true,
          isSigner: false,
        },
        {
          name: "stakeAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "transientStakeAccount",
          isMut: false,
          isSigner: false,
        },
        {
          name: "destinationStakeAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "clock",
          isMut: false,
          isSigner: false,
        },
        {
          name: "stakeProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "decreaseValidatorStake",
      accounts: [
        {
          name: "stakePool",
          isMut: false,
          isSigner: false,
        },
        {
          name: "staker",
          isMut: false,
          isSigner: true,
        },
        {
          name: "stakePoolWithdrawAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "validatorList",
          isMut: true,
          isSigner: false,
        },
        {
          name: "validatorStake",
          isMut: true,
          isSigner: false,
        },
        {
          name: "transientStake",
          isMut: true,
          isSigner: false,
        },
        {
          name: "clock",
          isMut: false,
          isSigner: false,
        },
        {
          name: "rent",
          isMut: false,
          isSigner: false,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "stakeProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "lamports",
          type: "u64",
        },
        {
          name: "transientStakeSeed",
          type: "u64",
        },
      ],
    },
    {
      name: "increaseValidatorStake",
      accounts: [
        {
          name: "stakePool",
          isMut: false,
          isSigner: false,
        },
        {
          name: "staker",
          isMut: false,
          isSigner: true,
        },
        {
          name: "stakePoolWithdrawAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "validatorList",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserveStake",
          isMut: true,
          isSigner: false,
        },
        {
          name: "transientStake",
          isMut: true,
          isSigner: false,
        },
        {
          name: "validatorStake",
          isMut: false,
          isSigner: false,
        },
        {
          name: "validator",
          isMut: false,
          isSigner: false,
        },
        {
          name: "clock",
          isMut: false,
          isSigner: false,
        },
        {
          name: "rent",
          isMut: false,
          isSigner: false,
        },
        {
          name: "sysvarStakeHistory",
          isMut: false,
          isSigner: false,
        },
        {
          name: "stakeConfig",
          isMut: false,
          isSigner: false,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "stakeProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "lamports",
          type: "u64",
        },
        {
          name: "transientStakeSeed",
          type: "u64",
        },
      ],
    },
    {
      name: "setPreferredValidator",
      accounts: [
        {
          name: "stakePoolAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "staker",
          isMut: false,
          isSigner: true,
        },
        {
          name: "validatorListAddress",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "validatorType",
          type: {
            defined: "PreferredValidatorType",
          },
        },
        {
          name: "validatorVoteAddress",
          type: {
            option: "publicKey",
          },
        },
      ],
    },
    {
      name: "updateValidatorListBalance",
      accounts: [
        {
          name: "stakePool",
          isMut: false,
          isSigner: false,
        },
        {
          name: "stakePoolWithdrawAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "validatorListAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserveStake",
          isMut: true,
          isSigner: false,
        },
        {
          name: "clock",
          isMut: false,
          isSigner: false,
        },
        {
          name: "sysvarStakeHistory",
          isMut: false,
          isSigner: false,
        },
        {
          name: "stakeProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "startIndex",
          type: "u32",
        },
        {
          name: "noMerge",
          type: "bool",
        },
      ],
    },
    {
      name: "updateStakePoolBalance",
      accounts: [
        {
          name: "stakePool",
          isMut: true,
          isSigner: false,
        },
        {
          name: "withdrawAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "validatorListStorage",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserveStake",
          isMut: false,
          isSigner: false,
        },
        {
          name: "managerFeeAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "stakePoolMint",
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
      name: "cleanupRemovedValidatorEntries",
      accounts: [
        {
          name: "stakePool",
          isMut: false,
          isSigner: false,
        },
        {
          name: "validatorListStorage",
          isMut: true,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "depositStake",
      accounts: [
        {
          name: "stakePool",
          isMut: true,
          isSigner: false,
        },
        {
          name: "validatorListStorage",
          isMut: true,
          isSigner: false,
        },
        {
          name: "stakePoolDepositAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "stakePoolWithdrawAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "depositStakeAddress",
          isMut: true,
          isSigner: false,
        },
        {
          name: "validatorStakeAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserveStakeAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "poolTokensTo",
          isMut: true,
          isSigner: false,
        },
        {
          name: "managerFeeAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "referrerPoolTokensAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "poolMint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "clock",
          isMut: false,
          isSigner: false,
        },
        {
          name: "sysvarStakeHistory",
          isMut: false,
          isSigner: false,
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "stakeProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "withdrawStake",
      accounts: [
        {
          name: "stakePool",
          isMut: true,
          isSigner: false,
        },
        {
          name: "validatorListStorage",
          isMut: true,
          isSigner: false,
        },
        {
          name: "stakePoolWithdraw",
          isMut: false,
          isSigner: false,
        },
        {
          name: "stakeToSplit",
          isMut: true,
          isSigner: false,
        },
        {
          name: "stakeToReceive",
          isMut: true,
          isSigner: false,
        },
        {
          name: "userStakeAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "userTransferAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "userPoolTokenAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "managerFeeAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "poolMint",
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
        {
          name: "stakeProgram",
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
      name: "setManager",
      accounts: [
        {
          name: "stakePool",
          isMut: true,
          isSigner: false,
        },
        {
          name: "manager",
          isMut: false,
          isSigner: true,
        },
        {
          name: "newManager",
          isMut: false,
          isSigner: true,
        },
        {
          name: "newFeeReceiver",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "setFee",
      accounts: [
        {
          name: "stakePool",
          isMut: true,
          isSigner: false,
        },
        {
          name: "manager",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [
        {
          name: "fee",
          type: {
            defined: "FeeType",
          },
        },
      ],
    },
    {
      name: "setStaker",
      accounts: [
        {
          name: "stakePool",
          isMut: true,
          isSigner: false,
        },
        {
          name: "setStakerAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "newStaker",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "depositSol",
      accounts: [
        {
          name: "stakePool",
          isMut: true,
          isSigner: false,
        },
        {
          name: "stakePoolWithdrawAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "reserveStakeAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "lamportsFrom",
          isMut: true,
          isSigner: true,
        },
        {
          name: "poolTokensTo",
          isMut: true,
          isSigner: false,
        },
        {
          name: "managerFeeAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "referrerPoolTokensAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "poolMint",
          isMut: true,
          isSigner: false,
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
          name: "arg",
          type: "u64",
        },
      ],
    },
    {
      name: "setFundingAuthority",
      accounts: [
        {
          name: "stakePool",
          isMut: true,
          isSigner: false,
        },
        {
          name: "manager",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [
        {
          name: "arg",
          type: {
            defined: "FundingType",
          },
        },
      ],
    },
    {
      name: "withdrawSol",
      accounts: [
        {
          name: "stakePool",
          isMut: true,
          isSigner: false,
        },
        {
          name: "stakePoolWithdrawAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "userTransferAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "poolTokensFrom",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserveStakeAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "lamportsTo",
          isMut: true,
          isSigner: false,
        },
        {
          name: "managerFeeAccount",
          isMut: true,
          isSigner: false,
        },
        {
          name: "poolMint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "clock",
          isMut: false,
          isSigner: false,
        },
        {
          name: "sysvarStakeHistory",
          isMut: false,
          isSigner: false,
        },
        {
          name: "stakeProgram",
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
      name: "createTokenMetadata",
      accounts: [
        {
          name: "stakePool",
          isMut: false,
          isSigner: false,
        },
        {
          name: "manager",
          isMut: false,
          isSigner: true,
        },
        {
          name: "stakePoolWithdrawAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "poolMint",
          isMut: false,
          isSigner: false,
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "tokenMetadata",
          isMut: true,
          isSigner: false,
        },
        {
          name: "mplTokenMetadata",
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
          name: "name",
          type: "string",
        },
        {
          name: "symbol",
          type: "string",
        },
        {
          name: "uri",
          type: "string",
        },
      ],
    },
    {
      name: "updateTokenMetadata",
      accounts: [
        {
          name: "stakePool",
          isMut: false,
          isSigner: false,
        },
        {
          name: "manager",
          isMut: false,
          isSigner: true,
        },
        {
          name: "stakePoolWithdrawAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "tokenMetadata",
          isMut: true,
          isSigner: false,
        },
        {
          name: "mplTokenMetadata",
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
          name: "symbol",
          type: "string",
        },
        {
          name: "uri",
          type: "string",
        },
      ],
    },
  ],
  accounts: [
    {
      name: "stakePool",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "AccountType",
            },
          },
          {
            name: "manager",
            type: "publicKey",
          },
          {
            name: "staker",
            type: "publicKey",
          },
          {
            name: "stakeDepositAuthority",
            type: "publicKey",
          },
          {
            name: "stakeWithdrawBumpSeed",
            type: "u8",
          },
          {
            name: "validatorList",
            type: "publicKey",
          },
          {
            name: "reserveStake",
            type: "publicKey",
          },
          {
            name: "poolMint",
            type: "publicKey",
          },
          {
            name: "managerFeeAccount",
            type: "publicKey",
          },
          {
            name: "tokenProgramId",
            type: "publicKey",
          },
          {
            name: "totalLamports",
            type: "u64",
          },
          {
            name: "poolTokenSupply",
            type: "u64",
          },
          {
            name: "lastUpdateEpoch",
            type: "u64",
          },
          {
            name: "lockup",
            type: {
              defined: "Lockup",
            },
          },
          {
            name: "epochFee",
            type: {
              defined: "Fee",
            },
          },
          {
            name: "nextEpochFee",
            type: {
              option: {
                defined: "Fee",
              },
            },
          },
          {
            name: "preferredDepositValidatorVoteAddress",
            type: {
              option: "publicKey",
            },
          },
          {
            name: "preferredWithdrawValidatorVoteAddress",
            type: {
              option: "publicKey",
            },
          },
          {
            name: "stakeDepositFee",
            type: {
              defined: "Fee",
            },
          },
          {
            name: "stakeWithdrawalFee",
            type: {
              defined: "Fee",
            },
          },
          {
            name: "nextStakeWithdrawalFee",
            type: {
              option: {
                defined: "Fee",
              },
            },
          },
          {
            name: "stakeReferralFee",
            type: "u8",
          },
          {
            name: "solDepositAuthority",
            type: {
              option: "publicKey",
            },
          },
          {
            name: "solDepositFee",
            type: {
              defined: "Fee",
            },
          },
          {
            name: "solReferralFee",
            type: "u8",
          },
          {
            name: "solWithdrawAuthority",
            type: {
              option: "publicKey",
            },
          },
          {
            name: "solWithdrawalFee",
            type: {
              defined: "Fee",
            },
          },
          {
            name: "nextSolWithdrawalFee",
            type: {
              option: {
                defined: "Fee",
              },
            },
          },
          {
            name: "lastEpochPoolTokenSupply",
            type: "u64",
          },
          {
            name: "lastEpochTotalLamports",
            type: "u64",
          },
        ],
      },
    },
    {
      name: "validatorStakeInfo",
      type: {
        kind: "struct",
        fields: [
          {
            name: "activeStakeLamports",
            type: "u64",
          },
          {
            name: "transientStakeLamports",
            type: "u64",
          },
          {
            name: "lastUpdateEpoch",
            type: "u64",
          },
          {
            name: "transientSeedSuffixStart",
            type: "u64",
          },
          {
            name: "transientSeedSuffixEnd",
            type: "u64",
          },
          {
            name: "status",
            type: {
              defined: "StakeStatus",
            },
          },
          {
            name: "voteAccountAddress",
            type: "publicKey",
          },
        ],
      },
    },
    {
      name: "validatorList",
      type: {
        kind: "struct",
        fields: [
          {
            name: "header",
            type: {
              defined: "ValidatorListHeader",
            },
          },
          {
            name: "validators",
            type: {
              vec: {
                defined: "ValidatorStakeInfo",
              },
            },
          },
        ],
      },
    },
  ],
  types: [
    {
      name: "Fee",
      type: {
        kind: "struct",
        fields: [
          {
            name: "denominator",
            type: "u64",
          },
          {
            name: "numerator",
            type: "u64",
          },
        ],
      },
    },
    {
      name: "ValidatorListHeader",
      type: {
        kind: "struct",
        fields: [
          {
            name: "accountType",
            type: {
              defined: "AccountType",
            },
          },
          {
            name: "maxValidators",
            type: "u32",
          },
        ],
      },
    },
    {
      name: "AccountType",
      type: {
        kind: "enum",
        variants: [
          {
            name: "Uninitialized",
          },
          {
            name: "StakePool",
          },
          {
            name: "ValidatorList",
          },
        ],
      },
    },
    {
      name: "StakeStatus",
      type: {
        kind: "enum",
        variants: [
          {
            name: "Active",
          },
          {
            name: "DeactivatingTransient",
          },
          {
            name: "ReadyForRemoval",
          },
        ],
      },
    },
    {
      name: "PreferredValidatorType",
      type: {
        kind: "enum",
        variants: [
          {
            name: "Deposit",
          },
          {
            name: "Withdraw",
          },
        ],
      },
    },
    {
      name: "FeeType",
      type: {
        kind: "enum",
        variants: [
          {
            name: "SolReferral",
            fields: ["u8"],
          },
          {
            name: "StakeReferral",
            fields: ["u8"],
          },
          {
            name: "Epoch",
            fields: [
              {
                defined: "Fee",
              },
            ],
          },
          {
            name: "StakeWithdrawal",
            fields: [
              {
                defined: "Fee",
              },
            ],
          },
          {
            name: "SolDeposit",
            fields: [
              {
                defined: "Fee",
              },
            ],
          },
          {
            name: "StakeDeposit",
            fields: [
              {
                defined: "Fee",
              },
            ],
          },
          {
            name: "SolWithdrawal",
            fields: [
              {
                defined: "Fee",
              },
            ],
          },
        ],
      },
    },
    {
      name: "FundingType",
      type: {
        kind: "enum",
        variants: [
          {
            name: "StakeDeposit",
          },
          {
            name: "SolDeposit",
          },
          {
            name: "SolWithdraw",
          },
        ],
      },
    },
  ],
  errors: [
    {
      code: 0,
      name: "AlreadyInUse",
      msg: "AlreadyInUse",
    },
    {
      code: 1,
      name: "InvalidProgramAddress",
      msg: "InvalidProgramAddress",
    },
    {
      code: 2,
      name: "InvalidState",
      msg: "InvalidState",
    },
    {
      code: 3,
      name: "CalculationFailure",
      msg: "CalculationFailure",
    },
    {
      code: 4,
      name: "FeeTooHigh",
      msg: "FeeTooHigh",
    },
    {
      code: 5,
      name: "WrongAccountMint",
      msg: "WrongAccountMint",
    },
    {
      code: 6,
      name: "WrongManager",
      msg: "WrongManager",
    },
    {
      code: 7,
      name: "SignatureMissing",
      msg: "SignatureMissing",
    },
    {
      code: 8,
      name: "InvalidValidatorStakeList",
      msg: "InvalidValidatorStakeList",
    },
    {
      code: 9,
      name: "InvalidFeeAccount",
      msg: "InvalidFeeAccount",
    },
    {
      code: 10,
      name: "WrongPoolMint",
      msg: "WrongPoolMint",
    },
    {
      code: 11,
      name: "WrongStakeState",
      msg: "WrongStakeState",
    },
    {
      code: 12,
      name: "UserStakeNotActive",
      msg: "UserStakeNotActive",
    },
    {
      code: 13,
      name: "ValidatorAlreadyAdded",
      msg: "ValidatorAlreadyAdded",
    },
    {
      code: 14,
      name: "ValidatorNotFound",
      msg: "ValidatorNotFound",
    },
    {
      code: 15,
      name: "InvalidStakeAccountAddress",
      msg: "InvalidStakeAccountAddress",
    },
    {
      code: 16,
      name: "StakeListOutOfDate",
      msg: "StakeListOutOfDate",
    },
    {
      code: 17,
      name: "StakeListAndPoolOutOfDate",
      msg: "StakeListAndPoolOutOfDate",
    },
    {
      code: 18,
      name: "UnknownValidatorStakeAccount",
      msg: "UnknownValidatorStakeAccount",
    },
    {
      code: 19,
      name: "WrongMintingAuthority",
      msg: "WrongMintingAuthority",
    },
    {
      code: 20,
      name: "UnexpectedValidatorListAccountSize",
      msg: "UnexpectedValidatorListAccountSize",
    },
    {
      code: 21,
      name: "WrongStaker",
      msg: "WrongStaker",
    },
    {
      code: 22,
      name: "NonZeroPoolTokenSupply",
      msg: "NonZeroPoolTokenSupply",
    },
    {
      code: 23,
      name: "StakeLamportsNotEqualToMinimum",
      msg: "StakeLamportsNotEqualToMinimum",
    },
    {
      code: 24,
      name: "IncorrectDepositVoteAddress",
      msg: "IncorrectDepositVoteAddress",
    },
    {
      code: 25,
      name: "IncorrectWithdrawVoteAddress",
      msg: "IncorrectWithdrawVoteAddress",
    },
    {
      code: 26,
      name: "InvalidMintFreezeAuthority",
      msg: "InvalidMintFreezeAuthority",
    },
    {
      code: 27,
      name: "FeeIncreaseTooHigh",
      msg: "FeeIncreaseTooHigh",
    },
    {
      code: 28,
      name: "WithdrawalTooSmall",
      msg: "WithdrawalTooSmall",
    },
    {
      code: 29,
      name: "DepositTooSmall",
      msg: "DepositTooSmall",
    },
    {
      code: 30,
      name: "InvalidStakeDepositAuthority",
      msg: "InvalidStakeDepositAuthority",
    },
    {
      code: 31,
      name: "InvalidSolDepositAuthority",
      msg: "InvalidSolDepositAuthority",
    },
    {
      code: 32,
      name: "InvalidPreferredValidator",
      msg: "InvalidPreferredValidator",
    },
    {
      code: 33,
      name: "TransientAccountInUse",
      msg: "TransientAccountInUse",
    },
    {
      code: 34,
      name: "InvalidSolWithdrawAuthority",
      msg: "InvalidSolWithdrawAuthority",
    },
    {
      code: 35,
      name: "SolWithdrawalTooLarge",
      msg: "SolWithdrawalTooLarge",
    },
    {
      code: 36,
      name: "InvalidMetadataAccount",
      msg: "InvalidMetadataAccount",
    },
  ],
};
