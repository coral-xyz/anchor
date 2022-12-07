import { PublicKey } from "@solana/web3.js";
import { Program, AnchorProvider } from "@project-serum/anchor";

import { SplTokenLendingCoder } from "./coder";

export const SPL_TOKEN_LENDING_PROGRAM_ID = new PublicKey(
  "FJAwitEMXUEUibVHXXwpikL7Ct1xTujaY2XMtccUBSoK"
);

interface GetProgramParams {
  programId?: PublicKey;
  provider?: AnchorProvider;
}

export function splTokenLendingProgram(
  params?: GetProgramParams
): Program<SplTokenLending> {
  return new Program<SplTokenLending>(
    IDL,
    params?.programId ?? SPL_TOKEN_LENDING_PROGRAM_ID,
    params?.provider,
    new SplTokenLendingCoder(IDL)
  );
}

type SplTokenLending = {
  version: "0.2.0";
  name: "spl_token_lending";
  instructions: [
    {
      name: "initLendingMarket";
      accounts: [
        {
          name: "lendingMarket";
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
        },
        {
          name: "oracleProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "owner";
          type: "publicKey";
        },
        {
          name: "quoteCurrency";
          type: {
            array: ["u8", 32];
          };
        }
      ];
    },
    {
      name: "setLendingMarketOwner";
      accounts: [
        {
          name: "lendingMarket";
          isMut: true;
          isSigner: false;
        },
        {
          name: "lendingMarketOwner";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [
        {
          name: "newOwner";
          type: "publicKey";
        }
      ];
    },
    {
      name: "initReserve";
      accounts: [
        {
          name: "sourceLiquidity";
          isMut: true;
          isSigner: false;
        },
        {
          name: "destinationCollateral";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserve";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserveLiquidityMint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "reserveLiquiditySupply";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserveLiquidityFeeReceiver";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserveCollateralMint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserveCollateralSupply";
          isMut: true;
          isSigner: false;
        },
        {
          name: "pythProduct";
          isMut: false;
          isSigner: false;
        },
        {
          name: "pythPrice";
          isMut: false;
          isSigner: false;
        },
        {
          name: "lendingMarket";
          isMut: false;
          isSigner: false;
        },
        {
          name: "lendingMarketAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "lendingMarketOwner";
          isMut: false;
          isSigner: true;
        },
        {
          name: "userTransferAuthority";
          isMut: false;
          isSigner: true;
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
          name: "tokenProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "liquidityAmount";
          type: "u64";
        },
        {
          name: "config";
          type: {
            defined: "ReserveConfig";
          };
        }
      ];
    },
    {
      name: "refreshReserve";
      accounts: [
        {
          name: "reserve";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserveLiquidityOracle";
          isMut: false;
          isSigner: false;
        },
        {
          name: "clock";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "depositReserveLiquidity";
      accounts: [
        {
          name: "sourceLiquidity";
          isMut: true;
          isSigner: false;
        },
        {
          name: "destinationCollateral";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserve";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserveLiquiditySupply";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserveCollateralMint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "lendingMarket";
          isMut: false;
          isSigner: false;
        },
        {
          name: "lendingMarketAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "userTransferAuthority";
          isMut: false;
          isSigner: true;
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
          name: "liquidityAmount";
          type: "u64";
        }
      ];
    },
    {
      name: "redeemReserveCollateral";
      accounts: [
        {
          name: "sourceCollateral";
          isMut: true;
          isSigner: false;
        },
        {
          name: "destinationLiquidity";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserve";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserveCollateralMint";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserveLiquiditySupply";
          isMut: true;
          isSigner: false;
        },
        {
          name: "lendingMarket";
          isMut: false;
          isSigner: false;
        },
        {
          name: "lendingMarketAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "userTransferAuthority";
          isMut: false;
          isSigner: true;
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
          name: "collateralAmount";
          type: "u64";
        }
      ];
    },
    {
      name: "initObligation";
      accounts: [
        {
          name: "obligation";
          isMut: true;
          isSigner: false;
        },
        {
          name: "lendingMarket";
          isMut: false;
          isSigner: false;
        },
        {
          name: "obligationOwner";
          isMut: false;
          isSigner: true;
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
          name: "tokenProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "refreshObligation";
      accounts: [
        {
          name: "obligation";
          isMut: true;
          isSigner: false;
        },
        {
          name: "clock";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "depositObligationCollateral";
      accounts: [
        {
          name: "sourceCollateral";
          isMut: true;
          isSigner: false;
        },
        {
          name: "destinationCollateral";
          isMut: true;
          isSigner: false;
        },
        {
          name: "depositReserve";
          isMut: false;
          isSigner: false;
        },
        {
          name: "obligation";
          isMut: true;
          isSigner: false;
        },
        {
          name: "lendingMarket";
          isMut: false;
          isSigner: false;
        },
        {
          name: "obligationOwner";
          isMut: false;
          isSigner: true;
        },
        {
          name: "userTransferAuthority";
          isMut: false;
          isSigner: true;
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
          name: "collateralAmount";
          type: "u64";
        }
      ];
    },
    {
      name: "withdrawObligationCollateral";
      accounts: [
        {
          name: "sourceCollateral";
          isMut: true;
          isSigner: false;
        },
        {
          name: "destinationCollateral";
          isMut: true;
          isSigner: false;
        },
        {
          name: "withdrawReserve";
          isMut: false;
          isSigner: false;
        },
        {
          name: "obligation";
          isMut: true;
          isSigner: false;
        },
        {
          name: "lendingMarket";
          isMut: false;
          isSigner: false;
        },
        {
          name: "lendingMarketAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "obligationOwner";
          isMut: false;
          isSigner: true;
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
          name: "collateralAmount";
          type: "u64";
        }
      ];
    },
    {
      name: "borrowObligationLiquidity";
      accounts: [
        {
          name: "sourceLiquidity";
          isMut: true;
          isSigner: false;
        },
        {
          name: "destinationLiquidity";
          isMut: true;
          isSigner: false;
        },
        {
          name: "borrowReserve";
          isMut: true;
          isSigner: false;
        },
        {
          name: "borrowReserveLiquidityFeeReceiver";
          isMut: true;
          isSigner: false;
        },
        {
          name: "obligation";
          isMut: true;
          isSigner: false;
        },
        {
          name: "lendingMarket";
          isMut: false;
          isSigner: false;
        },
        {
          name: "lendingMarketAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "obligationOwner";
          isMut: false;
          isSigner: true;
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
          name: "liquidityAmount";
          type: "u64";
        }
      ];
    },
    {
      name: "repayObligationLiquidity";
      accounts: [
        {
          name: "sourceLiquidity";
          isMut: true;
          isSigner: false;
        },
        {
          name: "destinationLiquidity";
          isMut: true;
          isSigner: false;
        },
        {
          name: "repayReserve";
          isMut: true;
          isSigner: false;
        },
        {
          name: "obligation";
          isMut: true;
          isSigner: false;
        },
        {
          name: "lendingMarket";
          isMut: false;
          isSigner: false;
        },
        {
          name: "userTransferAuthority";
          isMut: false;
          isSigner: true;
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
          name: "liquidityAmount";
          type: "u64";
        }
      ];
    },
    {
      name: "liquidateObligation";
      accounts: [
        {
          name: "sourceLiquidity";
          isMut: true;
          isSigner: false;
        },
        {
          name: "destinationCollateral";
          isMut: true;
          isSigner: false;
        },
        {
          name: "repayReserve";
          isMut: true;
          isSigner: false;
        },
        {
          name: "repayReserveLiquiditySupply";
          isMut: true;
          isSigner: false;
        },
        {
          name: "withdrawReserve";
          isMut: false;
          isSigner: false;
        },
        {
          name: "withdrawReserveCollateralSupply";
          isMut: true;
          isSigner: false;
        },
        {
          name: "obligation";
          isMut: true;
          isSigner: false;
        },
        {
          name: "lendingMarket";
          isMut: false;
          isSigner: false;
        },
        {
          name: "lendingMarketAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "userTransferAuthority";
          isMut: false;
          isSigner: true;
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
          name: "liquidityAmount";
          type: "u64";
        }
      ];
    },
    {
      name: "flashLoan";
      accounts: [
        {
          name: "sourceLiquidity";
          isMut: true;
          isSigner: false;
        },
        {
          name: "destinationLiquidity";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserve";
          isMut: true;
          isSigner: false;
        },
        {
          name: "reserveLiquidityFeeReceiver";
          isMut: true;
          isSigner: false;
        },
        {
          name: "hostFeeReceiver";
          isMut: true;
          isSigner: false;
        },
        {
          name: "lendingMarket";
          isMut: false;
          isSigner: false;
        },
        {
          name: "lendingMarketAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "tokenProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "flashLoanReceiverProgram";
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
    }
  ];
  accounts: [
    {
      name: "obligation";
      type: {
        kind: "struct";
        fields: [
          {
            name: "version";
            type: "u8";
          },
          {
            name: "lastUpdate";
            type: {
              defined: "LastUpdate";
            };
          },
          {
            name: "lendingMarket";
            type: "publicKey";
          },
          {
            name: "owner";
            type: "publicKey";
          },
          {
            name: "deposits";
            type: {
              vec: {
                defined: "ObligationCollateral";
              };
            };
          },
          {
            name: "borrows";
            type: {
              vec: {
                defined: "ObligationLiquidity";
              };
            };
          },
          {
            name: "depositedValue";
            type: {
              defined: "Decimal";
            };
          },
          {
            name: "borrowedValue";
            type: {
              defined: "Decimal";
            };
          },
          {
            name: "allowedBorrowValue";
            type: {
              defined: "Decimal";
            };
          },
          {
            name: "unhealthyBorrowValue";
            type: {
              defined: "Decimal";
            };
          }
        ];
      };
    },
    {
      name: "lendingMarket";
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
            name: "owner";
            type: "publicKey";
          },
          {
            name: "quoteCurrency";
            type: {
              array: ["u8", 32];
            };
          },
          {
            name: "tokenProgramId";
            type: "publicKey";
          },
          {
            name: "oracleProgramId";
            type: "publicKey";
          }
        ];
      };
    },
    {
      name: "reserve";
      type: {
        kind: "struct";
        fields: [
          {
            name: "version";
            type: "u8";
          },
          {
            name: "lastUpdate";
            type: {
              defined: "LastUpdate";
            };
          },
          {
            name: "lendingMarket";
            type: "publicKey";
          },
          {
            name: "liquidity";
            type: {
              defined: "ReserveLiquidity";
            };
          },
          {
            name: "collateral";
            type: {
              defined: "ReserveCollateral";
            };
          },
          {
            name: "config";
            type: {
              defined: "ReserveConfig";
            };
          }
        ];
      };
    }
  ];
  types: [
    {
      name: "LastUpdate";
      type: {
        kind: "struct";
        fields: [
          {
            name: "slot";
            type: "u64";
          },
          {
            name: "stale";
            type: "bool";
          }
        ];
      };
    },
    {
      name: "ObligationCollateral";
      type: {
        kind: "struct";
        fields: [
          {
            name: "depositReserve";
            type: "publicKey";
          },
          {
            name: "depositedAmount";
            type: "u64";
          },
          {
            name: "marketValue";
            type: {
              defined: "Decimal";
            };
          }
        ];
      };
    },
    {
      name: "ObligationLiquidity";
      type: {
        kind: "struct";
        fields: [
          {
            name: "borrowReserve";
            type: "publicKey";
          },
          {
            name: "cumulativeBorrowRateWads";
            type: {
              defined: "Decimal";
            };
          },
          {
            name: "borrowedAmountWads";
            type: {
              defined: "Decimal";
            };
          },
          {
            name: "marketValue";
            type: {
              defined: "Decimal";
            };
          }
        ];
      };
    },
    {
      name: "ReserveLiquidity";
      type: {
        kind: "struct";
        fields: [
          {
            name: "mintPubkey";
            type: "publicKey";
          },
          {
            name: "mintDecimals";
            type: "u8";
          },
          {
            name: "supplyPubkey";
            type: "publicKey";
          },
          {
            name: "feeReceiver";
            type: "publicKey";
          },
          {
            name: "oraclePubkey";
            type: "publicKey";
          },
          {
            name: "availableAmount";
            type: "u64";
          },
          {
            name: "borrowedAmountWads";
            type: {
              defined: "Decimal";
            };
          },
          {
            name: "cumulativeBorrowRateWads";
            type: {
              defined: "Decimal";
            };
          },
          {
            name: "marketPrice";
            type: {
              defined: "Decimal";
            };
          }
        ];
      };
    },
    {
      name: "ReserveCollateral";
      type: {
        kind: "struct";
        fields: [
          {
            name: "mintPubkey";
            type: "publicKey";
          },
          {
            name: "mintTotalSupply";
            type: "u64";
          },
          {
            name: "supplyPubkey";
            type: "publicKey";
          }
        ];
      };
    },
    {
      name: "ReserveFees";
      type: {
        kind: "struct";
        fields: [
          {
            name: "borrowFeeWad";
            type: "u64";
          },
          {
            name: "flashLoanFeeWad";
            type: "u64";
          },
          {
            name: "hostFeePercentage";
            type: "u8";
          }
        ];
      };
    },
    {
      name: "ReserveConfig";
      type: {
        kind: "struct";
        fields: [
          {
            name: "optimalUtilizationRate";
            type: "u8";
          },
          {
            name: "loanToValueRatio";
            type: "u8";
          },
          {
            name: "liquidationBonus";
            type: "u8";
          },
          {
            name: "liquidationThreshold";
            type: "u8";
          },
          {
            name: "minBorrowRate";
            type: "u8";
          },
          {
            name: "optimalBorrowRate";
            type: "u8";
          },
          {
            name: "maxBorrowRate";
            type: "u8";
          },
          {
            name: "fees";
            type: {
              defined: "ReserveFees";
            };
          }
        ];
      };
    }
  ];
  errors: [
    {
      code: 0;
      name: "InstructionUnpackError";
      msg: "Failed to unpack instruction data";
    },
    {
      code: 1;
      name: "AlreadyInitialized";
      msg: "Account is already initialized";
    },
    {
      code: 2;
      name: "NotRentExempt";
      msg: "Lamport balance below rent-exempt threshold";
    },
    {
      code: 3;
      name: "InvalidMarketAuthority";
      msg: "Market authority is invalid";
    },
    {
      code: 4;
      name: "InvalidMarketOwner";
      msg: "Market owner is invalid";
    },
    {
      code: 5;
      name: "InvalidAccountOwner";
      msg: "Input account owner is not the program address";
    },
    {
      code: 6;
      name: "InvalidTokenOwner";
      msg: "Input token account is not owned by the correct token program id";
    },
    {
      code: 7;
      name: "InvalidTokenAccount";
      msg: "Input token account is not valid";
    },
    {
      code: 8;
      name: "InvalidTokenMint";
      msg: "Input token mint account is not valid";
    },
    {
      code: 9;
      name: "InvalidTokenProgram";
      msg: "Input token program account is not valid";
    },
    {
      code: 10;
      name: "InvalidAmount";
      msg: "Input amount is invalid";
    },
    {
      code: 11;
      name: "InvalidConfig";
      msg: "Input config value is invalid";
    },
    {
      code: 12;
      name: "InvalidSigner";
      msg: "Input account must be a signer";
    },
    {
      code: 13;
      name: "InvalidAccountInput";
      msg: "Invalid account input";
    },
    {
      code: 14;
      name: "MathOverflow";
      msg: "Math operation overflow";
    },
    {
      code: 15;
      name: "TokenInitializeMintFailed";
      msg: "Token initialize mint failed";
    },
    {
      code: 16;
      name: "TokenInitializeAccountFailed";
      msg: "Token initialize account failed";
    },
    {
      code: 17;
      name: "TokenTransferFailed";
      msg: "Token transfer failed";
    },
    {
      code: 18;
      name: "TokenMintToFailed";
      msg: "Token mint to failed";
    },
    {
      code: 19;
      name: "TokenBurnFailed";
      msg: "Token burn failed";
    },
    {
      code: 20;
      name: "InsufficientLiquidity";
      msg: "Insufficient liquidity available";
    },
    {
      code: 21;
      name: "ReserveCollateralDisabled";
      msg: "Input reserve has collateral disabled";
    },
    {
      code: 22;
      name: "ReserveStale";
      msg: "Reserve state needs to be refreshed";
    },
    {
      code: 23;
      name: "WithdrawTooSmall";
      msg: "Withdraw amount too small";
    },
    {
      code: 24;
      name: "WithdrawTooLarge";
      msg: "Withdraw amount too large";
    },
    {
      code: 25;
      name: "BorrowTooSmall";
      msg: "Borrow amount too small to receive liquidity after fees";
    },
    {
      code: 26;
      name: "BorrowTooLarge";
      msg: "Borrow amount too large for deposited collateral";
    },
    {
      code: 27;
      name: "RepayTooSmall";
      msg: "Repay amount too small to transfer liquidity";
    },
    {
      code: 28;
      name: "LiquidationTooSmall";
      msg: "Liquidation amount too small to receive collateral";
    },
    {
      code: 29;
      name: "ObligationHealthy";
      msg: "Cannot liquidate healthy obligations";
    },
    {
      code: 30;
      name: "ObligationStale";
      msg: "Obligation state needs to be refreshed";
    },
    {
      code: 31;
      name: "ObligationReserveLimit";
      msg: "Obligation reserve limit exceeded";
    },
    {
      code: 32;
      name: "InvalidObligationOwner";
      msg: "Obligation owner is invalid";
    },
    {
      code: 33;
      name: "ObligationDepositsEmpty";
      msg: "Obligation deposits are empty";
    },
    {
      code: 34;
      name: "ObligationBorrowsEmpty";
      msg: "Obligation borrows are empty";
    },
    {
      code: 35;
      name: "ObligationDepositsZero";
      msg: "Obligation deposits have zero value";
    },
    {
      code: 36;
      name: "ObligationBorrowsZero";
      msg: "Obligation borrows have zero value";
    },
    {
      code: 37;
      name: "InvalidObligationCollateral";
      msg: "Invalid obligation collateral";
    },
    {
      code: 38;
      name: "InvalidObligationLiquidity";
      msg: "Invalid obligation liquidity";
    },
    {
      code: 39;
      name: "ObligationCollateralEmpty";
      msg: "Obligation collateral is empty";
    },
    {
      code: 40;
      name: "ObligationLiquidityEmpty";
      msg: "Obligation liquidity is empty";
    },
    {
      code: 41;
      name: "NegativeInterestRate";
      msg: "Interest rate is negative";
    },
    {
      code: 42;
      name: "InvalidOracleConfig";
      msg: "Input oracle config is invalid";
    },
    {
      code: 43;
      name: "InvalidFlashLoanReceiverProgram";
      msg: "Input flash loan receiver program account is not valid";
    },
    {
      code: 44;
      name: "NotEnoughLiquidityAfterFlashLoan";
      msg: "Not enough liquidity after flash loan";
    }
  ];
};

const IDL: SplTokenLending = {
  version: "0.2.0",
  name: "spl_token_lending",
  instructions: [
    {
      name: "initLendingMarket",
      accounts: [
        {
          name: "lendingMarket",
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
        {
          name: "oracleProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "owner",
          type: "publicKey",
        },
        {
          name: "quoteCurrency",
          type: {
            array: ["u8", 32],
          },
        },
      ],
    },
    {
      name: "setLendingMarketOwner",
      accounts: [
        {
          name: "lendingMarket",
          isMut: true,
          isSigner: false,
        },
        {
          name: "lendingMarketOwner",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [
        {
          name: "newOwner",
          type: "publicKey",
        },
      ],
    },
    {
      name: "initReserve",
      accounts: [
        {
          name: "sourceLiquidity",
          isMut: true,
          isSigner: false,
        },
        {
          name: "destinationCollateral",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserve",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserveLiquidityMint",
          isMut: false,
          isSigner: false,
        },
        {
          name: "reserveLiquiditySupply",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserveLiquidityFeeReceiver",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserveCollateralMint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserveCollateralSupply",
          isMut: true,
          isSigner: false,
        },
        {
          name: "pythProduct",
          isMut: false,
          isSigner: false,
        },
        {
          name: "pythPrice",
          isMut: false,
          isSigner: false,
        },
        {
          name: "lendingMarket",
          isMut: false,
          isSigner: false,
        },
        {
          name: "lendingMarketAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "lendingMarketOwner",
          isMut: false,
          isSigner: true,
        },
        {
          name: "userTransferAuthority",
          isMut: false,
          isSigner: true,
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
          name: "tokenProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "liquidityAmount",
          type: "u64",
        },
        {
          name: "config",
          type: {
            defined: "ReserveConfig",
          },
        },
      ],
    },
    {
      name: "refreshReserve",
      accounts: [
        {
          name: "reserve",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserveLiquidityOracle",
          isMut: false,
          isSigner: false,
        },
        {
          name: "clock",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "depositReserveLiquidity",
      accounts: [
        {
          name: "sourceLiquidity",
          isMut: true,
          isSigner: false,
        },
        {
          name: "destinationCollateral",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserve",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserveLiquiditySupply",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserveCollateralMint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "lendingMarket",
          isMut: false,
          isSigner: false,
        },
        {
          name: "lendingMarketAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "userTransferAuthority",
          isMut: false,
          isSigner: true,
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
          name: "liquidityAmount",
          type: "u64",
        },
      ],
    },
    {
      name: "redeemReserveCollateral",
      accounts: [
        {
          name: "sourceCollateral",
          isMut: true,
          isSigner: false,
        },
        {
          name: "destinationLiquidity",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserve",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserveCollateralMint",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserveLiquiditySupply",
          isMut: true,
          isSigner: false,
        },
        {
          name: "lendingMarket",
          isMut: false,
          isSigner: false,
        },
        {
          name: "lendingMarketAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "userTransferAuthority",
          isMut: false,
          isSigner: true,
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
          name: "collateralAmount",
          type: "u64",
        },
      ],
    },
    {
      name: "initObligation",
      accounts: [
        {
          name: "obligation",
          isMut: true,
          isSigner: false,
        },
        {
          name: "lendingMarket",
          isMut: false,
          isSigner: false,
        },
        {
          name: "obligationOwner",
          isMut: false,
          isSigner: true,
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
          name: "tokenProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "refreshObligation",
      accounts: [
        {
          name: "obligation",
          isMut: true,
          isSigner: false,
        },
        {
          name: "clock",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "depositObligationCollateral",
      accounts: [
        {
          name: "sourceCollateral",
          isMut: true,
          isSigner: false,
        },
        {
          name: "destinationCollateral",
          isMut: true,
          isSigner: false,
        },
        {
          name: "depositReserve",
          isMut: false,
          isSigner: false,
        },
        {
          name: "obligation",
          isMut: true,
          isSigner: false,
        },
        {
          name: "lendingMarket",
          isMut: false,
          isSigner: false,
        },
        {
          name: "obligationOwner",
          isMut: false,
          isSigner: true,
        },
        {
          name: "userTransferAuthority",
          isMut: false,
          isSigner: true,
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
          name: "collateralAmount",
          type: "u64",
        },
      ],
    },
    {
      name: "withdrawObligationCollateral",
      accounts: [
        {
          name: "sourceCollateral",
          isMut: true,
          isSigner: false,
        },
        {
          name: "destinationCollateral",
          isMut: true,
          isSigner: false,
        },
        {
          name: "withdrawReserve",
          isMut: false,
          isSigner: false,
        },
        {
          name: "obligation",
          isMut: true,
          isSigner: false,
        },
        {
          name: "lendingMarket",
          isMut: false,
          isSigner: false,
        },
        {
          name: "lendingMarketAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "obligationOwner",
          isMut: false,
          isSigner: true,
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
          name: "collateralAmount",
          type: "u64",
        },
      ],
    },
    {
      name: "borrowObligationLiquidity",
      accounts: [
        {
          name: "sourceLiquidity",
          isMut: true,
          isSigner: false,
        },
        {
          name: "destinationLiquidity",
          isMut: true,
          isSigner: false,
        },
        {
          name: "borrowReserve",
          isMut: true,
          isSigner: false,
        },
        {
          name: "borrowReserveLiquidityFeeReceiver",
          isMut: true,
          isSigner: false,
        },
        {
          name: "obligation",
          isMut: true,
          isSigner: false,
        },
        {
          name: "lendingMarket",
          isMut: false,
          isSigner: false,
        },
        {
          name: "lendingMarketAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "obligationOwner",
          isMut: false,
          isSigner: true,
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
          name: "liquidityAmount",
          type: "u64",
        },
      ],
    },
    {
      name: "repayObligationLiquidity",
      accounts: [
        {
          name: "sourceLiquidity",
          isMut: true,
          isSigner: false,
        },
        {
          name: "destinationLiquidity",
          isMut: true,
          isSigner: false,
        },
        {
          name: "repayReserve",
          isMut: true,
          isSigner: false,
        },
        {
          name: "obligation",
          isMut: true,
          isSigner: false,
        },
        {
          name: "lendingMarket",
          isMut: false,
          isSigner: false,
        },
        {
          name: "userTransferAuthority",
          isMut: false,
          isSigner: true,
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
          name: "liquidityAmount",
          type: "u64",
        },
      ],
    },
    {
      name: "liquidateObligation",
      accounts: [
        {
          name: "sourceLiquidity",
          isMut: true,
          isSigner: false,
        },
        {
          name: "destinationCollateral",
          isMut: true,
          isSigner: false,
        },
        {
          name: "repayReserve",
          isMut: true,
          isSigner: false,
        },
        {
          name: "repayReserveLiquiditySupply",
          isMut: true,
          isSigner: false,
        },
        {
          name: "withdrawReserve",
          isMut: false,
          isSigner: false,
        },
        {
          name: "withdrawReserveCollateralSupply",
          isMut: true,
          isSigner: false,
        },
        {
          name: "obligation",
          isMut: true,
          isSigner: false,
        },
        {
          name: "lendingMarket",
          isMut: false,
          isSigner: false,
        },
        {
          name: "lendingMarketAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "userTransferAuthority",
          isMut: false,
          isSigner: true,
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
          name: "liquidityAmount",
          type: "u64",
        },
      ],
    },
    {
      name: "flashLoan",
      accounts: [
        {
          name: "sourceLiquidity",
          isMut: true,
          isSigner: false,
        },
        {
          name: "destinationLiquidity",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserve",
          isMut: true,
          isSigner: false,
        },
        {
          name: "reserveLiquidityFeeReceiver",
          isMut: true,
          isSigner: false,
        },
        {
          name: "hostFeeReceiver",
          isMut: true,
          isSigner: false,
        },
        {
          name: "lendingMarket",
          isMut: false,
          isSigner: false,
        },
        {
          name: "lendingMarketAuthority",
          isMut: false,
          isSigner: false,
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "flashLoanReceiverProgram",
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
  ],
  accounts: [
    {
      name: "obligation",
      type: {
        kind: "struct",
        fields: [
          {
            name: "version",
            type: "u8",
          },
          {
            name: "lastUpdate",
            type: {
              defined: "LastUpdate",
            },
          },
          {
            name: "lendingMarket",
            type: "publicKey",
          },
          {
            name: "owner",
            type: "publicKey",
          },
          {
            name: "deposits",
            type: {
              vec: {
                defined: "ObligationCollateral",
              },
            },
          },
          {
            name: "borrows",
            type: {
              vec: {
                defined: "ObligationLiquidity",
              },
            },
          },
          {
            name: "depositedValue",
            type: {
              defined: "Decimal",
            },
          },
          {
            name: "borrowedValue",
            type: {
              defined: "Decimal",
            },
          },
          {
            name: "allowedBorrowValue",
            type: {
              defined: "Decimal",
            },
          },
          {
            name: "unhealthyBorrowValue",
            type: {
              defined: "Decimal",
            },
          },
        ],
      },
    },
    {
      name: "lendingMarket",
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
            name: "owner",
            type: "publicKey",
          },
          {
            name: "quoteCurrency",
            type: {
              array: ["u8", 32],
            },
          },
          {
            name: "tokenProgramId",
            type: "publicKey",
          },
          {
            name: "oracleProgramId",
            type: "publicKey",
          },
        ],
      },
    },
    {
      name: "reserve",
      type: {
        kind: "struct",
        fields: [
          {
            name: "version",
            type: "u8",
          },
          {
            name: "lastUpdate",
            type: {
              defined: "LastUpdate",
            },
          },
          {
            name: "lendingMarket",
            type: "publicKey",
          },
          {
            name: "liquidity",
            type: {
              defined: "ReserveLiquidity",
            },
          },
          {
            name: "collateral",
            type: {
              defined: "ReserveCollateral",
            },
          },
          {
            name: "config",
            type: {
              defined: "ReserveConfig",
            },
          },
        ],
      },
    },
  ],
  types: [
    {
      name: "LastUpdate",
      type: {
        kind: "struct",
        fields: [
          {
            name: "slot",
            type: "u64",
          },
          {
            name: "stale",
            type: "bool",
          },
        ],
      },
    },
    {
      name: "ObligationCollateral",
      type: {
        kind: "struct",
        fields: [
          {
            name: "depositReserve",
            type: "publicKey",
          },
          {
            name: "depositedAmount",
            type: "u64",
          },
          {
            name: "marketValue",
            type: {
              defined: "Decimal",
            },
          },
        ],
      },
    },
    {
      name: "ObligationLiquidity",
      type: {
        kind: "struct",
        fields: [
          {
            name: "borrowReserve",
            type: "publicKey",
          },
          {
            name: "cumulativeBorrowRateWads",
            type: {
              defined: "Decimal",
            },
          },
          {
            name: "borrowedAmountWads",
            type: {
              defined: "Decimal",
            },
          },
          {
            name: "marketValue",
            type: {
              defined: "Decimal",
            },
          },
        ],
      },
    },
    {
      name: "ReserveLiquidity",
      type: {
        kind: "struct",
        fields: [
          {
            name: "mintPubkey",
            type: "publicKey",
          },
          {
            name: "mintDecimals",
            type: "u8",
          },
          {
            name: "supplyPubkey",
            type: "publicKey",
          },
          {
            name: "feeReceiver",
            type: "publicKey",
          },
          {
            name: "oraclePubkey",
            type: "publicKey",
          },
          {
            name: "availableAmount",
            type: "u64",
          },
          {
            name: "borrowedAmountWads",
            type: {
              defined: "Decimal",
            },
          },
          {
            name: "cumulativeBorrowRateWads",
            type: {
              defined: "Decimal",
            },
          },
          {
            name: "marketPrice",
            type: {
              defined: "Decimal",
            },
          },
        ],
      },
    },
    {
      name: "ReserveCollateral",
      type: {
        kind: "struct",
        fields: [
          {
            name: "mintPubkey",
            type: "publicKey",
          },
          {
            name: "mintTotalSupply",
            type: "u64",
          },
          {
            name: "supplyPubkey",
            type: "publicKey",
          },
        ],
      },
    },
    {
      name: "ReserveFees",
      type: {
        kind: "struct",
        fields: [
          {
            name: "borrowFeeWad",
            type: "u64",
          },
          {
            name: "flashLoanFeeWad",
            type: "u64",
          },
          {
            name: "hostFeePercentage",
            type: "u8",
          },
        ],
      },
    },
    {
      name: "ReserveConfig",
      type: {
        kind: "struct",
        fields: [
          {
            name: "optimalUtilizationRate",
            type: "u8",
          },
          {
            name: "loanToValueRatio",
            type: "u8",
          },
          {
            name: "liquidationBonus",
            type: "u8",
          },
          {
            name: "liquidationThreshold",
            type: "u8",
          },
          {
            name: "minBorrowRate",
            type: "u8",
          },
          {
            name: "optimalBorrowRate",
            type: "u8",
          },
          {
            name: "maxBorrowRate",
            type: "u8",
          },
          {
            name: "fees",
            type: {
              defined: "ReserveFees",
            },
          },
        ],
      },
    },
  ],
  errors: [
    {
      code: 0,
      name: "InstructionUnpackError",
      msg: "Failed to unpack instruction data",
    },
    {
      code: 1,
      name: "AlreadyInitialized",
      msg: "Account is already initialized",
    },
    {
      code: 2,
      name: "NotRentExempt",
      msg: "Lamport balance below rent-exempt threshold",
    },
    {
      code: 3,
      name: "InvalidMarketAuthority",
      msg: "Market authority is invalid",
    },
    {
      code: 4,
      name: "InvalidMarketOwner",
      msg: "Market owner is invalid",
    },
    {
      code: 5,
      name: "InvalidAccountOwner",
      msg: "Input account owner is not the program address",
    },
    {
      code: 6,
      name: "InvalidTokenOwner",
      msg: "Input token account is not owned by the correct token program id",
    },
    {
      code: 7,
      name: "InvalidTokenAccount",
      msg: "Input token account is not valid",
    },
    {
      code: 8,
      name: "InvalidTokenMint",
      msg: "Input token mint account is not valid",
    },
    {
      code: 9,
      name: "InvalidTokenProgram",
      msg: "Input token program account is not valid",
    },
    {
      code: 10,
      name: "InvalidAmount",
      msg: "Input amount is invalid",
    },
    {
      code: 11,
      name: "InvalidConfig",
      msg: "Input config value is invalid",
    },
    {
      code: 12,
      name: "InvalidSigner",
      msg: "Input account must be a signer",
    },
    {
      code: 13,
      name: "InvalidAccountInput",
      msg: "Invalid account input",
    },
    {
      code: 14,
      name: "MathOverflow",
      msg: "Math operation overflow",
    },
    {
      code: 15,
      name: "TokenInitializeMintFailed",
      msg: "Token initialize mint failed",
    },
    {
      code: 16,
      name: "TokenInitializeAccountFailed",
      msg: "Token initialize account failed",
    },
    {
      code: 17,
      name: "TokenTransferFailed",
      msg: "Token transfer failed",
    },
    {
      code: 18,
      name: "TokenMintToFailed",
      msg: "Token mint to failed",
    },
    {
      code: 19,
      name: "TokenBurnFailed",
      msg: "Token burn failed",
    },
    {
      code: 20,
      name: "InsufficientLiquidity",
      msg: "Insufficient liquidity available",
    },
    {
      code: 21,
      name: "ReserveCollateralDisabled",
      msg: "Input reserve has collateral disabled",
    },
    {
      code: 22,
      name: "ReserveStale",
      msg: "Reserve state needs to be refreshed",
    },
    {
      code: 23,
      name: "WithdrawTooSmall",
      msg: "Withdraw amount too small",
    },
    {
      code: 24,
      name: "WithdrawTooLarge",
      msg: "Withdraw amount too large",
    },
    {
      code: 25,
      name: "BorrowTooSmall",
      msg: "Borrow amount too small to receive liquidity after fees",
    },
    {
      code: 26,
      name: "BorrowTooLarge",
      msg: "Borrow amount too large for deposited collateral",
    },
    {
      code: 27,
      name: "RepayTooSmall",
      msg: "Repay amount too small to transfer liquidity",
    },
    {
      code: 28,
      name: "LiquidationTooSmall",
      msg: "Liquidation amount too small to receive collateral",
    },
    {
      code: 29,
      name: "ObligationHealthy",
      msg: "Cannot liquidate healthy obligations",
    },
    {
      code: 30,
      name: "ObligationStale",
      msg: "Obligation state needs to be refreshed",
    },
    {
      code: 31,
      name: "ObligationReserveLimit",
      msg: "Obligation reserve limit exceeded",
    },
    {
      code: 32,
      name: "InvalidObligationOwner",
      msg: "Obligation owner is invalid",
    },
    {
      code: 33,
      name: "ObligationDepositsEmpty",
      msg: "Obligation deposits are empty",
    },
    {
      code: 34,
      name: "ObligationBorrowsEmpty",
      msg: "Obligation borrows are empty",
    },
    {
      code: 35,
      name: "ObligationDepositsZero",
      msg: "Obligation deposits have zero value",
    },
    {
      code: 36,
      name: "ObligationBorrowsZero",
      msg: "Obligation borrows have zero value",
    },
    {
      code: 37,
      name: "InvalidObligationCollateral",
      msg: "Invalid obligation collateral",
    },
    {
      code: 38,
      name: "InvalidObligationLiquidity",
      msg: "Invalid obligation liquidity",
    },
    {
      code: 39,
      name: "ObligationCollateralEmpty",
      msg: "Obligation collateral is empty",
    },
    {
      code: 40,
      name: "ObligationLiquidityEmpty",
      msg: "Obligation liquidity is empty",
    },
    {
      code: 41,
      name: "NegativeInterestRate",
      msg: "Interest rate is negative",
    },
    {
      code: 42,
      name: "InvalidOracleConfig",
      msg: "Input oracle config is invalid",
    },
    {
      code: 43,
      name: "InvalidFlashLoanReceiverProgram",
      msg: "Input flash loan receiver program account is not valid",
    },
    {
      code: 44,
      name: "NotEnoughLiquidityAfterFlashLoan",
      msg: "Not enough liquidity after flash loan",
    },
  ],
};
