import * as assert from "assert";
import { BorshCoder, Idl, BN } from "../src";
// import {}

import SplGov from '../idl.json';

describe("coder.types", () => {
  test("Can encode and decode user-defined types", () => {
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
      types: [
        {
          name: "MintInfo",
          type: {
            kind: "struct" as const,
            fields: [
              {
                name: "minted",
                type: "bool" as const,
              },
              {
                name: "metadataUrl",
                type: "string" as const,
              },
            ],
          },
        },
      ],
    };
    const coder = new BorshCoder(idl);

    const mintInfo = {
      minted: true,
      metadataUrl: "hello",
    };
    const encoded = coder.types.encode("MintInfo", mintInfo);

    assert.deepEqual(coder.types.decode("MintInfo", encoded), mintInfo);
  });
  it("Test tuple enum variant decoding", () => {
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
      types: [
        {
          name: "Vote",
          type: {
            kind: "enum" as const,
            variants: [
              {
                name: "VoteWithComment",
                fields: [
                  "bool" as const,
                  "string" as const,
                ]
              }
            ]
          }
        },
      ],
    };
    const coder = new BorshCoder(idl);

    let vote = {
      voteWithComment: {
        arg0: true,
        arg1: "blessed"
      }
    };
    let encoded = coder.types.encode("Vote", vote);

    assert.deepEqual(coder.types.decode("Vote", encoded), vote);
  })
});

