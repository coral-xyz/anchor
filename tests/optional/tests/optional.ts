import * as anchor from "@project-serum/anchor";
import {
  AnchorError,
  LangErrorCode,
  LangErrorMessage,
  Program,
  ProgramError,
  web3,
} from "@project-serum/anchor";
import { Optional } from "../target/types/optional";
import { assert, expect } from "chai";

describe("Optional", () => {
  // configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());
  let optional2Keypair = web3.Keypair.generate();
  let requiredKeypair1 = web3.Keypair.generate();
  let requiredKeypair2 = web3.Keypair.generate();
  const program = anchor.workspace.Optional as Program<Optional>;

  // payer of the transactions
  const payer = (program.provider as anchor.AnchorProvider).wallet;
  let initValue = new anchor.BN(10);

  // optional pda
  let seeds = [Buffer.from("data1"), optional2Keypair.publicKey.toBuffer()];
  let optional1Pubkey = web3.PublicKey.findProgramAddressSync(
    seeds,
    program.programId
  )[0];
  let createOptional2;
  let createRequired2;

  it("Initialize with optionals null works", async () => {
    let createRequired = await program.account.data2.createInstruction(
      requiredKeypair1
    );
    await program.methods
      .initialize(new anchor.BN(10), optional2Keypair.publicKey)
      .preInstructions([createRequired])
      .accounts({
        payer: payer.publicKey,
        systemProgram: web3.SystemProgram.programId,
        required: requiredKeypair1.publicKey,
        optional1: null,
        optional2: null,
      })
      .signers([requiredKeypair1])
      .rpc({ skipPreflight: true });

    let required1 = await program.account.data2.fetchNullable(
      requiredKeypair1.publicKey
    );
    expect(required1.optional1.toString()).to.equal(
      web3.PublicKey.default.toString()
    );
  });

  it("Initialize missing optional2 fails", async () => {
    try {
      let x = await program.methods
        .initialize(initValue, optional2Keypair.publicKey)
        .preInstructions([await createRequired2])
        .accounts({
          payer: payer.publicKey,
          systemProgram: web3.SystemProgram.programId,
          required: requiredKeypair2.publicKey,
          optional1: optional1Pubkey,
          optional2: null,
        })
        .signers([requiredKeypair2])
        .transaction();
      console.log("hi");
      assert.ok(false);
    } catch (e) {
      const errMsg1 = "ProgramFailedToComplete";
      const errMsg2 = "Program failed to complete";
      let error: string = e.toString();
      console.log("Error:", error);
      assert(
        error.includes(errMsg1) || error.includes(errMsg2),
        "Program didn't fail to complete!!"
      );
    }
  });

  it("Initialize missing payer fails", async () => {
    createOptional2 = await program.account.data2.createInstruction(
      optional2Keypair
    );
    createRequired2 = await program.account.data2.createInstruction(
      requiredKeypair2
    );

    try {
      await program.methods
        .initialize(initValue, optional2Keypair.publicKey)
        .preInstructions([createOptional2, createRequired2])
        .accounts({
          payer: null,
          systemProgram: web3.SystemProgram.programId,
          required: requiredKeypair2.publicKey,
          optional1: optional1Pubkey,
          optional2: optional2Keypair.publicKey,
        })
        .signers([requiredKeypair2, optional2Keypair])
        .rpc({ skipPreflight: true });
      assert.ok(false);
    } catch (e) {
      assert.isTrue(e instanceof ProgramError);
      const err: ProgramError = e;
      assert.strictEqual(
        err.msg,
        LangErrorMessage.get(LangErrorCode.ConstraintAccountIsNone)
      );
      assert.strictEqual(err.code, LangErrorCode.ConstraintAccountIsNone);
    }
  });

  it("Initialize with bad PDA fails", async () => {
    try {
      // bad optional pda
      let seeds = [
        Buffer.from("fakedata1"),
        optional2Keypair.publicKey.toBuffer(),
      ];
      let badOptional1Pubkey = web3.PublicKey.findProgramAddressSync(
        seeds,
        program.programId
      )[0];
      await program.methods
        .initialize(initValue, optional2Keypair.publicKey)
        .preInstructions([createOptional2, createRequired2])
        .accounts({
          systemProgram: web3.SystemProgram.programId,
          required: requiredKeypair2.publicKey,
          optional1: badOptional1Pubkey,
          optional2: optional2Keypair.publicKey,
        })
        .signers([requiredKeypair2, optional2Keypair])
        .rpc({ skipPreflight: true });
      assert.ok(false);
    } catch (e) {
      assert.isTrue(e instanceof ProgramError);
      const err: ProgramError = e;
      assert.strictEqual(
        err.msg,
        LangErrorMessage.get(LangErrorCode.ConstraintSeeds)
      );
      assert.strictEqual(err.code, LangErrorCode.ConstraintSeeds);
    }
  });

  it("Initialize with all valid accounts works", async () => {
    await program.methods
      .initialize(initValue, optional2Keypair.publicKey)
      .preInstructions([createOptional2, createRequired2])
      .accounts({
        payer: payer.publicKey,
        systemProgram: web3.SystemProgram.programId,
        required: requiredKeypair2.publicKey,
        optional1: optional1Pubkey,
        optional2: optional2Keypair.publicKey,
      })
      .signers([requiredKeypair2, optional2Keypair])
      .rpc({ skipPreflight: true });

    let required2 = await program.account.data2.fetchNullable(
      requiredKeypair2.publicKey
    );
    let optional1 = await program.account.data1.fetchNullable(optional1Pubkey);
    let optional2 = await program.account.data2.fetchNullable(
      optional2Keypair.publicKey
    );

    expect(optional1.data.toNumber()).to.equal(initValue.toNumber());
    expect(optional2.optional1.toString()).to.equal(optional1Pubkey.toString());
    expect(required2.optional1.toString()).to.equal(
      web3.PublicKey.default.toString()
    );
  });

  it("realloc_with_constraints", async () => {
    try {
      await program.methods
        .realloc()
        .accounts({
          payer: payer.publicKey,
          optional1: optional1Pubkey,
          required: optional2Keypair.publicKey,
          systemProgram: null,
        })
        .rpc({ skipPreflight: true });

      assert.ok(false);
    } catch (e) {
      assert.isTrue(e instanceof AnchorError);
      const err: AnchorError = e;
      const errorCode = LangErrorCode.ConstraintHasOne;
      assert.strictEqual(
        err.error.errorMessage,
        LangErrorMessage.get(errorCode)
      );
      assert.strictEqual(err.error.errorCode.number, errorCode);
    }
    //
    //     try {
    //         await program.methods
    //             .realloc()
    //             .accounts({
    //                 payer: payer.publicKey,
    //                 optional1: optional1Pubkey,
    //                 required: optional2Keypair.publicKey,
    //                 systemProgram: null,
    //             })
    //             .rpc({skipPreflight: true});
    //
    //         assert.ok(false);
    //     } catch (e) {
    //         assert.isTrue(e instanceof AnchorError);
    //         const err: AnchorError = e;
    //         const errorCode = LangErrorCode.ConstraintHasOne;
    //         assert.strictEqual(err.error.errorMessage, LangErrorMessage.get(errorCode));
    //         assert.strictEqual(err.error.errorCode.number, errorCode);
    //     }
    //
    //     try {
    //         await program.methods
    //             .realloc()
    //             .accounts({
    //                 payer: payer.publicKey,
    //                 optional1: optional1Pubkey,
    //                 required: optional2Keypair.publicKey,
    //                 systemProgram: null,
    //             })
    //             .rpc({skipPreflight: true});
    //
    //         assert.ok(false);
    //     } catch (e) {
    //         assert.isTrue(e instanceof AnchorError);
    //         const err: AnchorError = e;
    //         const errorCode = LangErrorCode.ConstraintHasOne;
    //         assert.strictEqual(err.error.errorMessage, LangErrorMessage.get(errorCode));
    //         assert.strictEqual(err.error.errorCode.number, errorCode);
    //     }
  });
});
