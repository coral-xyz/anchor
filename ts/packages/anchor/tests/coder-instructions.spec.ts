import * as assert from "assert";
import { createNodeArray } from "typescript";
import { BorshCoder } from "../src";
import { sha256 } from "js-sha256";
import { ACCOUNT_DISCRIMINATOR_SIZE } from "../src/coder/borsh/accounts";
import { IdlType } from "../src/idl";

describe("coder.accounts", () => {
  test("Can encode and decode user-defined accounts, including those with consecutive capital letters", () => {
    const idl = {
      version: "0.0.0",
      name: "basic_0",
      instructions: [
        {
          name: "initialize",
          accounts: [],
          args: [],
        },
        {
          name: "initMapping",
          discriminant: {
            value: [2, 0, 0, 0, 0, 0, 0, 1],
            type: { array: ["u8", 8] as [IdlType, number] },
          },
          accounts: [
            {
              name: "fundingAccount",
              isMut: true,
              isSigner: true,
            },
            {
              name: "freshMappingAccount",
              isMut: true,
              isSigner: true,
            },
          ],
          args: [],
        },
      ],

      accounts: [
        {
          name: "MemberDAO",
          type: {
            kind: "struct" as const,
            fields: [
              {
                name: "name",
                type: "string" as const,
              },
            ],
          },
        },
      ],
    };
    const coder = new BorshCoder(idl);

    const memberDAO = {
      name: "test",
    };

    let buffer = coder.instruction.encode("initialize", null);
    // start of encoded account = account discriminator
    assert.deepEqual(
      buffer,
      Buffer.from(sha256.digest("global:initialize")).subarray(
        0,
        ACCOUNT_DISCRIMINATOR_SIZE
      )
    );
    let buffer2 = coder.instruction.encode("initMapping", null);
    // start of encoded account = account discriminator
    assert.deepEqual(
      buffer2,
      Buffer.from([2, 0, 0, 0, 0, 0, 0, 1]).subarray(
        0,
        ACCOUNT_DISCRIMINATOR_SIZE
      )
    );
  });
});
