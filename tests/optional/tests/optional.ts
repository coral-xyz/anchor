import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import { expect } from 'chai';
import { Optional } from "../target/types/optional";

describe("Optional", () => {
  // configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());
  // candy guard for the tests
  const keypair = anchor.web3.Keypair.generate();
  // candy guard program
  const program = anchor.workspace.Optional as Program<Optional>;
  // payer of the transactions
  const payer = (program.provider as anchor.AnchorProvider).wallet;

  it("initialize", async () => {
    await program.methods
        .initialize()
        .accounts({
          candyGuard: keypair.publicKey,
          authority: payer.publicKey,
          payer: payer.publicKey,
        })
        .signers([keypair])
        .rpc();
    let data = await program.account.candyGuard.fetch(keypair.publicKey);

    expect(candy_guard.features.toNumber()).to.equal(0);
  });

  it("update", async () => {
    let candy_guard = await program.account.candyGuard.fetch(keypair.publicKey);
    expect(candy_guard.features.toNumber()).to.equal(0);
    // console.log(program.)


    await program.methods.update(settings).accounts({
      // candyGuard: null,
      authority: null,
      candyGuard: keypair.publicKey,
      // authority: payer.publicKey,
    }).rpc();

    candy_guard = await program.account.candyGuard.fetch(keypair.publicKey);
    // bot_tax (1) + live_date (2) + lamports_charge (8)
    console.log(candy_guard.features.toNumber());
    expect(candy_guard.features.toNumber()).to.equal(11);
  });
});
