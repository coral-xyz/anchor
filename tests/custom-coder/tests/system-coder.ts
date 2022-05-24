import * as anchor from "@project-serum/anchor";
import { Spl } from "@project-serum/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  NONCE_ACCOUNT_LENGTH,
  PublicKey,
  SystemProgram,
  SYSVAR_RECENT_BLOCKHASHES_PUBKEY
} from "@solana/web3.js";
import * as assert from "assert";
import BN from "bn.js";

describe("system-coder", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Client.
  const program = Spl.system();

  // Constants.
  const aliceTokenKeypair = Keypair.generate();

  it("Creates an account", async () => {
    // arrange
    const space = 100;
    const lamports =
      await program.provider.connection.getMinimumBalanceForRentExemption(
        space
      );
    const owner = SystemProgram.programId;
    // act
    await program.methods
      .createAccount(new BN(lamports), new BN(space), owner)
      .accounts({
        from: provider.wallet.publicKey,
        to: aliceTokenKeypair.publicKey,
      })
      .signers([aliceTokenKeypair])
      .rpc();
    // assert
    const aliceAccount = await program.provider.connection.getAccountInfo(
      aliceTokenKeypair.publicKey
    );
    assert.notEqual(aliceAccount, null);
    assert.ok(owner.equals(aliceAccount.owner));
    assert.equal(lamports, aliceAccount.lamports);
  });

  it("Assigns an account to a program", async () => {
    // arrange
    const owner = TOKEN_PROGRAM_ID;
    // act
    await program.methods
      .assign(owner)
      .accounts({
        pubkey: aliceTokenKeypair.publicKey,
      })
      .signers([aliceTokenKeypair])
      .rpc();
    // assert
    const aliceAccount = await program.provider.connection.getAccountInfo(
      aliceTokenKeypair.publicKey
    );
    assert.notEqual(aliceAccount, null);
    assert.ok(owner.equals(aliceAccount.owner));
  });

  it("Creates an account with seed", async () => {
    const space = 100;
    const lamports =
      await program.provider.connection.getMinimumBalanceForRentExemption(
        space
      );
    const owner = SystemProgram.programId;
    const seed = "seeds";
    const bobPublicKey = await PublicKey.createWithSeed(
      aliceTokenKeypair.publicKey,
      seed,
      owner
    );
    // act
    await program.methods
      .createAccountWithSeed(
        aliceTokenKeypair.publicKey,
        seed,
        new BN(lamports),
        new BN(space),
        owner
      )
      .accounts({
        base: aliceTokenKeypair.publicKey,
        from: provider.wallet.publicKey,
        to: bobPublicKey,
      })
      .signers([aliceTokenKeypair])
      .rpc();
    // assert
    const bobAccount = await program.provider.connection.getAccountInfo(
      bobPublicKey
    );
    assert.notEqual(bobAccount, null);
  });

  it("Initializes nonce account", async () => {
    // arrange
    const nonceKeypair = Keypair.generate();
    const owner = SystemProgram.programId;
    const space = NONCE_ACCOUNT_LENGTH;
    const lamports =
      await provider.connection.getMinimumBalanceForRentExemption(space);
    // act
    await program.methods
      .initializeNonceAccount(provider.wallet.publicKey)
      .accounts({
        nonce: nonceKeypair.publicKey,
        recentBlockhashes: SYSVAR_RECENT_BLOCKHASHES_PUBKEY,
      })
      .preInstructions([
        await program.methods
          .createAccount(new BN(lamports), new BN(space), owner)
          .accounts({
            from: provider.wallet.publicKey,
            to: nonceKeypair.publicKey,
          })
          .instruction(),
      ])
      .signers([nonceKeypair])
      .rpc();
    // assert
    const nonceAccount = await program.provider.connection.getAccountInfo(
      nonceKeypair.publicKey
    );
    assert.notEqual(nonceAccount, null);
  });

  it("Transfers lamports", async () => {
    // arrange
    const receiverKeypair = Keypair.generate();
    const lamports = 0.1* LAMPORTS_PER_SOL;
    // act
    await program.methods
      .transfer(new BN(lamports))
      .accounts({
        from: provider.wallet.publicKey,
        to: receiverKeypair.publicKey,
      })
      .rpc();
    // assert
    const receiverAccount = await program.provider.connection.getAccountInfo(
      receiverKeypair.publicKey
    );
    assert.notEqual(receiverAccount, null);
    assert.equal(lamports, receiverAccount.lamports);
  });
});
