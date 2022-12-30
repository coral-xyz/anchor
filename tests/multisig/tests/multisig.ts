import * as anchor from "@coral-xyz/anchor";
import { AnchorError } from "@coral-xyz/anchor";
import { assert } from "chai";
import { Multisig } from "../target/types/multisig";

describe("multisig", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Multisig as anchor.Program<Multisig>;

  it("Tests the multisig program", async () => {
    const multisig = anchor.web3.Keypair.generate();
    const [multisigSigner, nonce] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [multisig.publicKey.toBuffer()],
        program.programId
      );

    const ownerA = anchor.web3.Keypair.generate();
    const ownerB = anchor.web3.Keypair.generate();
    const ownerC = anchor.web3.Keypair.generate();
    const ownerD = anchor.web3.Keypair.generate();
    const owners = [ownerA.publicKey, ownerB.publicKey, ownerC.publicKey];

    const threshold = new anchor.BN(2);
    await program.methods
      .createMultisig(owners, threshold, nonce)
      .accounts({
        payer: program.provider.publicKey,
        multisig: multisig.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([multisig])
      .rpc();

    let multisigAccount = await program.account.multisig.fetch(
      multisig.publicKey
    );
    assert.strictEqual(multisigAccount.nonce, nonce);
    assert.isTrue(multisigAccount.threshold.eq(new anchor.BN(2)));
    assert.deepStrictEqual(multisigAccount.owners, owners);

    const pid = program.programId;
    const accounts = [
      {
        pubkey: multisig.publicKey,
        isWritable: true,
        isSigner: false,
      },
      {
        pubkey: multisigSigner,
        isWritable: false,
        isSigner: true,
      },
    ];
    const newOwners = [ownerA.publicKey, ownerB.publicKey, ownerD.publicKey];
    const data = program.coder.instruction.encode("set_owners", {
      owners: newOwners,
    });

    const transaction = anchor.web3.Keypair.generate();

    await program.methods
      .createTransaction(pid, accounts, data)
      .accounts({
        payer: program.provider.publicKey,
        multisig: multisig.publicKey,
        transaction: transaction.publicKey,
        proposer: ownerA.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([transaction, ownerA])
      .rpc();

    const txAccount = await program.account.transaction.fetch(
      transaction.publicKey
    );

    assert.isTrue(txAccount.programId.equals(pid));
    assert.deepStrictEqual(txAccount.accounts, accounts);
    assert.deepStrictEqual(txAccount.data, data);
    assert.isTrue(txAccount.multisig.equals(multisig.publicKey));
    assert.deepStrictEqual(txAccount.didExecute, false);

    // Other owner approves transaction.
    await program.methods
      .approve()
      .accounts({
        multisig: multisig.publicKey,
        transaction: transaction.publicKey,
        owner: ownerB.publicKey,
      })
      .signers([ownerB])
      .rpc();

    const setOwnersIntruction = await program.methods
      .setOwners([])
      .accounts({
        multisig: multisig.publicKey,
        multisigSigner,
      })
      .instruction();

    // Now that we've reached the threshold, send the transaction.
    await program.methods
      .executeTransaction()
      .accounts({
        multisig: multisig.publicKey,
        multisigSigner,
        transaction: transaction.publicKey,
      })
      .remainingAccounts(
        setOwnersIntruction.keys
          // Change the signer status on the vendor signer since it's signed by the program, not the client.
          .map((meta) =>
            meta.pubkey.equals(multisigSigner)
              ? { ...meta, isSigner: false }
              : meta
          )
          .concat({
            pubkey: program.programId,
            isWritable: false,
            isSigner: false,
          })
      )
      .rpc();

    multisigAccount = await program.account.multisig.fetch(multisig.publicKey);

    assert.strictEqual(multisigAccount.nonce, nonce);
    assert.isTrue(multisigAccount.threshold.eq(new anchor.BN(2)));
    assert.deepStrictEqual(multisigAccount.owners, newOwners);
  });

  it("Assert Unique Owners", async () => {
    const multisig = anchor.web3.Keypair.generate();
    const [_multisigSigner, nonce] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [multisig.publicKey.toBuffer()],
        program.programId
      );

    const ownerA = anchor.web3.Keypair.generate();
    const ownerB = anchor.web3.Keypair.generate();
    const owners = [ownerA.publicKey, ownerB.publicKey, ownerA.publicKey];

    const threshold = new anchor.BN(2);
    try {
      await program.methods
        .createMultisig(owners, threshold, nonce)
        .accounts({
          payer: program.provider.publicKey,
          multisig: multisig.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([multisig])
        .rpc();
      assert.fail();
    } catch (err) {
      console.log(err);
      const error = err.error;
      assert.strictEqual(error.errorCode.number, 6008);
      assert.strictEqual(error.errorMessage, "Owners must be unique");
    }
  });
});
