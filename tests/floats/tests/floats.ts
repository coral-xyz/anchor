import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Floats } from "../target/types/floats";
import assert from "assert";

describe("floats", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.Floats as Program<Floats>;

  it("Creates an account with float data", async () => {
    const accountKeypair = anchor.web3.Keypair.generate();

    await program.rpc.create(1.0, 2.0, {
      accounts: {
        account: accountKeypair.publicKey,
        authority: anchor.getProvider().wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [accountKeypair],
    });

    const account = await program.account.floatDataAccount.fetch(
      accountKeypair.publicKey
    );

    assert.strictEqual(account.dataF32, 1.0);
    assert.strictEqual(account.dataF64, 2.0);
  });

  it("Updates an account with float data", async () => {
    const accountKeypair = anchor.web3.Keypair.generate();
    const authorityPublicKey = anchor.getProvider().wallet.publicKey;

    await program.rpc.create(1.0, 2.0, {
      accounts: {
        account: accountKeypair.publicKey,
        authority: authorityPublicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [accountKeypair],
    });

    let account = await program.account.floatDataAccount.fetch(
      accountKeypair.publicKey
    );

    await program.rpc.update(3.0, 4.0, {
      accounts: {
        account: accountKeypair.publicKey,
        authority: authorityPublicKey,
      },
    });

    account = await program.account.floatDataAccount.fetch(
      accountKeypair.publicKey
    );

    assert.strictEqual(account.dataF32, 3.0);
    assert.strictEqual(account.dataF64, 4.0);
  });
});
