import * as anchor from "@project-serum/anchor";
import {
  Program,
  BN,
  IdlAccounts,
  AnchorError,
  Wallet,
} from "@project-serum/anchor";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  Token,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { IdlDoc } from "../../target/types/idl_doc";
const utf8 = anchor.utils.bytes.utf8;
const { assert } = require("chai");
const idl_doc_idl = require("../../target/idl/idl_doc.json");

describe("idl_doc", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  const wallet = provider.wallet as Wallet;
  anchor.setProvider(provider);
  const program = anchor.workspace.IdlDoc as Program<IdlDoc>;

  describe("IDL doc strings", () => {
    const instruction = program.idl.instructions.find(
        (i) => i.name === "testIdlDocParse"
    );
    it("includes instruction doc comment", async () => {
        assert.equal(
            instruction.doc,
                "This instruction doc should appear in the IDL"
        );
    });

    it("includes account doc comment", async () => {
        const act = instruction.accounts.find(
            (i) => i.name === 'act'
        );
        assert.equal(
            act.doc,
                "This account doc comment should appear in the IDL"
        );
    });

    const dataWithDoc = program.idl.accounts.find(
        (i) => i.name === 'DataWithDoc'
    );

    it("includes accounts doc comment", async () => {
        assert.equal(
            dataWithDoc.doc,
                "Custom account doc comment should appear in the IDL"
        );
    });

    it("includes account attribute doc comment", async () => {
        const dataField = dataWithDoc.type.fields.find(
            (i) => i.name === 'data'
        );
        assert.equal(
            dataField.doc, 
                "Account attribute doc comment should appear in the IDL"
        );
    });
  });
});
