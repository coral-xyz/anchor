import * as assert from "assert";
import { BorshCoder, Idl } from "../src";
import BN from "bn.js";

describe("coder.types", () => {
  test("Can encode and decode user-defined types", () => {
    const idl: Idl = {
      address: "Test111111111111111111111111111111111111111",
      metadata: {
        name: "basic_0",
        version: "0.0.0",
        spec: "0.1.0",
      },
      instructions: [
        {
          name: "initialize",
          accounts: [],
          args: [],
          discriminator: [],
        },
      ],
      types: [
        {
          name: "MintInfo",
          type: {
            kind: "struct",
            fields: [
              {
                name: "minted",
                type: "bool",
              },
              {
                name: "metadataUrl",
                type: "string",
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

  test("Can encode and decode 256-bit integers", () => {
    const idl: Idl = {
      address: "Test111111111111111111111111111111111111111",
      metadata: {
        name: "basic_0",
        version: "0.0.0",
        spec: "0.1.0",
      },
      instructions: [
        {
          name: "initialize",
          accounts: [],
          args: [],
          discriminator: [],
        },
      ],
      types: [
        {
          name: "IntegerTest",
          type: {
            kind: "struct",
            fields: [
              {
                name: "unsigned",
                type: "u256",
              },
              {
                name: "signed",
                type: "i256",
              },
            ],
          },
        },
      ],
    };

    const testing = {
      unsigned: new BN(2588012355),
      signed: new BN(-93842345),
    };

    const coder = new BorshCoder(idl);
    const encoded = coder.types.encode("IntegerTest", testing);
    assert.strictEqual(
      coder.types.decode("IntegerTest", encoded).toString(),
      testing.toString()
    );
  });

  test("Can encode and decode generics populated with user-defined types", () => {
    const idl: Idl = {
      address: "Test111111111111111111111111111111111111111",
      metadata: {
        name: "basic_0",
        version: "0.0.0",
        spec: "0.1.0",
      },
      instructions: [
        {
          name: "initialize",
          accounts: [],
          args: [],
          discriminator: [],
        },
      ],
      "types": [
        {
          "name": "GStruct",
          "generics": [
            {
              "kind": "type",
              "name": "T"
            }
          ],
          "type": {
            "kind": "struct",
            "fields": [
              {
                "name": "data",
                "type": {
                  "generic": "T"
                }
              }
            ]
          }
        },
        {
          "name": "MyAccount",
          "type": {
            "kind": "struct",
            "fields": [
              {
                "name": "data",
                "type": {
                  "defined": {
                    "name": "GStruct",
                    "generics": [
                      {
                        "kind": "type",
                        "type": {
                          "defined": {
                            "name": "MyStruct"
                          }
                        }
                      }
                    ]
                  }
                }
              }
            ]
          }
        },
        {
          "name": "MyStruct",
          "type": {
            "kind": "struct",
            "fields": [
              {
                "name": "data",
                "type": "u32"
              }
            ]
          }
        }
      ]
    };

    const coder = new BorshCoder(idl);

    const testing = {
      data: {
        data: 123,
      },
    };

    const encoded = coder.types.encode("MyAccount", testing);
    assert.strictEqual(
      coder.types.decode("MyAccount", encoded).toString(),
      testing.toString()
    );
  });
});
