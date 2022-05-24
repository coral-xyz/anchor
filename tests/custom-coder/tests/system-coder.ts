import * as anchor from "@project-serum/anchor";
import { Spl } from "@project-serum/anchor";
import * as assert from "assert";
import BN from "bn.js";
import { Keypair, SystemProgram } from "@solana/web3.js";

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
    const lamports = await program.provider.connection.getMinimumBalanceForRentExemption(space);
    const owner = SystemProgram.programId;
    // act
    await program.methods
      .createAccount(new BN(lamports), new BN(space), owner)
      .accounts({
        from: provider.wallet.publicKey,
        to: aliceTokenKeypair.publicKey,
      }).signers([aliceTokenKeypair]).rpc();
    // assert
    const aliceAccount = await program.provider.connection.getAccountInfo(aliceTokenKeypair.publicKey);
    assert.notEqual(aliceAccount, null);
    assert.ok(owner.equals(aliceAccount.owner));
    assert.equal(lamports, aliceAccount.lamports);
  });
});
