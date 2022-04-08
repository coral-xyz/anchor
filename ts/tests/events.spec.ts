import { PublicKey } from "@solana/web3.js";
import { EventParser } from "../src/program/event";
import { BorshCoder } from "../src";

describe("Events", () => {
  it("Parses multiple instructions", async () => {
    const logs = [
      "Program 11111111111111111111111111111111 invoke [1]",
      "Program 11111111111111111111111111111111 success",
      "Program J2XMGdW2qQLx7rAdwWtSZpTXDgAQ988BLP9QTgUZvm54 invoke [1]",
      "Program J2XMGdW2qQLx7rAdwWtSZpTXDgAQ988BLP9QTgUZvm54 consumed 17867 of 200000 compute units",
      "Program J2XMGdW2qQLx7rAdwWtSZpTXDgAQ988BLP9QTgUZvm54 success",
    ];
    const idl = {
      version: "0.0.0",
      name: "basic_0",
      instructions: [
        {
          name: "initialize",
          accounts: [],
          args: [],
        },
      ],
    };
    const coder = new BorshCoder(idl);
    const programId = PublicKey.default;
    const eventParser = new EventParser(programId, coder);

    eventParser.parseLogs(logs, () => {
      throw new Error("Should never find logs");
    });
  });
  it("Upgrade event check", () => {
    const logs = [
      "Upgraded program J2XMGdW2qQLx7rAdwWtSZpTXDgAQ988BLP9QTgUZvm54",
      "Program 11111111111111111111111111111111 invoke [1]",
      "Program 11111111111111111111111111111111 success",
      "Program J2XMGdW2qQLx7rAdwWtSZpTXDgAQ988BLP9QTgUZvm54 invoke [1]",
      "Program J2XMGdW2qQLx7rAdwWtSZpTXDgAQ988BLP9QTgUZvm54 consumed 17867 of 200000 compute units",
      "Program J2XMGdW2qQLx7rAdwWtSZpTXDgAQ988BLP9QTgUZvm54 success",
    ];
    const idl = {
      version: "0.0.0",
      name: "basic_0",
      instructions: [
        {
          name: "initialize",
          accounts: [],
          args: [],
        },
      ],
    };
    const coder = new BorshCoder(idl);
    const programId = PublicKey.default;
    const eventParser = new EventParser(programId, coder);

    eventParser.parseLogs(logs, () => {
      throw new Error("Should never find logs");
    });
  });
  it("Find event from logs", () => {
    const logs = [
      `Upgraded program J2XMGdW2qQLx7rAdwWtSZpTXDgAQ988BLP9QTgUZvm54`,
      'Program J2XMGdW2qQLx7rAdwWtSZpTXDgAQ988BLP9QTgUZvm54 invoke [1]',
      'Program log: Instruction: BuyNft',
      'Program 11111111111111111111111111111111 invoke [2]',
      'Program log: UhUxVlc2hGeTBjNPCGmmZjvNSuBOYpfpRPJLfJmTLZueJAmbgEtIMGl9lLKKH6YKy1AQd8lrsdJPPc7joZ6kCkEKlNLKhbUv',
      'Program 11111111111111111111111111111111 success',
      'Program 11111111111111111111111111111111 invoke [2]',
      'Program 11111111111111111111111111111111 success',
      'Program 11111111111111111111111111111111 invoke [2]',
      'Program 11111111111111111111111111111111 success',
      'Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]',
      'Program log: Instruction: Transfer',
      'Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2549 of 141128 compute units',
      'Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success',
      'Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]',
      'Program log: Instruction: CloseAccount',
      'Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1745 of 135127 compute units',
      'Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success',
      'Program log: UhUxVlc2hGeTBjNPCGmmZjvNSuBOYpfpRPJLfJmTLZueJAmbgEtIMGl9lLKKH6YKy1AQd8lrsdJPPc7joZ6kCkEKlNLKhbUv',
      'Program J2XMGdW2qQLx7rAdwWtSZpTXDgAQ988BLP9QTgUZvm54 consumed 73106 of 200000 compute units',
      'Program J2XMGdW2qQLx7rAdwWtSZpTXDgAQ988BLP9QTgUZvm54 success',
    ];

    const idl = {
      version: "0.0.0",
      name: "basic_1",
      instructions: [
        {
          name: "initialize",
          accounts: [],
          args: [],
        },
      ],
      events: [
        {
          name: "NftSold",
          fields: [
            {
              name: "nftMintAddress",
              type: "publicKey" as "publicKey",
              index: false
            },
            {
              name: "accountAddress",
              type: "publicKey" as "publicKey",
              index: false
            }
          ]
        }
      ]
    };

    const coder = new BorshCoder(idl);
    const programId = new PublicKey('J2XMGdW2qQLx7rAdwWtSZpTXDgAQ988BLP9QTgUZvm54');
    const eventParser = new EventParser(programId, coder);

    eventParser.parseLogs(logs, (event) => {
      expect(event.name).toEqual('NftSold')
    });
  })
});
