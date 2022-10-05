import * as anchor from "@project-serum/anchor";
import {
  Program,
  web3,
  BN,
  AnchorError,
  LangErrorCode,
  LangErrorMessage,
} from "@project-serum/anchor";
import { Optional } from "../target/types/optional";
import { assert, expect } from "chai";

describe("Optional", () => {
  // configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.Optional as Program<Optional>;

  const DATA_PDA_PREFIX = "data_pda";

  const findDataPda = (
    dataAccount: web3.PublicKey
  ): [web3.PublicKey, number] => {
    return web3.PublicKey.findProgramAddressSync(
      [Buffer.from(DATA_PDA_PREFIX), dataAccount.toBuffer()],
      program.programId
    );
  };

  // payer of the transactions
  const payerWallet = (program.provider as anchor.AnchorProvider).wallet;
  const payer = payerWallet.publicKey;
  const systemProgram = web3.SystemProgram.programId;

  let requiredKeypair1 = web3.Keypair.generate();
  let requiredKeypair2 = web3.Keypair.generate();

  let createRequiredIx1: web3.TransactionInstruction;
  let createRequiredIx2: web3.TransactionInstruction;

  let dataAccountKeypair1 = web3.Keypair.generate();
  let dataAccountKeypair2 = web3.Keypair.generate();

  let dataPda1 = findDataPda(dataAccountKeypair1.publicKey);
  let dataPda2 = findDataPda(dataAccountKeypair2.publicKey);

  const initializeValue1 = new BN(10);
  const initializeValue2 = new BN(100);
  const initializeKey = web3.PublicKey.default;

  const createRequired = async (
    requiredKeypair?: web3.Keypair
  ): Promise<[web3.Keypair, web3.TransactionInstruction]> => {
    const keypair = requiredKeypair ?? new web3.Keypair();
    const createIx = await program.account.dataAccount.createInstruction(
      keypair
    );
    return [keypair, createIx];
  };

  before("Setup async stuff", async () => {
    createRequiredIx1 = (await createRequired(requiredKeypair1))[1];
    createRequiredIx2 = (await createRequired(requiredKeypair2))[1];
  });

  describe("Initialize tests", async () => {
    it("Initialize with required null fails anchor-ts validation", async () => {
      const [requiredKeypair, createRequiredIx] = await createRequired();
      try {
        await program.methods
          .initialize(initializeValue1, initializeKey)
          .preInstructions([createRequiredIx])
          .accounts({
            payer,
            systemProgram,
            // @ts-ignore
            required: null, //requiredKeypair.publicKey,
            optionalPda: null,
            optionalAccount: null,
          })
          .signers([requiredKeypair])
          .rpc({ skipPreflight: true });
        assert.ok(false);
      } catch (e) {
        const errMsg = "Invalid arguments: required not provided";
        // @ts-ignore
        let error: string = e.toString();
        assert(error.includes(errMsg), `Unexpected error: ${e}`);
      }
    });

    it("Can initialize with no payer and no optionals", async () => {
      const [requiredKeypair, createRequiredIx] = await createRequired();
      await program.methods
        .initialize(initializeValue1, initializeKey)
        .preInstructions([createRequiredIx])
        .accounts({
          payer: null,
          systemProgram,
          required: requiredKeypair.publicKey,
          optionalPda: null,
          optionalAccount: null,
        })
        .signers([requiredKeypair])
        .rpc({ skipPreflight: true });

      let required = await program.account.dataAccount.fetch(
        requiredKeypair.publicKey
      );
      expect(required.data.toNumber()).to.equal(0);
    });

    it("Can initialize with no optionals", async () => {
      const [requiredKeypair, createRequiredIx] = await createRequired();
      await program.methods
        .initialize(initializeValue1, initializeKey)
        .preInstructions([createRequiredIx])
        .accounts({
          payer: null,
          systemProgram: null,
          required: requiredKeypair.publicKey,
          optionalPda: null,
          optionalAccount: null,
        })
        .signers([requiredKeypair])
        .rpc({ skipPreflight: true });

      let required = await program.account.dataAccount.fetch(
        requiredKeypair.publicKey
      );
      expect(required.data.toNumber()).to.equal(0);
    });

    it("Initialize with optionals and missing system program fails optional checks", async () => {
      const [requiredKeypair, createRequiredIx] = await createRequired();
      const dataAccount = new web3.Keypair();
      try {
        await program.methods
          .initialize(initializeValue1, initializeKey)
          .preInstructions([createRequiredIx])
          .accounts({
            payer,
            systemProgram: null,
            required: requiredKeypair.publicKey,
            optionalPda: null,
            optionalAccount: dataAccount.publicKey,
          })
          .signers([requiredKeypair, dataAccount])
          .rpc({ skipPreflight: true });
        assert.ok(false);
      } catch (e) {
        assert.isTrue(e instanceof AnchorError);
        const err: AnchorError = <AnchorError>e;
        const errorCode = LangErrorCode.ConstraintAccountIsNone;
        assert.strictEqual(
          err.error.errorMessage,
          LangErrorMessage.get(errorCode)
        );
        assert.strictEqual(err.error.errorCode.number, errorCode);
      }
    });

    it("Panics with reference to None account in constraint", async () => {
      const [requiredKeypair, createRequiredIx] = await createRequired();
      const dataAccount = new web3.Keypair();
      const [dataPda] = findDataPda(dataAccount.publicKey);
      try {
        await program.methods
          .initialize(initializeValue1, initializeKey)
          .preInstructions([createRequiredIx])
          .accounts({
            payer,
            systemProgram,
            required: requiredKeypair.publicKey,
            optionalPda: dataPda,
            optionalAccount: null,
          })
          .signers([requiredKeypair])
          .rpc({ skipPreflight: true });
        assert.ok(false);
      } catch (e) {
        const errMsg = "ProgramFailedToComplete";
        // @ts-ignore
        let error: string = e.toString();
        assert(error.includes(errMsg), `Unexpected error: ${e}`);
      }
    });

    it("Can initialize with required and optional account", async () => {
      await program.methods
        .initialize(initializeValue1, initializeKey)
        .preInstructions([createRequiredIx1])
        .accounts({
          payer,
          systemProgram,
          required: requiredKeypair1.publicKey,
          optionalPda: null,
          optionalAccount: dataAccountKeypair1.publicKey,
        })
        .signers([requiredKeypair1, dataAccountKeypair1])
        .rpc({ skipPreflight: true });

      const requiredDataAccount = await program.account.dataAccount.fetch(
        requiredKeypair1.publicKey
      );
      expect(requiredDataAccount.data.toNumber()).to.equal(0);

      const optionalDataAccount = await program.account.dataAccount.fetch(
        dataAccountKeypair1.publicKey
      );
      expect(optionalDataAccount.data.toNumber()).to.equal(
        initializeValue1.muln(2).toNumber()
      );
    });

    it("Initialize with everything with invalid seeds fails", async () => {
      try {
        await program.methods
          .initialize(initializeValue2, initializeKey)
          .preInstructions([createRequiredIx2])
          .accounts({
            payer,
            systemProgram,
            required: requiredKeypair2.publicKey,
            optionalPda: dataPda1[0],
            optionalAccount: dataAccountKeypair2.publicKey,
          })
          .signers([requiredKeypair2, dataAccountKeypair2])
          .rpc({ skipPreflight: true });
        assert.ok(false);
      } catch (e) {
        assert.isTrue(e instanceof AnchorError);
        const err: AnchorError = <AnchorError>e;
        const errorCode = LangErrorCode.ConstraintSeeds;
        assert.strictEqual(
          err.error.errorMessage,
          LangErrorMessage.get(errorCode)
        );
        assert.strictEqual(err.error.errorCode.number, errorCode);
      }
    });

    it("Initialize with everything succeeds", async () => {
      await program.methods
        .initialize(initializeValue2, initializeKey)
        .preInstructions([createRequiredIx2])
        .accounts({
          payer,
          systemProgram,
          required: requiredKeypair2.publicKey,
          optionalPda: dataPda2[0],
          optionalAccount: dataAccountKeypair2.publicKey,
        })
        .signers([requiredKeypair2, dataAccountKeypair2])
        .rpc({ skipPreflight: true });

      const requiredDataAccount = await program.account.dataAccount.fetch(
        requiredKeypair2.publicKey
      );
      expect(requiredDataAccount.data.toNumber()).to.equal(0);

      const optionalDataAccount = await program.account.dataAccount.fetch(
        dataAccountKeypair2.publicKey
      );
      expect(optionalDataAccount.data.toNumber()).to.equal(
        initializeValue2.toNumber()
      );

      const optionalDataPda = await program.account.dataPda.fetch(dataPda2[0]);
      expect(optionalDataPda.dataAccount.toString()).to.equal(
        initializeKey.toString()
      );
    });
  });

  // describe("Update tests", async () => {
  //   it("Initialize with required null fails anchor-ts validation", async () => {
  //     const [requiredKeypair, createRequiredIx] = await createRequired();
  //     try {
  //       await program.methods
  //         .initialize(initializeValue1, initializeKey)
  //         .preInstructions([createRequiredIx])
  //         .accounts({
  //           payer,
  //           systemProgram,
  //           // @ts-ignore
  //           required: null, //requiredKeypair.publicKey,
  //           optionalPda: null,
  //           optionalAccount: null,
  //         })
  //         .signers([requiredKeypair])
  //         .rpc({ skipPreflight: true });
  //       assert.ok(false);
  //     } catch (e) {
  //       const errMsg = "Invalid arguments: required not provided";
  //       // @ts-ignore
  //       let error: string = e.toString();
  //       assert(error.includes(errMsg), `Unexpected error: ${e}`);
  //     }
  //   });
  //
  //   it("Can initialize with no payer and no optionals", async () => {
  //     const [requiredKeypair, createRequiredIx] = await createRequired();
  //     await program.methods
  //       .initialize(initializeValue1, initializeKey)
  //       .preInstructions([createRequiredIx])
  //       .accounts({
  //         payer: null,
  //         systemProgram,
  //         required: requiredKeypair.publicKey,
  //         optionalPda: null,
  //         optionalAccount: null,
  //       })
  //       .signers([requiredKeypair])
  //       .rpc({ skipPreflight: true });
  //
  //     let required = await program.account.dataAccount.fetch(
  //       requiredKeypair.publicKey
  //     );
  //     expect(required.data.toNumber()).to.equal(0);
  //   });
  // });
});
