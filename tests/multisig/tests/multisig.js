const anchor = require("@coral-xyz/anchor");
const { assert } = require("chai");

describe("multisig", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.getProvider());

  const program = anchor.workspace.CoralMultisig;

  it("Tests the multisig program", async () => {
    const multisig = anchor.web3.Keypair.generate();
    const [multisigSigner, nonce] =
      await anchor.web3.PublicKey.findProgramAddress(
        [multisig.publicKey.toBuffer()],
        program.programId
      );
    const multisigSize = 200; // Big enough.

    const ownerA = anchor.web3.Keypair.generate();
    const ownerB = anchor.web3.Keypair.generate();
    const ownerC = anchor.web3.Keypair.generate();
    const ownerD = anchor.web3.Keypair.generate();
    const owners = [ownerA.publicKey, ownerB.publicKey, ownerC.publicKey];

    const threshold = new anchor.BN(2);
    await program.rpc.createMultisig(owners, threshold, nonce, {
      accounts: {
        multisig: multisig.publicKey,
      },
      instructions: [
        await program.account.multisig.createInstruction(
          multisig,
          multisigSize
        ),
      ],
      signers: [multisig],
    });

    let multisigAccount = await program.account.multisig.fetch(
      multisig.publicKey
    );
    assert.strictEqual(multisigAccount.nonce, nonce);
    assert.isTrue(multisigAccount.threshold.eq(new anchor.BN(2)));
    assert.deepStrictEqual(multisigAccount.owners, owners);
    assert.isTrue(multisigAccount.ownerSetSeqno === 0);

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
    const txSize = 1000; // Big enough, cuz I'm lazy.
    await program.rpc.createTransaction(pid, accounts, data, {
      accounts: {
        multisig: multisig.publicKey,
        transaction: transaction.publicKey,
        proposer: ownerA.publicKey,
      },
      instructions: [
        await program.account.transaction.createInstruction(
          transaction,
          txSize
        ),
      ],
      signers: [transaction, ownerA],
    });

    const txAccount = await program.account.transaction.fetch(
      transaction.publicKey
    );

    assert.isTrue(txAccount.programId.equals(pid));
    assert.deepStrictEqual(txAccount.accounts, accounts);
    assert.deepStrictEqual(txAccount.data, data);
    assert.isTrue(txAccount.multisig.equals(multisig.publicKey));
    assert.deepStrictEqual(txAccount.didExecute, false);
    assert.isTrue(txAccount.ownerSetSeqno === 0);

    // Other owner approves transaction.
    await program.rpc.approve({
      accounts: {
        multisig: multisig.publicKey,
        transaction: transaction.publicKey,
        owner: ownerB.publicKey,
      },
      signers: [ownerB],
    });

    // Now that we've reached the threshold, send the transaction.
    await program.rpc.executeTransaction({
      accounts: {
        multisig: multisig.publicKey,
        multisigSigner,
        transaction: transaction.publicKey,
      },
      remainingAccounts: program.instruction.setOwners
        .accounts({
          multisig: multisig.publicKey,
          multisigSigner,
        })
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
        }),
    });

    multisigAccount = await program.account.multisig.fetch(multisig.publicKey);

    assert.strictEqual(multisigAccount.nonce, nonce);
    assert.isTrue(multisigAccount.threshold.eq(new anchor.BN(2)));
    assert.deepStrictEqual(multisigAccount.owners, newOwners);
    assert.isTrue(multisigAccount.ownerSetSeqno === 1);
  });

  it("Assert Unique Owners", async () => {
    const multisig = anchor.web3.Keypair.generate();
    const [_multisigSigner, nonce] =
      await anchor.web3.PublicKey.findProgramAddress(
        [multisig.publicKey.toBuffer()],
        program.programId
      );
    const multisigSize = 200; // Big enough.

    const ownerA = anchor.web3.Keypair.generate();
    const ownerB = anchor.web3.Keypair.generate();
    const owners = [ownerA.publicKey, ownerB.publicKey, ownerA.publicKey];

    const threshold = new anchor.BN(2);
    try {
      await program.rpc.createMultisig(owners, threshold, nonce, {
        accounts: {
          multisig: multisig.publicKey,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        instructions: [
          await program.account.multisig.createInstruction(
            multisig,
            multisigSize
          ),
        ],
        signers: [multisig],
      });
      assert.fail();
    } catch (err) {
      const error = err.error;
      assert.strictEqual(error.errorCode.number, 6008);
      assert.strictEqual(error.errorMessage, "Owners must be unique");
    }
  });
});