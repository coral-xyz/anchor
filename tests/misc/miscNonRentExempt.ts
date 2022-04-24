import * as anchor from "@project-serum/anchor";
import { Program, AnchorError } from "@project-serum/anchor";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
// @ts-expect-error
import { Misc } from "../../target/types/misc";
const { assert } = require("chai");

describe("miscNonRentExempt", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Misc as Program<Misc>;

  it("init_if_needed checks rent_exemption if init is not needed", async () => {
    const data = Keypair.generate();
    await program.methods
      .initDecreaseLamports()
      .accounts({
        data: data.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([data])
      .rpc();

    try {
      await program.methods.initIfNeededChecksRentExemption().accounts({
        data: data.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      }).signers([data]).rpc();
      assert.ok(false);
    } catch (_err) {
      assert.isTrue(_err instanceof AnchorError);
      const err: AnchorError = _err;
      assert.strictEqual(err.error.errorCode.number, 2005);
    }
  });

  it("allows non-rent exempt accounts", async () => {
    const data = Keypair.generate();
    await program.methods.initializeNoRentExempt().accounts({
      data: data.publicKey,
      rent: SYSVAR_RENT_PUBKEY,
    }).signers([data]).preInstructions([
      SystemProgram.createAccount({
        programId: program.programId,
        space: 8 + 16 + 16,
        lamports:
          await program.provider.connection.getMinimumBalanceForRentExemption(
            39
          ),
        fromPubkey: provider.wallet.publicKey,
        newAccountPubkey: data.publicKey,
      }),
    ]).rpc();
    await program.methods.testNoRentExempt().accounts({
      data: data.publicKey,
    }).rpc();
  });

  it("allows rent exemption to be skipped", async () => {
    const data = anchor.web3.Keypair.generate();
    await program.methods.initializeSkipRentExempt().accounts({
      data: data.publicKey,
      rent: SYSVAR_RENT_PUBKEY,
    }).signers([data]).preInstructions([
      SystemProgram.createAccount({
        programId: program.programId,
        space: 8 + 16 + 16,
        lamports:
          await program.provider.connection.getMinimumBalanceForRentExemption(
            39
          ),
        fromPubkey: provider.wallet.publicKey,
        newAccountPubkey: data.publicKey,
      }),
    ]).rpc();
  });

  it("can use rent_exempt to enforce rent exemption", async () => {
    const data = Keypair.generate();
    await program.methods.initializeSkipRentExempt().accounts({
      data: data.publicKey,
      rent: SYSVAR_RENT_PUBKEY,
    }).signers([data]).preInstructions([
      SystemProgram.createAccount({
        programId: program.programId,
        space: 8 + 16 + 16,
        lamports:
          await program.provider.connection.getMinimumBalanceForRentExemption(
            39
          ),
        fromPubkey: provider.wallet.publicKey,
        newAccountPubkey: data.publicKey,
      }),
    ]).rpc();

    try {
      await program.methods.testEnforceRentExempt().accounts({
        data: data.publicKey,
      }).rpc();
      assert.ok(false);
    } catch (_err) {
      assert.isTrue(_err instanceof AnchorError);
      const err: AnchorError = _err;
      assert.strictEqual(err.error.errorCode.number, 2005);
      assert.strictEqual(
        "A rent exemption constraint was violated",
        err.error.errorMessage
      );
    }
  });
});
