import * as anchor from "@project-serum/anchor";
import BN from "bn.js";
import { Buffer } from "buffer";
import { Keypair } from "@solana/web3.js";

describe("typescript", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.PdaDerivation;
  const base = Keypair.generate();
  const dataKey = Keypair.generate();
  const data = new BN(1);
  const seedA = 4;

  it("Inits the base account", async () => {
    await program.methods
      .initBase(data, dataKey.publicKey)
      .accounts({
        base: base.publicKey,
      })
      .signers([base])
      .rpc();
  });

  it("Inits the derived accounts", async () => {
    await program.methods
      .initMyAccount(seedA)
      .accounts({
        base: base.publicKey,
        base2: base.publicKey,
      })
      .rpc();
  });
  it("Fetches derived accounts from their seeds", async () => {

    const seeds = [
      new BN(seedA).toBuffer("le", 1),
      Buffer.from("another-seed"),
      Buffer.from("test"),
      base.publicKey.toBuffer(),
      base.publicKey.toBuffer(),
      Buffer.from("hi"),
      Buffer.from("hi"),
      new BN(1).toBuffer("le", 1),
      new BN(2).toBuffer("le", 4),
      new BN(3).toBuffer("le", 8),
      data.toBuffer("le", 8),
      dataKey.publicKey.toBuffer(),
    ]

    await program.account.myAccount.fetchPda(seeds);
    await program.account.myAccount.fetchMultiplePda([seeds, seeds]);

  });
});
